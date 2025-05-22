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
    unsafe { get_vtable_rtti(name.as_ptr() as *const i8, offset) }
}