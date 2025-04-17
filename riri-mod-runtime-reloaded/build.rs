use csbindgen;
use riri_mod_tools::{ config_codegen, mod_package, reloaded_codegen };
use walkdir::{ DirEntry, WalkDir };
use std::error::Error;
// generate bindgen DllImport
fn csbindgen_callback<P: AsRef<std::path::Path>>(this: &mut reloaded_codegen::HookEvaluator<P>, class: reloaded_codegen::ReloadedHookClass) -> Result<(), Box<dyn Error>> {
    let class_name = class.csharp_class_name();
    csbindgen::Builder::default()
        .input_extern(class.eval.file)
        .method_filter(|_| true)
        .csharp_dll_name(this.get_output_dll_name()?)
        .csharp_namespace(this.ffi_hook_namespace())
        .csharp_class_name(class_name)
        .csharp_make_extern_delegates(Some( |x| !x.starts_with("_")))
        .calling_convention_type_dllimport("StdCall".to_owned())
        .calling_convention_type_fnptr("Stdcall".to_owned())
        .generate_csharp_file(class.cs_path.to_str().unwrap())
}

// Copy from intermediate dir to C# project
fn copy_to_output<S, T>(src: S, tgt: T) -> Result<(), Box<dyn Error>> 
    where S: AsRef<std::path::Path>, T: AsRef<std::path::Path>
{
    let src_path_cut = src.as_ref().to_str().unwrap().len() + 1;
    let filter_entries = |entry: &DirEntry| {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap();
            if file_name == "timestamp" { return false; }
            let path_str = entry.path().to_str().unwrap();
            path_str.len() > src_path_cut
        } else {
            false
        }
    };
    for gen_file in WalkDir::new(src.as_ref()).into_iter().filter(|f| f.is_ok() 
        // && f.as_ref().unwrap().file_name().to_str().unwrap() != "timestamp")  {
        && filter_entries(f.as_ref().unwrap())) {
        if let Ok(f) = gen_file {
            let path_out = tgt.as_ref().join(&f.path().to_str().unwrap()[src_path_cut..]);
            println!("Copy file to {:?}", path_out);
            std::fs::copy(&f.path(), &path_out)?;
        }
    }
    Ok(())
}

fn main() {
    // Read package.toml and Cargo.toml
    let base = std::env::current_dir().unwrap();
    let cargo_info = mod_package::CargoInfo::new(&base).unwrap();
    let package_toml = mod_package::reloaded3ririext::Package::new(&base, &cargo_info).unwrap();
    let mut hash_e = mod_package::HashFile::new_builtin(&base, package_toml.get_mod_id(), package_toml.get_mod_name()).unwrap();
    // Make FFI: Evaluate riri_hook macro, create hooked classes
    let mut hook_e = reloaded_codegen::HookEvaluator::new(&base, &package_toml, &cargo_info).unwrap();
    hook_e.set_ignore_files(vec![base.join("src/logger.rs"), base.join("src/config.rs")]);
    let call_hook_register = hook_e.evaluate_hooks(csbindgen_callback).unwrap();
    // Generate Mod.cs
    hook_e.generate_mod_main(call_hook_register).unwrap();
    hash_e.generate_mod_hashes().unwrap();
    hook_e.update_timestamp().unwrap();
    let middata = hook_e.get_middata_path().to_path_buf();
    // Generate ModConfig.json
    // let mod_id = package_toml.get_mod_id().to_owned();
    let r2_modconfig: mod_package::reloaded2::Package = package_toml.try_into().unwrap();
    r2_modconfig.save(&base).unwrap(); 
    let output_path = base.parent().unwrap().join(r2_modconfig.get_mod_id());
    // Make FFI: Reloaded-II interfaces
    // (metaphor.multiplayer.ReloadedFFI.Interfaces.Ryo)
    // ...

    // Generate Config.cs
    config_codegen::generate(&base).unwrap(); 
    // Copy middata to C# project 
    copy_to_output(&middata, &output_path).unwrap();
}