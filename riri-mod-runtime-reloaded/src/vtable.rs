use cpp_types::msvc::type_info::{ ObjectLocator, TypeInfo };
use riri_mod_tools_rt::{ address::ProcessInfo, logln };
use rkyv::{
    Archive,
    bytecheck::CheckBytes,
    Deserialize,
    munge::munge,
    Portable,
    Serialize,
    rancor::{
        Error as RkyvError,
        Fallible,
        Source as RkyvErrorSource
    },
    ser::Writer as RkyvWriter
};
use std::{
    collections::HashMap,
    ffi::CStr,
    error::Error,
    path::PathBuf,
    sync::OnceLock
};

type VtableEntries = HashMap<VtableKey, u64>;

#[derive(Archive, Deserialize, Serialize)]
#[derive(Debug, Clone)]

pub struct VtableRTTICache {
    entries: HashMap<u64, VtableEntries>
}

impl VtableRTTICache {
    pub fn new_empty() -> Self { Self { entries: HashMap::new() }}
    pub fn new_from_entry(proc: &ProcessInfo, first: VtableEntries) -> Self {
        let mut cache = Self::new_empty();
        cache.entries.insert(proc.get_executable_hash(), first);
        cache
    }
    pub fn add(&mut self, proc: &ProcessInfo, entry: VtableEntries) {
        self.entries.insert(proc.get_executable_hash(), entry);
    }
    pub fn get(&self, proc: &ProcessInfo) -> Option<&VtableEntries> {
        self.entries.get(&proc.get_executable_hash())
    }
}

unsafe impl <C> CheckBytes<C> for VtableRTTICache 
where C: Fallible + ?Sized,
      C::Error: RkyvErrorSource {
    unsafe fn check_bytes(
            _value: *const Self,
            _context: &mut C,
        ) -> Result<(), <C as Fallible>::Error> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct VtableKey {
    name: String,
    offset: u32
}

unsafe impl <C> CheckBytes<C> for ArchivedVtableKey
where C: Fallible + ?Sized,
      C::Error: RkyvErrorSource {
    unsafe fn check_bytes(
            _value: *const Self,
            _context: &mut C,
        ) -> Result<(), <C as Fallible>::Error> {
        Ok(())
    }
}

#[repr(C)]
#[derive(Portable, PartialEq, Eq, Hash)]
pub struct ArchivedVtableKey {
    name: <String as Archive>::Archived,
    offset: <u32 as Archive>::Archived,
}

pub struct VtableKeyResolver {
    name: rkyv::string::StringResolver,
    offset: ()
}

impl Archive for VtableKey {
    type Archived = ArchivedVtableKey;
    type Resolver = VtableKeyResolver;
    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        munge!(let ArchivedVtableKey { name, offset } = out);
        self.name.resolve(resolver.name, name);
        self.offset.resolve(resolver.offset, offset);
    }
}
impl<D> Deserialize<VtableKey, D> for ArchivedVtableKey
where D: Fallible + ?Sized,
      D::Error: RkyvErrorSource
{
    fn deserialize(&self, deserializer: &mut D) -> Result<VtableKey, <D as Fallible>::Error> {
        let name = self.name.deserialize(deserializer)?;
        let offset = self.offset.deserialize(deserializer)?;
        Ok(VtableKey { name, offset })
    }
}

impl<S> Serialize<S> for VtableKey
where S: Fallible + RkyvWriter,
      S::Error: RkyvErrorSource
{
    fn serialize(&self, serializer: &mut S)
        -> Result<Self::Resolver, <S as Fallible>::Error> {
        Ok(Self::Resolver {
            name: rkyv::string::ArchivedString::serialize_from_str(&self.name, serializer)?,
            offset: ()
        })
    }
}

impl VtableKey {
    pub fn new(name: String, offset: u32) -> Self {
        Self { name, offset }
    }
}

// const RTTI_CLASS_NAME_PREFIX: &'static str = ".?AV";
const RTTI_CLASS_NAME_PREFIX: &'static [u8] = &[0x2e, 0x3f, 0x41, 0x56];

