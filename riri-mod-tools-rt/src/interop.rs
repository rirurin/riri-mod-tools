#![allow(dead_code)]

use std::alloc::{alloc, dealloc, Layout};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
// use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::ops::Deref;
use std::sync::OnceLock;
use crate::mod_loader_data::CSharpString;

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

type SetObjectAsU8 = unsafe extern "C" fn(usize) -> u8;
static OBJECT_AS_U8: OnceLock<SetObjectAsU8> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u8(cb: SetObjectAsU8) {
    _ = OBJECT_AS_U8.set(cb).unwrap();
}

pub unsafe fn object_as_u8(object: usize) -> u8 {
    OBJECT_AS_U8.get().unwrap()(object)
}

type SetObjectAsU16 = unsafe extern "C" fn(usize) -> u16;
static OBJECT_AS_U16: OnceLock<SetObjectAsU16> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u16(cb: SetObjectAsU16) {
    _ = OBJECT_AS_U16.set(cb).unwrap();
}

pub unsafe fn object_as_u16(object: usize) -> u16 {
    OBJECT_AS_U16.get().unwrap()(object)
}

type SetObjectAsU32 = unsafe extern "C" fn(usize) -> u32;
static OBJECT_AS_U32: OnceLock<SetObjectAsU32> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u32(cb: SetObjectAsU32) {
    _ = OBJECT_AS_U32.set(cb).unwrap();
}

pub unsafe fn object_as_u32(object: usize) -> u32 {
    OBJECT_AS_U32.get().unwrap()(object)
}

type SetObjectAsU64 = unsafe extern "C" fn(usize) -> u64;
static OBJECT_AS_U64: OnceLock<SetObjectAsU64> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_u64(cb: SetObjectAsU64) {
    _ = OBJECT_AS_U64.set(cb).unwrap();
}

pub unsafe fn object_as_u64(object: usize) -> u64 {
    OBJECT_AS_U64.get().unwrap()(object)
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

type SetObjectAsString = unsafe extern "C" fn(usize) -> usize;
static OBJECT_AS_STRING: OnceLock<SetObjectAsString> = OnceLock::new();

#[no_mangle]
pub unsafe extern "C" fn set_object_as_string(cb: SetObjectAsString) {
    _ = OBJECT_AS_STRING.set(cb).unwrap();
}

pub unsafe fn object_as_string(object: usize) -> usize {
    OBJECT_AS_STRING.get().unwrap()(object)
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

/*
#[derive(Debug)]
pub struct CSharpString(usize);

impl Deref for CSharpString {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for CSharpString {
    fn drop(&mut self) {
        unsafe { FREE_OBJECT.get().unwrap()(**self) }
    }
}

#[derive(Debug)]
pub struct IConfigurable(usize);

impl IConfigurable {
    pub unsafe fn get_config_name(&self) -> CSharpString {
        CSharpString(call_function(0x353e27b5f098294d, 0x94ef1d9227a8a5bc, **self))
    }

    pub unsafe fn set_mod_directory(&self, mod_directory: &CSharpString) {
        push_parameter(0x6ad4f16197263c81, 0x48816999ff6e398c, **mod_directory);
        call_function(0x6ad4f16197263c81, 0x48816999ff6e398c, **self);
    }
}

impl Deref for IConfigurable {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for IConfigurable {
    fn drop(&mut self) {
        unsafe { free_object(**self) };
    }
}
*/

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum InteropError {
    CouldNotMakeObjectInstance
}

impl Display for InteropError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <InteropError as Debug>::fmt(self, f)
    }
}

impl Error for InteropError {}

// Generate from your build script's interface bindgen
// riri_mod_tools_rt::system::Byte = 0x1db8be3780c4d69b
// riri_mod_tools_rt::system::SByte = 0x54e169cb81fff155
// riri_mod_tools_rt::system::Boolean = 0xbc513183a12017df
// riri_mod_tools_rt::system::Int16 = 0x332dbcf3e3e3a961
// riri_mod_tools_rt::system::UInt16 = 0x7a05331174d1347b
// riri_mod_tools_rt::system::Int32 = 0xe0342d8ff9932e02
// riri_mod_tools_rt::system::UInt32 = 0x33f722fd018171dc
// riri_mod_tools_rt::system::Int64 = 0x4d177c41298e2277
// riri_mod_tools_rt::system::UInt64 = 0x22b04bb57694ec47
// riri_mod_tools_rt::system::Single = 0x106efd909a7609a9
// riri_mod_tools_rt::system::Double = 0xdd6aef94114ecd43
// riri_mod_tools_rt::system::String = 0xd17d6432bd7c2cc9

