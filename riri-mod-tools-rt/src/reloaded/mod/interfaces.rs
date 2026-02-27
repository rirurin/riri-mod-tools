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

    unsafe fn new_unchecked(value: Object) -> Self {
        Self(value)
    }
}

/// impl IModConfigV1
impl IModConfig {
    pub fn get_mod_id(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x83c1e998b284e252, ***self))).value() }
    }

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

    pub fn get_mod_description(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x4cd2fffff6099559, ***self))).value() }
    }

    pub fn get_mod_dll(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0xdc0f9b10f395a06a, ***self))).value() }
    }

    pub fn get_mod_icon(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x36613f5cfc110a99, ***self))).value() }
    }

    pub fn get_mod_dependencies(&self) -> Result<Vec<String>, crate::interop::InteropError> {
        unsafe { crate::system::Array::<'_, crate::system::String>::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x8f82cb5fd77c47c3, ***self))).value() }
    }

    pub fn get_supported_app_id(&self) -> Result<Vec<String>, crate::interop::InteropError> {
        unsafe { crate::system::Array::<'_, crate::system::String>::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x109e6c478ef466ec, ***self))).value() }
    }
}

/// impl IModConfigV2
impl IModConfig {
    pub fn get_optional_dependencies(&self) -> Result<Vec<String>, crate::interop::InteropError> {
        unsafe { crate::system::Array::<'_, crate::system::String>::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x6c89567cb1b12249, ***self))).value() }
    }
}

/// impl IModConfigV3
impl IModConfig {
    pub fn get_mod_native_dll_32(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0xe535ac1327080d6d, ***self))).value() }
    }

    pub fn get_mod_native_dll_64(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x48e477c0956b8344, ***self))).value() }
    }

    pub fn get_mod_r2r_managed_dll_32(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x40cb44205404b982, ***self))).value() }
    }

    pub fn get_mod_r2r_managed_dll_64(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x3879584a12914665, ***self))).value() }
    }

    pub fn is_r2r(&self, config_path: &crate::system::String) -> Result<bool, crate::interop::InteropError> {
        unsafe { crate::interop::push_parameter(Self::HASH, 0x4d99314b05830d34, ***config_path) };
        unsafe { crate::system::Bool::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x4d99314b05830d34, ***self))).value() }
    }

    pub fn get_dll_path(&self, config_path: &crate::system::String) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::interop::push_parameter(Self::HASH, 0x89d644a362dde743, ***config_path) };
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x89d644a362dde743, ***self))).value() }
    }

    pub fn is_native_mod(&self, config_path: &crate::system::String) -> Result<bool, crate::interop::InteropError> {
        unsafe { crate::interop::push_parameter(Self::HASH, 0xad636bcd6ad7721, ***config_path) };
        unsafe { crate::system::Bool::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0xad636bcd6ad7721, ***self))).value() }
    }

    pub fn get_managed_dll_path(&self, config_path: &crate::system::String) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::interop::push_parameter(Self::HASH, 0xc99a21c5e73611c7, ***config_path) };
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0xc99a21c5e73611c7, ***self))).value() }
    }

    pub fn get_native_dll_path(&self, config_path: &crate::system::String) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::interop::push_parameter(Self::HASH, 0x7f9d675d3319d6d5, ***config_path) };
        unsafe { crate::system::String::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x7f9d675d3319d6d5, ***self))).value() }
    }
}

/// impl IModConfigV4
impl IModConfig {
    pub fn get_is_library(&self) -> Result<bool, crate::interop::InteropError> {
        unsafe { crate::system::Bool::new_unchecked(Object::new_unchecked(
            crate::interop::call_function(Self::HASH, 0x783773fb8daff1ca, ***self))).value() }
    }
}

/// impl IModConfigV5
impl IModConfig {
    pub fn get_release_metadata_file_name(&self) -> Result<String, crate::interop::InteropError> {
        unsafe { crate::system::String::new_unchecked(
            Object::new_unchecked(crate::interop::call_function(Self::HASH, 0x50b83b0c8620672b, ***self))).value() }
    }

    pub fn get_is_universal_mod(&self) -> Result<bool, crate::interop::InteropError> {
        unsafe { crate::system::Bool::new_unchecked(
            Object::new_unchecked(crate::interop::call_function(Self::HASH, 0xffd11e37801daac8, ***self))).value() }
    }
}