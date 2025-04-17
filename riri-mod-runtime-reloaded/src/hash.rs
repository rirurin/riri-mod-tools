#![allow(unused_imports)]
use riri_mod_tools_rt::logln;
use rkyv::{
    Archive,
    Deserialize,
    rancor::{
        Error as RkyvError,
        Fallible,
        Source
    },
    Serialize,
    ser::Writer as RkyvWriter
};
use std::{
    error::Error,
    mem::MaybeUninit,
    path::{ Path, PathBuf },
    sync::OnceLock,
    time::SystemTime
};
use twox_hash::XxHash3_64;
use windows::Win32::{
    Foundation::{ FILETIME, HMODULE, SYSTEMTIME },
    Storage::FileSystem,
    System::{ ProcessStatus, Threading, Time }
};


#[derive(Debug)]
pub struct WriteTime(SystemTime);
impl Archive for WriteTime {
    type Archived = [u8; size_of::<SystemTime>()];
    type Resolver = ();
    fn resolve(&self, _resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        let val: [u8; size_of::<SystemTime>()] = unsafe { std::slice::from_raw_parts(
            &raw const self.0 as *const u8, size_of::<SystemTime>()).try_into().unwrap() };
        out.write(val);
    }
}

impl<D> Deserialize<WriteTime, D> for [u8; size_of::<SystemTime>()]
where D: Fallible + ?Sized,
      D::Error: Source
{
    fn deserialize(&self, _: &mut D) -> Result<WriteTime, <D as Fallible>::Error> {
        Ok(unsafe { std::mem::transmute::<_, _>(*self) })
    }
}

impl<S> Serialize<S> for WriteTime
where S: Fallible + RkyvWriter
{
    fn serialize(&self, _: &mut S)
        -> Result<Self::Resolver, <S as Fallible>::Error> {
        Ok(())
    }
}

#[derive(Debug)]
#[derive(Archive, Deserialize, Serialize)]
pub struct ExecutableEntry {
    full_path: String,
    write_time: WriteTime,
    hash: u64
}

#[derive(Debug)]
#[derive(Archive, Deserialize, Serialize)]
pub struct ExecutableData {
    entries: Vec<ExecutableEntry>
}
impl ExecutableData {
    fn new() -> Self {
        Self { entries: Vec::new() }
    }

    fn add_entry<P>(&mut self, path: P) -> Result<u64, Box<dyn Error>> where P: AsRef<Path> {
        let full_path = path.as_ref().to_str().unwrap().to_owned();
        let write_time = WriteTime(std::fs::metadata(path.as_ref())?.modified()?);
        let hash = get_new_hash(path)?;
        self.entries.push(ExecutableEntry { full_path, write_time, hash });
        Ok(hash)
    }
}

pub(crate) static EXECUTABLE_HASH: OnceLock<u64> = OnceLock::new();

// This function is slow! We need to make sure to only call it when necessary!
fn get_new_hash<P>(path: P) -> Result<u64, Box<dyn Error>> where P: AsRef<Path> {
    let exec = std::fs::read(path.as_ref())?;
    let hash = XxHash3_64::oneshot(exec.as_slice());
    Ok(hash)
}

fn generate_cache<P>(path: P, meta_path: P) -> Result<(), Box<dyn Error>> where P: AsRef<Path> {
    // No executable cache, we'll have to make a new one
    let mut new = ExecutableData::new();
    let hash = new.add_entry(&path)?;
    logln!(Information, "New executable info cache created, {} generated hash 0x{:x}", path.as_ref().to_str().unwrap(), hash);
    let serialized = rkyv::to_bytes::<RkyvError>(&new)?;
    std::fs::write(&meta_path, serialized)?;
    let _ = EXECUTABLE_HASH.set(hash);
    Ok(())
}

pub fn set_executable_hash() -> Result<(), Box<dyn Error>> {
    // Get currently running executable name
    let curr_handle = unsafe { Threading::GetCurrentProcess() };
    let mut main_module: MaybeUninit<HMODULE> = MaybeUninit::uninit();
    let mut mod_sz_total: u32 = 0;
    unsafe { ProcessStatus::EnumProcessModules(
        curr_handle, 
        main_module.as_mut_ptr(), 
        std::mem::size_of::<HMODULE>() as u32,
        &raw mut mod_sz_total
    )? };
    let main_module = unsafe { main_module.assume_init() };
    let mut file_path: MaybeUninit<[u8; 260]> = MaybeUninit::uninit();
    let path_len = unsafe { ProcessStatus::GetModuleFileNameExA(
        Some(curr_handle), 
        Some(main_module), 
        &mut file_path.assume_init_mut()[..]) 
    };
    let exe_path = PathBuf::from(unsafe { std::str::from_utf8_unchecked(&file_path.assume_init_ref()[..path_len as usize]) });
    let fmt_path = exe_path.to_str().unwrap();
    // Check the executable info file to see what the last written time is.
    let mod_dir: String = riri_mod_tools_rt::mod_loader_data::get_directory_for_mod().into();
    let meta_path = PathBuf::from(mod_dir).join("exe_info");
    if std::fs::exists(&meta_path)? {
        // Find entry
        let bytes = std::fs::read(&meta_path)?;
        let arch_file = match rkyv::access::<ArchivedExecutableData, RkyvError>(bytes.as_slice()) {
            Ok(v) => v,
            Err(_) => {
                // Something is wrong with the file, regenerate
                generate_cache(&exe_path, &meta_path)?;
                return Ok(());
            }
        };
        let mut file = rkyv::deserialize::<ExecutableData, RkyvError>(arch_file)?;
        let res = match file.entries.iter_mut().find(|v| v.full_path == fmt_path) {
            Some(v) => { // Check that timestamp matches
                let real_write_time = std::fs::metadata(&exe_path)?.modified()?;
                let update = if v.write_time.0 == real_write_time { 
                    logln!(Information, "{}: Use cached hash 0x{:x}", fmt_path, v.hash);
                    false 
                } else {
                    v.hash = get_new_hash(&exe_path)?;
                    v.write_time = WriteTime(real_write_time);
                    logln!(Information, "{} has been modified, updated hash entry to 0x{:x}", fmt_path, v.hash);
                    true
                };
                (v.hash, update)
            },
            None => { // Not in list, add new entry
                let hash = file.add_entry(&exe_path)?;
                logln!(Information, "{} is not in the executable info cache, generated new hash 0x{:x}", fmt_path, hash);
                (hash, true)
            }
        };
        if res.1 {
            let serialized = rkyv::to_bytes::<RkyvError>(&file)?;
            std::fs::write(&meta_path, serialized)?;
        }
        let _ = EXECUTABLE_HASH.set(res.0);
    } else {
        generate_cache(&exe_path, &meta_path)?;
    }
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn get_executable_hash_ex() -> u64 {
    *EXECUTABLE_HASH.get().unwrap()
}