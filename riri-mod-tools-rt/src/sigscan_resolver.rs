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

#[allow(dead_code)]
#[inline(always)]
fn module_size() -> usize {
    CURRENT_PROCESS.get().unwrap().get_executable_size()
}

pub trait PointerBounds {
    unsafe fn in_bounds(p: *mut u8) -> bool;
}
pub struct PointerExecutableBound;
pub struct PointerUnbounded;

impl PointerBounds for PointerExecutableBound {
    unsafe fn in_bounds(res: *mut u8) -> bool {
        res > base_address() && res < base_address().add(module_size())
    }
}
impl PointerBounds for PointerUnbounded {
    unsafe fn in_bounds(res: *mut u8) -> bool {
        res != std::ptr::null_mut()
    }
}

// Set the target process that the DLL is attached to. 
#[no_mangle]
pub extern "C" fn set_current_process() {
    CURRENT_PROCESS.set(ProcessInfo::get_current_process().unwrap()).unwrap();
}

unsafe fn deref_int_relative_pointer<B>(p: *mut u8) -> CanNull 
where B: PointerBounds {
    let res = deref_int_relative_pointer_unchecked(p); 
    if B::in_bounds(res) {
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
unsafe fn deref_instruction_pointer_short<B>(p: *mut u8) -> CanNull 
where B: PointerBounds {
    try_deref_instruction_pointer::<B>(p.offset((*p.add(1) as i8 + 2) as isize))
}

#[cfg(target_arch = "x86_64")]
unsafe fn deref_instruction_pointer_near<B>(p: *mut u8) -> CanNull 
where B: PointerBounds { 
    try_deref_instruction_pointer::<B>(deref_int_relative_pointer_unchecked(p))
}

unsafe fn deref_long_absolute_pointer_unchecked(p: *mut u8) -> *mut u8 {
    let abs = std::ptr::read_unaligned::<i64>(p as *const i64);
    abs as *mut u8
}

#[cfg(target_arch = "x86_64")]
unsafe fn deref_instruction_pointer_long<B>(p: *mut u8) -> CanNull 
where B: PointerBounds {
    try_deref_instruction_pointer::<B>(deref_long_absolute_pointer_unchecked(p))    
}

#[cfg(target_arch = "x86_64")]
unsafe fn try_deref_instruction_pointer<B>(p: *mut u8) -> CanNull 
where B: PointerBounds {
    // If the pointer is outside of the executable, return a null pointer
    if !B::in_bounds(p) { return None; }
    match *p {
        0xeb => deref_instruction_pointer_short::<B>(p),
        0xe9 => deref_instruction_pointer_near::<B>(p.add(1)),
        0xff => {
            // *4 (Jump near, absolute indirect)
            // *5 (Jump far, absolute indirect)
            let second_byte = *p.add(1) & 0xf;
            if second_byte == 5 {
                // 2 bytes for instruction + 4 bytes for ????
                deref_instruction_pointer_long::<B>(p.add(6))
            } else {
                NonNull::new(p)
            }
        },
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
/// ```ignore
/// let set_bustup_shadow_color = get_direct_address(0xbe34240);
/// // returns 0x14be34240
/// ```
#[no_mangle]
pub unsafe extern "C" fn get_address(ofs: usize) -> NonNull<u8> {
    NonNull::new(base_address().add(ofs)).unwrap()
}

/// Like `get_address`, but contains extra logic to travel through jump instructions until it
/// reaches a non-jump. As of Reloaded-II's hooking library, hooks crash on thunked functions, so
/// this provides a convenient method to work around that. If resolve_type is not specified in
/// riri_hook, this resolver is used by default. Additionally, #[cpp_class] uses this when
/// hooking functions from a vtable pointer.
#[no_mangle]
pub unsafe extern "C" fn get_address_may_thunk(ofs: usize) -> CanNull {
    try_deref_instruction_pointer::<PointerUnbounded>(base_address().add(ofs))
}

#[no_mangle]
pub unsafe extern "C" fn get_address_may_thunk_absolute(addr: usize) -> CanNull {
    try_deref_instruction_pointer::<PointerUnbounded>(addr as *mut u8)
}

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the second byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short(ofs: usize) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(base_address().add(ofs + 1))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(addr.add(1))
}

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the third byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short2(ofs: usize) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(base_address().add(ofs + 2))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_short2_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(addr.add(2))
}

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the fourth byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long(ofs: usize) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(base_address().add(ofs + 3))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(addr.add(3))
}

/// Function to dereference an arbitrary near pointer, where the pointee is an address within the
/// memory map of the executable. This is used for instructions where the near pointer starts at
/// the fifth byte of the target instruction.
#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long4(ofs: usize) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(base_address().add(ofs + 4))
}

#[no_mangle]
pub unsafe extern "C" fn get_indirect_address_long4_abs(addr: *mut u8) -> CanNull {
    deref_int_relative_pointer::<PointerUnbounded>(addr.add(4))
}

/// Get the hash of the target executable. Useful for selectively applying signatures by hash
/// if a signature breaks between game updates
#[no_mangle]
#[cfg(feature = "reloaded")]
pub unsafe extern "C" fn get_executable_hash() -> u64 {
    crate::address::get_executable_hash_ex()
}

/// Get the hash of the target executable. Useful for selectively applying signatures by hash
/// if a signature breaks between game updates
#[no_mangle]
#[cfg(not(feature = "reloaded"))]
pub unsafe extern "C" fn get_executable_hash() -> u64 {
    u64::MAX
}