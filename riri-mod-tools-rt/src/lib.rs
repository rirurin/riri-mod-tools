//! Some Cool Reloaded Library
//! Here's the crate documentation
#[cfg(target_os = "windows")]
#[path = "address_win.rs"]
pub mod address;
pub mod assembly_utils;
pub mod logger;
pub mod sigscan_resolver;
