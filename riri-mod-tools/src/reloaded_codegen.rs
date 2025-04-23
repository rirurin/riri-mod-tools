#![allow(dead_code, unused_variables)]
use crate::{
    mod_package::{ self, reloaded3ririext },
    r2::{
        hook_assignment::SourceFileEvaluationResult,
        hook_evaluation::HookSourceFile
    },
    utils
};
use std::{
    error::Error, 
    path::{ Path, PathBuf }, 
};

pub struct HookEvaluator<'a, P: AsRef<Path>> {
    pub package: &'a reloaded3ririext::Package,
    pub cargo: &'a mod_package::CargoInfo,
    pub(crate) csharp_files: std::collections::HashSet<u64>,
    pub(crate) rust_files: Vec<HookSourceFile>,
    pub(crate) delegate_fnptr: std::collections::HashMap<String, String>,
    // target paths
    pub(crate) base_path: P,
    pub(crate) middata: PathBuf,
    pub(crate) riri_hook_dir: PathBuf,
    // riri_hook_bootstrap: PathBuf,
    pub(crate) r2_interface_dir: PathBuf,
    // options
    pub(crate) ignore_files: std::collections::HashSet<PathBuf>,
    // Mod.g.cs storage
    pub(crate) mod_hook_declarations: String,
    pub(crate) mod_hook_set: String,
    pub(crate) uses_shared_scans: bool
}

impl<'a, P> HookEvaluator<'a, P>
where P: AsRef<Path>
{
    pub fn new(path: P, 
        package: &'a reloaded3ririext::Package,
        cargo: &'a mod_package::CargoInfo
        ) -> Result<Self, Box<dyn Error>> {
        let middata = utils::get_or_make_child_dir(path.as_ref(), "middata")?;
        let riri_hook_dir = utils::get_or_make_child_dir(&middata, "riri_hook")?;
        // let riri_hook_bootstrap = utils::get_or_make_child_dir(&middata, "riri_hook_bootstrap")?;
        let r2_interface_dir = utils::get_or_make_child_dir(&middata, "r2_interfaces")?;
        Ok(HookEvaluator {
            package, cargo,
            csharp_files: std::collections::HashSet::new(),
            rust_files: vec![],
            delegate_fnptr: std::collections::HashMap::new(),
            base_path: path, 
            middata, riri_hook_dir, r2_interface_dir,
            ignore_files: std::collections::HashSet::new(),
            mod_hook_declarations: String::new(),
            mod_hook_set: String::new(),
            uses_shared_scans: false
        })
    }
}

pub struct ReloadedHookClass {
    pub eval: SourceFileEvaluationResult,
    pub cs_path: PathBuf,
    pub hash: u64
}

impl ReloadedHookClass {
    pub fn csharp_class_name(&self) -> String {
        format!("Hooks_{:X}", self.hash)
    }
    pub fn csharp_register_hooks(&self) -> String {
        format!("RegisterHooks_{:X}", self.hash)
    }
    pub fn csharp_mod_loaded(&self) -> String {
        format!("ModLoaded_{:X}", self.hash)
    }
    pub fn csharp_mod_loader_init(&self) -> String {
        format!("ModLoaderInit_{:X}", self.hash)
    }
    pub fn csharp_class_name_static(hash: u64) -> String {
        format!("Hooks_{:X}", hash)
    }
    pub fn csharp_register_hooks_static(hash: u64) -> String {
        format!("RegisterHooks_{:X}", hash)
    }
    pub fn csharp_mod_loaded_static(hash: u64) -> String {
        format!("ModLoaded_{:X}", hash)
    }
    pub fn csharp_mod_loader_init_static(hash: u64) -> String {
        format!("ModLoaderInit_{:X}", hash)
    }
}