fn generate_vtable_entries(proc: &ProcessInfo) -> VtableEntries {
    let mem = proc.get_process_memory();
    let sec = proc.get_memory_sections();
    let first_offset = sec.first().unwrap().get_virtual_address() as usize - proc.get_executable_address();
    // Make executable as a list of ObjectLocstor pointers expcet for the header.
    let slice = unsafe { std::slice::from_raw_parts(
        mem.as_ptr().add(first_offset) as *const &ObjectLocator, 
        (mem.len() - first_offset) / size_of::<*const ObjectLocator>()) 
    };
    let mut vtables: VtableEntries = HashMap::new();
    // let mut table_count = 0;
    for s in slice {
        // A pointer to something, verify it's ObjectLocator
        if proc.in_executable(*s) {
            let type_info = unsafe { &*((proc.get_executable_address() + s.get_type_info_offset() as usize) as *const TypeInfo) };
            if proc.in_executable(type_info) {
                // Check that TypeInfo + 1 contains the start of an decorated name
                let name_start = unsafe { std::slice::from_raw_parts((&raw const *type_info).add(1) as *const u8, RTTI_CLASS_NAME_PREFIX.len()) };
                if name_start == RTTI_CLASS_NAME_PREFIX {
                    let name = &type_info.get_decorated_name()[RTTI_CLASS_NAME_PREFIX.len() - 1..];
                    let vtable = unsafe { (&raw const *s as *const usize).add(1) } as usize;
                    let vtable = (vtable - slice.as_ptr() as usize + first_offset) as u64;
                    let key = VtableKey::new(name.to_owned(), s.get_struct_offset());
                    if vtables.contains_key(&key) {
                        logln!(Verbose, "WARNING: The vtable {:?} was already located!", key);
                    }
                    vtables.insert(key, vtable);
                }
            }
        }
    }
    vtables
}

pub fn extract_vtables_msvc() -> Result<(), Box<dyn Error>> {
    let proc = ProcessInfo::get_current_process().unwrap();
    // check cache if it exists
    let mod_dir: String = riri_mod_tools_rt::mod_loader_data::get_directory_for_mod().into();
    let meta_path = PathBuf::from(mod_dir).join("vtable_rtti_msvc");
    let (mut defs, regen) = if std::fs::exists(&meta_path)? { // Find entry
        let bytes = std::fs::read(&meta_path)?;
        // try loading if the format is valid, otherwise regen
        match rkyv::access::<ArchivedVtableRTTICache, RkyvError>(bytes.as_slice()) {
            Ok(v) => match rkyv::deserialize::<VtableRTTICache, RkyvError>(v) {
                Ok(v) => (v, false),
                Err(_) => (VtableRTTICache::new_from_entry(&proc, generate_vtable_entries(&proc)), true)
            },
            Err(_) => (VtableRTTICache::new_from_entry(&proc, generate_vtable_entries(&proc)), true)
        }
    } else { (VtableRTTICache::new_from_entry(&proc, generate_vtable_entries(&proc)), true) };
    match defs.get(&proc) {
        Some(e) => {
            /* 
            for (k, v) in e {
                logln!(Verbose, "0x{:x} -> {:?}", v, k);
            }
            */
            let _ = VTABLES_FROM_RTTI.set((*e).clone());
            if regen {
                let serialized = rkyv::to_bytes::<RkyvError>(&defs)?;
                std::fs::write(&meta_path, serialized)?;
                logln!(Verbose, "Regenerated vtable_rtti_msvc");
            } else {
                logln!(Verbose, "Using cached entry for 0x{:x} from vtable_rtti_msvc", proc.get_executable_hash());
            }
        },
        None => {
            let new = generate_vtable_entries(&proc);
            let _ = VTABLES_FROM_RTTI.set(new.clone());
            defs.add(&proc, new);
            let serialized = rkyv::to_bytes::<RkyvError>(&defs)?;
            std::fs::write(&meta_path, serialized)?;
            logln!(Verbose, "Added new entry for executable 0x{:x} into vtable_rtti_msvc", proc.get_executable_hash());
        }
    }
    Ok(())
}

static VTABLES_FROM_RTTI: OnceLock<VtableEntries> = OnceLock::new();

// FFI API

#[no_mangle]
pub unsafe extern "C" fn get_vtable_rtti(name: *const i8, offset: u32) -> *const u8 {
    let proc = ProcessInfo::get_current_process().unwrap();
    
    let name = CStr::from_ptr(name).to_str().unwrap();
    let key = VtableKey::new(name.to_owned(), offset);
    match VTABLES_FROM_RTTI.get().unwrap().get(&key) {
        Some(t) => (*t as usize + proc.get_executable_address()) as *const u8,
        None => std::ptr::null()
    }
}