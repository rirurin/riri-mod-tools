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
#[cfg(feature = "reloaded")]
pub mod interop; // For C# interop (requires Reloaded runtime)
pub mod logger;
#[cfg(feature = "reloaded")]
pub mod mod_loader_data;
pub mod protection;
#[cfg(feature = "reloaded")]
pub mod reloaded {
    pub mod r#mod {
        pub mod interfaces; // namespace Reloaded.Mod.Interfaces;
    }
}
pub mod sigscan_resolver;
#[cfg(feature = "reloaded")]
pub mod system; // namespace System;
pub mod vtable;