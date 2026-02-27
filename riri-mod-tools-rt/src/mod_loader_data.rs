use std::sync::{Mutex, OnceLock};
#[cfg(feature = "reloaded")]
use crate::reloaded::r#mod::interfaces::IModConfig;
#[cfg(feature = "reloaded")]
use crate::interop::ObjectInitializable;
#[cfg(feature = "reloaded")]
use crate::system::Object;

type GetDirFn = unsafe extern "C" fn() -> *const u16;
type FreeStrFn = unsafe extern "C" fn(*const u16) -> ();
pub static GET_DIRECTORY_FOR_MOD: OnceLock<GetDirFn> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_get_directory_for_mod(cb: GetDirFn) {
    GET_DIRECTORY_FOR_MOD.set(cb).unwrap();
}

pub static GET_CONFIG_DIRECTORY: OnceLock<GetDirFn> = OnceLock::new();
#[no_mangle]
pub unsafe extern "C" fn set_get_config_directory(cb: GetDirFn) {
    GET_CONFIG_DIRECTORY.set(cb).unwrap();
}

pub static FREE_CSHARP_STRING: OnceLock<FreeStrFn> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_free_csharp_string(cb: FreeStrFn) {
    FREE_CSHARP_STRING.set(cb).unwrap();
}

type FindPatternFn = unsafe extern "C" fn(*const u8, *const u8, u32) -> *const u8;
pub static FIND_PATTERN_FUNC: OnceLock<FindPatternFn> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_find_pattern(cb: FindPatternFn) {
    FIND_PATTERN_FUNC.set(cb).unwrap();
}

#[repr(C)]
#[derive(Debug)]
pub struct CSharpString(*const u16);
impl CSharpString {
    pub fn new(p: *const u16) -> Self {
        Self(p)
    }
}
impl From<CSharpString> for String {
    fn from(value: CSharpString) -> Self {
        let mut len = 0;
        while unsafe { *value.0.add(len) } != 0 {
            len += 1;
        }
        let s = unsafe { std::slice::from_raw_parts(value.0, len) };
        String::from_utf16(s).unwrap()
    }
}
impl Drop for CSharpString {
    fn drop(&mut self) {
        unsafe { (FREE_CSHARP_STRING.get().unwrap())(self.0) }
    }
}

// IModLoader::GetDirectoryForModId
pub fn get_directory_for_mod() -> CSharpString {
    CSharpString::new(unsafe { GET_DIRECTORY_FOR_MOD.get().unwrap()() })
}

// IModLoader::GetModConfigDirectory
pub fn get_config_directory() -> CSharpString {
    CSharpString::new(unsafe { GET_CONFIG_DIRECTORY.get().unwrap()() })
}

pub fn find_pattern(text: &str, start: *const u8, len: u32) -> *const u8 {
    // Add null terminator if it's missing
    let text = match text.as_bytes().last() {
        Some(i) => if *i != 0 { format!("{}\0", text) } 
            else { text.to_owned() },
        None => text.to_owned()
    };
    unsafe { FIND_PATTERN_FUNC.get().unwrap()(text.as_ptr(), start, len) }
}

static ON_MOD_LOADING: Mutex<Vec<fn(&IModConfig)>> = Mutex::new(vec![]);

// OnModLoading API event
#[no_mangle]
pub unsafe extern "C" fn on_mod_loading(p_mod_config: usize) {
    let mod_config = IModConfig::new_unchecked(
        Object::new_unchecked(p_mod_config));
    let callbacks = ON_MOD_LOADING.lock().unwrap();
    (&*callbacks).iter().for_each(|cb| cb(&mod_config));
}

pub fn add_on_mod_loading_callback(cb: fn(&IModConfig)) {
    ON_MOD_LOADING.lock().unwrap().push(cb);
}