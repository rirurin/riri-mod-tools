use std::alloc::{alloc, dealloc, Layout};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::OnceLock;

type SetPushParameter = unsafe extern "C" fn(u64, u64, usize);
static PUSH_PARAMETER: OnceLock<SetPushParameter> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_push_parameter(cb: SetPushParameter) {
    _ = PUSH_PARAMETER.set(cb).unwrap();
}

pub unsafe fn push_parameter(type_hash: u64, method_hash: u64, object: usize) {
    PUSH_PARAMETER.get().unwrap()(type_hash, method_hash, object)
}

type SetCallFunction = unsafe extern "C" fn(u64, u64, usize) -> usize;
static CALL_FUNCTION: OnceLock<SetCallFunction> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_call_function(cb: SetCallFunction) {
    _ = CALL_FUNCTION.set(cb).unwrap();
}

pub unsafe fn call_function(type_hash: u64, method_hash: u64, object: usize) -> usize {
    CALL_FUNCTION.get().unwrap()(type_hash, method_hash, object)
}

type SetFreeObject = unsafe extern "C" fn(usize);
static FREE_OBJECT: OnceLock<SetFreeObject> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_free_object(cb: SetFreeObject) {
    _ = FREE_OBJECT.set(cb).unwrap();
}

pub unsafe fn free_object(object: usize) {
    FREE_OBJECT.get().unwrap()(object)
}

type SetObjectAsU8 = unsafe extern "C" fn(usize, *mut bool) -> u8;
static OBJECT_AS_U8: OnceLock<SetObjectAsU8> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u8(cb: SetObjectAsU8) {
    _ = OBJECT_AS_U8.set(cb).unwrap();
}

pub unsafe fn object_as_u8(object: usize, success: *mut bool) -> u8 {
    OBJECT_AS_U8.get().unwrap()(object, success)
}

type SetObjectAsU16 = unsafe extern "C" fn(usize, *mut bool) -> u16;
static OBJECT_AS_U16: OnceLock<SetObjectAsU16> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u16(cb: SetObjectAsU16) {
    _ = OBJECT_AS_U16.set(cb).unwrap();
}

pub unsafe fn object_as_u16(object: usize, success: *mut bool) -> u16 {
    OBJECT_AS_U16.get().unwrap()(object, success)
}

type SetObjectAsU32 = unsafe extern "C" fn(usize, *mut bool) -> u32;
static OBJECT_AS_U32: OnceLock<SetObjectAsU32> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u32(cb: SetObjectAsU32) {
    _ = OBJECT_AS_U32.set(cb).unwrap();
}

pub unsafe fn object_as_u32(object: usize, success: *mut bool) -> u32 {
    OBJECT_AS_U32.get().unwrap()(object, success)
}

type SetObjectAsU64 = unsafe extern "C" fn(usize, *mut bool) -> u64;
static OBJECT_AS_U64: OnceLock<SetObjectAsU64> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u64(cb: SetObjectAsU64) {
    _ = OBJECT_AS_U64.set(cb).unwrap();
}

pub unsafe fn object_as_u64(object: usize, success: *mut bool) -> u64 {
    OBJECT_AS_U64.get().unwrap()(object, success)
}

type SetGetObjectInstance = unsafe extern "C" fn(*const ObjectInitializer) -> usize;
static GET_OBJECT_INSTANCE: OnceLock<SetGetObjectInstance> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_get_object_instance(cb: SetGetObjectInstance) {
    _ = GET_OBJECT_INSTANCE.set(cb).unwrap();
}

pub unsafe fn get_object_instance(initializer: &ObjectInitializer) -> usize {
    GET_OBJECT_INSTANCE.get().unwrap()(initializer)
}

type SetObjectAsString = unsafe extern "C" fn(usize, *mut bool) -> usize;
static OBJECT_AS_STRING: OnceLock<SetObjectAsString> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_string(cb: SetObjectAsString) {
    _ = OBJECT_AS_STRING.set(cb).unwrap();
}

pub unsafe fn object_as_string(object: usize, success: *mut bool) -> usize {
    OBJECT_AS_STRING.get().unwrap()(object, success)
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ObjectInitializer {
    hash: u64,
    size: usize
}

#[derive(Debug)]
pub struct ObjectInitHandle<T>
where T: Copy {
    alloc: NonNull<ObjectInitializer>,
    marker: PhantomData<T>
}

impl<T> ObjectInitHandle<T>
where T: Copy {
    pub fn new(hash: u64, value: T) -> Self {
        let mut alloc: NonNull<ObjectInitializer> = unsafe { NonNull::new_unchecked(alloc(Self::get_layout()) as _) };
        unsafe {
            alloc.as_mut().hash = hash;
            alloc.as_mut().size = size_of::<T>();
            *(alloc.as_ptr().add(1) as *mut T) = value;
        }
        Self { alloc, marker: PhantomData::<T> }
    }

    pub fn get_layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(
            size_of::<ObjectInitializer>() + size_of::<T>(),
            align_of::<ObjectInitializer>().max(align_of::<T>()))
        }
    }

    pub fn set(&mut self, value: T) {
        unsafe { *(self.alloc.as_ptr().add(1) as *mut T) = value; }
    }

    pub fn get(&self) -> &T {
        unsafe { &*(self.alloc.as_ptr().add(1) as *const T) }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.alloc.as_ptr().add(1) as *mut T) }
    }

    pub fn raw(&self) -> &ObjectInitializer {
        unsafe { self.alloc.as_ref() }
    }
}

impl<T> Drop for ObjectInitHandle<T>
where T: Copy {
    fn drop(&mut self) {
        unsafe { dealloc(self.alloc.as_ptr() as _, Self::get_layout()) };
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum InteropError {
    CouldNotMakeObjectInstance,
    CouldNotCastToValue
}

impl Display for InteropError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <InteropError as Debug>::fmt(self, f)
    }
}

impl Error for InteropError {}

pub trait ObjectValuable where Self: Sized {
    type ValueType;
    fn value(&self) -> Result<Self::ValueType, InteropError>;
}

pub trait ObjectInitializable where Self: Sized {
    type InitType;
    fn new(value: Self::InitType) -> Result<Self, InteropError>;
}

pub trait ObjectHash where Self: Sized {
    const HASH: u64;
}