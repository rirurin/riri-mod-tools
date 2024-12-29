//! Utilities to assist with transforming an address after it's found using Relaaded-II's
//! sigscanning library, Reloaded.Memory.SigScan. It provides a method AddMainModuleScan which
//! returns an offset as an i32 relative to the beginning of the executable. The methods assume
//! that a result was successfully found.
//!
//! If the resulting value is found to be outside of the range of the module's memory map, a None
//! will be returned.
//! Currently, only the main module (executable) is supported.

use crate::address::ProcessInfo;
use std::{
    ptr::NonNull,
    sync::OnceLock
};

pub static CURRENT_PROCESS: OnceLock<ProcessInfo> = OnceLock::new();

type CanNull = Option<NonNull<u8>>;

#[inline(always)]
fn base_address() -> *mut u8 {
    CURRENT_PROCESS.get().unwrap().get_executable_address() as *mut u8
}
#[inline(always)]
fn module_size() -> usize {
    CURRENT_PROCESS.get().unwrap().get_executable_size()
}

// Set the target process that the DLL is attached to. 
#[no_mangle]
pub extern "C" fn set_current_process() {
    CURRENT_PROCESS.set(ProcessInfo::get_current_process().unwrap()).unwrap();
}

unsafe fn deref_int_relative_pointer(p: *mut u8) -> CanNull {
    let res = deref_int_relative_pointer_unchecked(p); 
    if res > base_address() && res < base_address().add(module_size()) {
        Some(NonNull::new(res).unwrap())
    } else {
        None
    }
}

unsafe fn deref_int_relative_pointer_unchecked(p: *mut u8) -> *mut u8 {
    let ofs = std::ptr::read_unaligned::<i32>(p as *const i32);
    p.offset(ofs as isize + 4)
}


#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
unsafe fn deref_instruction_pointer_short(p: *mut u8) -> CanNull {
    try_deref_instruction_pointer(p.offset((*p.add(1) as i8 + 2) as isize))
}

#[cfg(target_arch = "x86_64")]
unsafe fn deref_instruction_pointer_near(p: *mut u8) -> CanNull { 
    try_deref_instruction_pointer(deref_int_relative_pointer_unchecked(p))
}

#[cfg(target_arch = "x86_64")]
unsafe fn deref_instruction_pointer_long(_p: *mut u8) -> CanNull {
    todo!("long jump todo! (first byte was 0xff)")
}

#[cfg(target_arch = "x86_64")]
unsafe fn try_deref_instruction_pointer(p: *mut u8) -> CanNull {
    // If the pointer is outside of the executable, return a null pointer
    if p >= base_address().add(module_size()) || p < base_address() {
        return None;
    }
    match *p {
        0xeb => deref_instruction_pointer_short(p),        
        0xe9 => deref_instruction_pointer_near(p),
        0xff => deref_instruction_pointer_long(p),
        _ => NonNull::new(p)
    }
}

/// Get an absolute executable address given an offset. Assumes that the sigscanner library 
/// always returns a value within the executable's memory map (Reloaded.Memory.Sigscan does this). 
/// If it's possible that the offset result will point to a jump instruction, such as with a thunk
/// function, use `get_address_may_thunk` instead. 
///
/// # Example
/// In P3R.exe: External sigscanning library searches for value matching
/// "40 53 48 83 EC 30 F3 0F 10 05 ?? ?? ?? ?? 48 89 CB" -> 0xbe34240
/// ```
/// let set_bustup_shadow_color = get_direct_address(0xbe34240);
/// // returns 0x14be34240
/// ```
#[no_mangle]
pub unsafe extern "C" fn get_address(ofs: usize) -> NonNull<u8> {
    NonNull::new(base_address().add(ofs)).unwrap()
    // get_address_exec(base_address().add(ofs))
}
/*
unsafe fn get_address_exec(ptr: *mut u8) -> NonNull<u8> {
    NonNull::new(ptr).unwrap()
}
*/

/// Like `get_address`, but contains extra logic to travel through jump instructions until it
/// reaches a non-jump. As of Reloaded-II's hooking library, hooks crash on thunked functions, so
/// this provides a convenient method to work around that. If resolve_type is not specified in
/// riri_hook, this resolver is used by default. Additionally, #[cpp_class] uses this when
/// hooking functions from a vtable pointer.
#[no_mangle]
pub unsafe extern "C" fn get_address_may_thunk(ofs: usize) -> CanNull {
    try_deref_instruction_pointer(base_address().add(ofs))
    // get_address_may_thunk_exec(base_address().add(ofs))
}
/*
unsafe fn get_address_may_thunk_exec(ptr: *mut u8) -> CanNull {
    try_deref_instruction_pointer(ptr)
}
*/

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the second byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short(ofs: usize) -> CanNull {
    deref_int_relative_pointer(base_address().add(ofs + 1))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer(addr.add(1))
}

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the third byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short2(ofs: usize) -> CanNull {
    deref_int_relative_pointer(base_address().add(ofs + 2))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short2_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer(addr.add(2))
}

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the fourth byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long(ofs: usize) -> CanNull {
    deref_int_relative_pointer(base_address().add(ofs + 3))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer(addr.add(3))
}

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the fifth byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long4(ofs: usize) -> CanNull {
    deref_int_relative_pointer(base_address().add(ofs + 4))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long4_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer(addr.add(4))
}
