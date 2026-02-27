#![allow(dead_code)]

use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use crate::mod_loader_data::CSharpString;
use crate::interop::{InteropError, ObjectHash, ObjectInitializable, ObjectValuable};


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
pub struct Object(usize);

impl Object {
    pub unsafe fn new_unchecked(ptr: usize) -> Self {
        Self(ptr)
    }
}

impl Deref for Object {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        unsafe { crate::interop::free_object(**self) }
    }
}

#[derive(Debug)]
pub struct Int8(Object);

impl Deref for Int8 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for Int8 {
    const HASH: u64 = 0x54e169cb81fff155;
}

impl ObjectValuable for Int8 {
    type ValueType = i8;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u8(***self, &mut success) as i8 };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for Int8 {
    type InitType = i8;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<i8>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct UInt8(Object);

impl Deref for UInt8 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for UInt8 {
    const HASH: u64 = 0x1db8be3780c4d69b;
}

impl ObjectValuable for UInt8 {
    type ValueType = u8;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u8(***self, &mut success) };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for UInt8 {
    type InitType = u8;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<u8>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Int16(Object);

impl Deref for Int16 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for Int16 {
    const HASH: u64 = 0x332dbcf3e3e3a961;
}

impl ObjectValuable for Int16 {
    type ValueType = i16;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u16(***self, &mut success) as i16 };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for Int16 {
    type InitType = i16;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<i16>::new(0x332dbcf3e3e3a961, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct UInt16(Object);

impl Deref for UInt16 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for UInt16 {
    const HASH: u64 = 0x7a05331174d1347b;
}

impl ObjectValuable for UInt16 {
    type ValueType = u16;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u16(***self, &mut success) };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for UInt16 {
    type InitType = u16;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<u16>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Int32(Object);

impl Deref for Int32 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for Int32 {
    const HASH: u64 = 0xe0342d8ff9932e02;
}

impl ObjectValuable for Int32 {
    type ValueType = i32;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u32(***self, &mut success) as i32 };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for Int32 {
    type InitType = i32;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<i32>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct UInt32(Object);

impl Deref for UInt32 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for UInt32 {
    const HASH: u64 = 0x33f722fd018171dc;
}

impl ObjectValuable for UInt32 {
    type ValueType = u32;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u32(***self, &mut success) };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for UInt32 {
    type InitType = u32;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<u32>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Int64(Object);

impl Deref for Int64 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for Int64 {
    const HASH: u64 = 0x4d177c41298e2277;
}

impl ObjectValuable for Int64 {
    type ValueType = i64;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u64(***self, &mut success) as i64 };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for Int64 {
    type InitType = i64;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<i64>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct UInt64(Object);

impl Deref for UInt64 {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for UInt64 {
    const HASH: u64 = 0x22b04bb57694ec47;
}

impl ObjectValuable for UInt64 {
    type ValueType = u64;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u64(***self, &mut success) };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for UInt64 {
    type InitType = u64;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<u64>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Bool(Object);

impl Deref for Bool {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for Bool {
    const HASH: u64 = 0xbc513183a12017df;
}

impl ObjectValuable for Bool {
    type ValueType = bool;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = unsafe { crate::interop::object_as_u8(***self, &mut success) != 0 };
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for Bool {
    type InitType = bool;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<bool>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Float(Object);

impl Deref for Float {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for Float {
    const HASH: u64 = 0x106efd909a7609a9;
}

impl ObjectValuable for Float {
    type ValueType = f32;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = f32::from_bits(unsafe { crate::interop::object_as_u32(***self, &mut success) });
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for Float {
    type InitType = f32;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<f32>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Double(Object);

impl Deref for Double {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for Double {
    const HASH: u64 = 0xdd6aef94114ecd43;
}

impl ObjectValuable for Double {
    type ValueType = f64;

    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let mut success = true;
        let result = f64::from_bits(unsafe { crate::interop::object_as_u64(***self, &mut success) });
        match success {
            true => Ok(result),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl ObjectInitializable for Double {
    type InitType = f64;

    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let handle = crate::interop::ObjectInitHandle::<f64>::new(Self::HASH, value);
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(Object(result)))
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct StringData {
    ptr: *const u8,
    len: usize
}

#[derive(Debug)]
pub struct String<'a> {
    handle: Object,
    lifetime: PhantomData<&'a Object>
}

impl<'a> Deref for String<'a> {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<'a> ObjectHash for String<'a> {
    const HASH: u64 = 0xd17d6432bd7c2cc9;
}

impl<'a> ObjectValuable for String<'a> {
    type ValueType = std::string::String;
    fn value(&self) -> Result<Self::ValueType, InteropError> {
        // Use existing CSharpString struct from mod_loader_data for a unmanaged wrapper over wchar_t* string
        let mut success = true;
        let string_raw = unsafe { crate::interop::object_as_string(***self, &mut success) };
        match success {
            true => Ok(CSharpString::new(string_raw as _).into()),
            false => Err(InteropError::CouldNotCastToValue),
        }
    }
}

impl<'a> ObjectInitializable for String<'a> {
    type InitType = &'a str;
    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let mut handle = crate::interop::ObjectInitHandle::<MaybeUninit<StringData>>::new(
            Self::HASH, MaybeUninit::uninit()
        );
        let str_data = unsafe { handle.get_mut().assume_init_mut() };
        str_data.ptr = value.as_ptr();
        str_data.len = value.len();
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self {
                handle: Object(result),
                lifetime: PhantomData::<&'a Object>
            })
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self {
            handle: value,
            lifetime: PhantomData::<&'a Object>
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ArrayData {
    value_type: u64,
    len: usize
}

#[derive(Debug)]
pub struct Array<'a, T> {
    handle: Object,
    array_type: PhantomData<&'a T>
}

impl<'a, T> ObjectHash for Array<'a, T> {
    const HASH: u64 = 0xeeb00fffe2d5ca32;
}

impl<'a, T> Deref for Array<'a, T> {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<'a, T> ObjectValuable for Array<'a, T>
where T: ObjectInitializable + ObjectValuable + ObjectHash
{
    type ValueType = Vec<T::ValueType>;
    fn value(&self) -> Result<Self::ValueType, InteropError> {
        let length = self.get_length()? as usize;
        let mut out = Vec::with_capacity(length);
        for i in 0..length {
            // public object? System.Array.GetValue(int index);
            let index = Int32::new(i as _)?;
            unsafe { crate::interop::push_parameter(Self::HASH, 0x40fb3729a671e7f4, **index) };
            unsafe { out.push(T::new_unchecked(Object::new_unchecked(
                crate::interop::call_function(Self::HASH, 0x40fb3729a671e7f4, ***self))).value()?) };
        }
        Ok(out)
    }
}

impl<'a, T> ObjectInitializable for Array<'a, T>
where T: ObjectInitializable + ObjectHash {
    type InitType = &'a [T];
    fn new(value: Self::InitType) -> Result<Self, InteropError> {
        let mut handle = crate::interop::ObjectInitHandle::<MaybeUninit<ArrayData>>::new(
            Self::HASH, MaybeUninit::uninit()
        );
        let arr_data = unsafe { handle.get_mut().assume_init_mut() };
        arr_data.value_type = T::HASH;
        arr_data.len = value.len();
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self {
                handle: Object(result),
                array_type: PhantomData::<&'a T>
            })
        }
    }

    unsafe fn new_unchecked(value: Object) -> Self {
        Self {
            handle: value,
            array_type: PhantomData::<&'a T>
        }
    }
}

impl<'a, T> Array<'a, T> {
    pub fn get_length(&self) -> Result<<Int32 as ObjectValuable>::ValueType, InteropError> {
        unsafe { Int32::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x45eb5d69f4b05f30, ***self))).value() }
    }
}