#[derive(Debug)]
pub struct Int8(usize);

impl Deref for Int8 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Int8 {
    pub fn value(&self) -> i8 {
        unsafe { object_as_u8(**self) as i8 }
    }

    pub fn new(value: i8) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<i8>::new(0x54e169cb81fff155, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for Int8 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct UInt8(usize);

impl Deref for UInt8 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UInt8 {
    pub fn value(&self) -> u8 {
        unsafe { object_as_u8(**self) }
    }

    pub fn new(value: u8) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<u8>::new(0x1db8be3780c4d69b, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for UInt8 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct Int16(usize);

impl Deref for Int16 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Int16 {
    pub fn value(&self) -> i16 {
        unsafe { object_as_u16(**self) as i16 }
    }

    pub fn new(value: i16) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<i16>::new(0x332dbcf3e3e3a961, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for Int16 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct UInt16(usize);

impl Deref for UInt16 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UInt16 {
    pub fn value(&self) -> u16 {
        unsafe { object_as_u16(**self) }
    }

    pub fn new(value: u16) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<u16>::new(0x7a05331174d1347b, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for UInt16 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct Int32(usize);

impl Deref for Int32 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Int32 {
    pub fn value(&self) -> i32 {
        unsafe { object_as_u32(**self) as i32 }
    }

    pub fn new(value: i32) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<i32>::new(0xe0342d8ff9932e02, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for Int32 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct UInt32(usize);

impl Deref for UInt32 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UInt32 {
    pub fn value(&self) -> u32 {
        unsafe { object_as_u32(**self) }
    }

    pub fn new(value: u32) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<u32>::new(0x33f722fd018171dc, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for UInt32 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct Int64(usize);

impl Deref for Int64 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Int64 {
    pub fn value(&self) -> i64 {
        unsafe { object_as_u64(**self) as i64 }
    }

    pub fn new(value: i64) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<i64>::new(0x4d177c41298e2277, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for Int64 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct UInt64(usize);

impl Deref for UInt64 {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UInt64 {
    pub fn value(&self) -> u64 {
        unsafe { object_as_u64(**self) }
    }

    pub fn new(value: u64) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<u64>::new(0x22b04bb57694ec47, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for UInt64 {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct Bool(usize);

impl Deref for Bool {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Bool {
    pub fn value(&self) -> bool {
        unsafe { object_as_u8(**self) != 0 }
    }

    pub fn new(value: bool) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<bool>::new(0xbc513183a12017df, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for Bool {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct Float(usize);

impl Deref for Float {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Float {
    pub fn value(&self) -> f32 {
        f32::from_bits(unsafe { object_as_u32(**self) })
    }

    pub fn new(value: f32) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<f32>::new(0x106efd909a7609a9, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for Float {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug)]
pub struct Double(usize);

impl Deref for Double {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Double {
    pub fn value(&self) -> f64 {
        f64::from_bits(unsafe { object_as_u64(**self) })
    }

    pub fn new(value: f64) -> Result<Self, InteropError> {
        let handle = ObjectInitHandle::<f64>::new(0xdd6aef94114ecd43, value);
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for Double {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct StringData {
    ptr: *const u8,
    len: usize
}

#[derive(Debug)]
pub struct String(usize);

impl Deref for String {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl String {
    pub fn value(&self) -> std::string::String {
        // Use existing CSharpString struct from mod_loader_data for a unmanaged wrapper over wchar_t* string
        CSharpString::new(unsafe { object_as_string(**self) } as _).into()
    }

    pub fn new(value: &str) -> Result<Self, InteropError> {
        let mut handle = ObjectInitHandle::<MaybeUninit<StringData>>::new(
            0xd17d6432bd7c2cc9,
            MaybeUninit::uninit()
        );
        let str_data = unsafe { handle.get_mut().assume_init_mut() };
        str_data.ptr = value.as_ptr();
        str_data.len = value.len();
        let result = unsafe { get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(result))
        }
    }
}

impl Drop for String {
    fn drop(&mut self) {
        unsafe { free_object(**self) }
    }
}