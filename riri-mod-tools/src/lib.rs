//! :adachi_true:
pub mod config_codegen;
pub mod ensure_layout;
pub mod ensure_layout_tests;
pub(crate) mod hbs {
    pub(crate) mod ffi_builtin;
    pub(crate) mod mod_main;
}
pub mod mod_package;
pub mod platform;
pub mod reloaded_codegen;
pub mod riri_hook_tests;
pub mod utils;