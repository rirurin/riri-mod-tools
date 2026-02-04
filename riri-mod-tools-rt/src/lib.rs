//! Some Cool Reloaded Library
//! Here's the crate documentation
#[cfg(target_os = "windows")]
#[path = "address_win.rs"]
pub mod address;
#[cfg(target_os = "linux")]
#[path = "address_linux.rs"]
pub mod address;
pub mod assembly_utils;
pub mod interleave;
pub mod interop;
pub mod logger;
pub mod mod_loader_data;
pub mod protection;
#[cfg(feature = "reloaded")]
pub mod reloaded {
    pub mod r#mod {
        pub mod interfaces;
    }
}
pub mod sigscan_resolver;
pub mod system;
pub mod vtable;