use crate::address::ProcessInfo;

#[cfg(feature = "reloaded")]
#[link(name = "riri_mod_runtime_reloaded", kind = "raw-dylib")]
unsafe extern "C" {
    pub(crate) unsafe fn get_vtable_rtti(name: *const i8, offset: u32) -> *const u8;
}

pub fn get_vtable(name: &str) -> *const u8 {
    get_vtable_with_offset(name, 0)
}

pub fn get_vtable_with_offset(name: &str, offset: u32) -> *const u8 {
    // Add null terminator if it's missing
    let name = match name.as_bytes().last() {
        Some(i) => if *i != 0 { format!("{}\0", name) } 
            else { name.to_owned() },
        None => name.to_owned()
    };
    #[cfg(feature = "reloaded")]
    unsafe { get_vtable_rtti(name.as_ptr() as *const i8, offset) }
    #[cfg(not(feature = "reloaded"))]
    // TOOD: How will we represent Metaphor's C++ vtables on the server?
    std::ptr::null()
}

pub fn replace_vtable_method(name: &str, index: usize, handle_original: fn(usize), new_function: usize) -> bool {
    let vtable = get_vtable(name) as *mut usize;
    if vtable != std::ptr::null_mut() {
        handle_original(unsafe { *vtable.add(index) });
        // Often the portion of memory isn't set for writing, so set write permission
        let mut process = ProcessInfo::get_current_process().unwrap();
        unsafe { process.change_protection_raw(vtable.add(index) as *const u8, size_of::<usize>(), 0x40) }; // PAGE_EXECUTE_READWRITE
        unsafe { *vtable.add(index) = new_function };
        true
    } else { false }
}