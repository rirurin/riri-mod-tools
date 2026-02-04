use std::ops::Deref;
use crate::interop::{ ObjectHash, ObjectInitializable, ObjectValuable };
use crate::system::Object;

#[derive(Debug)]
pub struct IModConfig(Object);

impl Deref for IModConfig {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectHash for IModConfig {
    const HASH: u64 = 0xd2046ab0dcce183d;
}

impl ObjectInitializable for IModConfig {
    type InitType = ();
    fn new(_: Self::InitType) -> Result<Self, crate::interop::InteropError> {
        let handle = crate::interop::ObjectInitHandle::<()>::new(Self::HASH, ());
        let result = unsafe { crate::interop::get_object_instance(handle.raw()) };
        match result {
            0 => Err(crate::interop::InteropError::CouldNotMakeObjectInstance),
            _ => Ok(Self(unsafe { Object::new_unchecked(result) }))
        }
    }
}

impl IModConfig {
    pub fn get_mod_name(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x2cb0950ee30e6c9c, ***self))).value() }
    }

    pub fn get_mod_author(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x72f6e7b203a95ad8, ***self))).value() }
    }

    pub fn get_mod_version(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0xcfccce16ceb8e518, ***self))).value() }
    }

    pub unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}