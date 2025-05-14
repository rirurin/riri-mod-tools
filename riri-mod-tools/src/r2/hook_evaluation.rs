#![allow(dead_code, unused_variables)]
use crate::{
    r2::hook_assignment::{
        HookAssignCodegen,
        HookBootstrapFunctionState,
        HookBootstrapStaticState,
        HookAssignCodegenStaticOffset,
        HookAssignCodegenDynamicOffset,
        HookAssignCodegenDynamicOffsetSharedScans,
        HookAssignCodegenUserDefined,
        InitFunction,
        SourceFileEvaluationResult,
        SourceFileEvaluationParamMapEx,
    },
    reloaded_codegen::{ HookEvaluator, ReloadedHookClass },
    utils
};
use handlebars::Handlebars;
use riri_mod_tools_impl::{
    csharp::Utils,
    hook_codegen::Reloaded2CSharpHook,
    hook_parse::{
        AssemblyFunctionHook,
        AssemblyFunctionHookData,
        HookConditional, 
        HookEntry 
    },
    riri_hook::{
        SourceFileEvaluationType,
        SourceFileInitializeState
    }
};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{ Display, Formatter },
    fs::{ self, Metadata }, 
    io::Write, 
    path::{ Path, PathBuf }, 
    time::SystemTime
};
use twox_hash::XxHash3_64;
use walkdir::{ DirEntry, WalkDir };

#[derive(Debug)]
pub struct MacroParseError(String);
impl Error for MacroParseError { }
impl Display for MacroParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing macros during codegen: {}", &self.0)
    }
}

pub struct HookSourceFile {
    path: PathBuf,
    meta: Metadata,
    hash: u64
}

impl HookSourceFile {
    fn new<T: AsRef<Path>>(d: DirEntry, src: T) -> Self {
        Self {
            path: d.path().to_owned(),
            meta: d.metadata().unwrap(),
            hash: XxHash3_64::oneshot(d.path()
                .strip_prefix(src.as_ref()).unwrap()
                .to_str().unwrap().as_bytes())
        }
    }
    /*
    fn file_is_modified(&self, ot: &Option<SystemTime>) -> bool {
        if let Some(t) = ot {
            if self.meta.modified().unwrap() > *t { 
                true 
            } else { 
                false 
            }
        } else { true }
    }
    */
    // TEMP: Rebuild all
    fn file_is_modified(&self, _ot: &Option<SystemTime>) -> bool { true }
}

#[derive(Debug)]
pub struct HookEvaluationResult {
    register_hook_functions: Vec<String>,
    loader_initialized_functions: Vec<String>
}
impl HookEvaluationResult {
    pub fn new(register_hook_functions: Vec<String>, loader_initialized_functions: Vec<String>) -> Self {
        Self { register_hook_functions, loader_initialized_functions }
    }
    pub fn get_register_hook_functions(&self) -> &[String] {
        self.register_hook_functions.as_slice()
    }
    pub fn get_loader_initialized_functions(&self) -> &[String] {
        self.loader_initialized_functions.as_slice()
    }
}

impl<'a, P> HookEvaluator<'a, P> 
where P: AsRef<Path>
{ 

    pub fn set_ignore_files(&mut self, ignores: Vec<PathBuf>) {
        for ignore in ignores { self.ignore_files.insert(ignore); }
    }
    
    pub fn ffi_namespace(&self) -> String {
        format!("{}.ReloadedFFI", self.package.get_mod_id())
    }
    pub fn ffi_hook_namespace(&self) -> String {
        format!("{}.ReloadedFFI.Hooks", self.package.get_mod_id())
    }
    pub fn ffi_interface_namespace(&self, i: &str) -> String {
        format!("{}.ReloadedFFI.Interfaces.{}", self.package.get_mod_id(), i)
    }
    pub fn ffi_utility_class(&self) -> String {
        format!("{}.ReloadedFFI.Utilities", self.package.get_mod_id())
    }

    fn get_csharp_hook_path(&self, hash: u64) -> PathBuf {
        self.riri_hook_dir.join(&format!("{:X}.g.cs", hash))
    }
    /*
    fn get_csharp_bootstrap_path(&self, hash: u64) -> PathBuf {
        self.riri_hook_bootstrap.join(&format!("{:X}.g.cs", hash))
    }
    */
    fn get_csharp_interfaces_path(&self, hash: u64) -> PathBuf {
        self.r2_interface_dir.join(&format!("{:X}.g.cs", hash))
    }

    pub fn get_middata_path(&self) -> &Path { &self.middata }

    fn should_ignore(&self, d: &DirEntry) -> bool {
        self.ignore_files.contains(d.path())
    }

    pub fn get_output_dll_name(&self) -> Result<String, Box<dyn Error>> {
        let cargo_name = self.cargo.get_package_string_required("name")?;
        Ok(cargo_name.replace("-", "_"))
    } 

    pub fn add_delegate_type_to_registry(&mut self, name: &str, ty: &str) {
        self.delegate_fnptr.insert(name.to_owned(), ty.to_owned());
    }

    fn check_rust_function_for_attribute(func: &syn::ItemFn, ident: &str, def: &mut bool) -> Result<Option<usize>, MacroParseError> {
        match func.attrs.iter().position(|f| f.path().is_ident(ident)) {
            Some(p) => {
                if *def { return Err(MacroParseError("riri-mod-tools function attributes are mutually exclusive. Only define one per function".to_owned())) }
                *def = true;
                Ok(Some(p))
            }, None => Ok(None)
        }
    }

    pub fn evaluate_rust_file<T: AsRef<Path>>(path: T) -> Result<SourceFileEvaluationResult, Box<dyn Error>> {
        let src_str = fs::read_to_string(path.as_ref())?;
        let mut src_syntax = syn::parse_file(&src_str)?;
        let mut mutated_items: Vec<(usize, riri_mod_tools_impl::riri_hook::HookBuildScriptResult)> = vec![];
        for (i, src_item) in src_syntax.items.iter_mut().enumerate() {
            // evaluate #[riri_hook], make FFI bindings into ReloadedFFI.Hooks.[xxhash64]
            match src_item {
                syn::Item::Fn(f) => {
                    let mut fn_attr_defined = false;
                    let fn_attr_pos = Self::check_rust_function_for_attribute(f, "riri_hook_fn", &mut fn_attr_defined)?;
                    let fn_inline_attr_pos = Self::check_rust_function_for_attribute(f, "riri_hook_inline_fn", &mut fn_attr_defined)?;
                    let fn_init_pos = Self::check_rust_function_for_attribute(f, "riri_init_fn", &mut fn_attr_defined)?;
                    let fn_mods_loaded_pos = Self::check_rust_function_for_attribute(f, "riri_mods_loaded_fn", &mut fn_attr_defined)?;
                    if let Some(p) = fn_attr_pos {
                        let insertion = riri_mod_tools_impl::riri_hook::riri_hook_fn_build(
                            f.attrs.remove(p).meta.require_list()?.tokens.clone(),
                            f.clone()
                        )?;
                        mutated_items.push((i, insertion));
                    }
                    if let Some(p) = fn_inline_attr_pos {
                        let insertion = riri_mod_tools_impl::riri_hook::riri_hook_inline_fn_build(
                            f.attrs.remove(p).meta.require_list()?.tokens.clone(),
                            f.clone()
                        )?;
                        mutated_items.push((i, insertion));
                    }
                    if let Some(p) = fn_init_pos {
                        let insertion = riri_mod_tools_impl::riri_init::riri_init_fn_build(
                            f.clone()
                        )?;
                        mutated_items.push((i, insertion));
                    }
                    if let Some(p) = fn_mods_loaded_pos {
                        let insertion = riri_mod_tools_impl::riri_init::riri_mods_loaded_fn_build(
                            f.clone()
                        )?;
                        mutated_items.push((i, insertion));
                    }
                },
                syn::Item::Macro(m) => {
                    let fn_attr_pos = m.attrs.iter().position(|f| f.path().is_ident("riri_hook_static"));
                    if let Some(p) = fn_attr_pos {
                        let insertion = riri_mod_tools_impl::riri_hook::riri_hook_static_build(
                            m.attrs.remove(p).meta.require_list()?.tokens.clone(),
                            m.clone()
                        )?;
                        mutated_items.push((i, insertion));
                    }
                },
                _ => continue
            }
        }
        let mut items: SourceFileEvaluationParamMapEx = HashMap::new();
        // Attach mutated items
        let mut added_items = 0;
        for mutated_item in &mut mutated_items {
            src_syntax.items[mutated_item.0 + added_items] = mutated_item.1.items.remove(0);
            if mutated_item.1.items.len() > 0 { 
                src_syntax.items.extend_from_slice(mutated_item.1.items.as_slice());
                src_syntax.items[mutated_item.0 + added_items + 1..].rotate_right(mutated_item.1.items.len());
                added_items += mutated_item.1.items.len();
            }
        }
        /* 
        for src_item in &src_syntax.items {
            match src_item {
                syn::Item::Fn(f) => println!("{}", f().to_string()),
                syn::Item::Static(f) => println!("{}", f.to_token_stream().to_string()),
                _ => continue
            };
        }
        */
        // Move name and args to evaluation result
        for mutated_item in mutated_items {
            items.insert(mutated_item.1.name, mutated_item.1.args);
        }
        // Ok(src_syntax)
        Ok(SourceFileEvaluationResult::new(src_syntax, items))
    } 

    fn generate_hook_entry_block_function(
        &mut self, 
        ffi: &ReloadedHookClass, 
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str,
        entry: &HookEntry) 
        -> Result<String, Box<dyn Error>> {
        Ok(match entry {
            HookEntry::Static(s) => {
                let res = HookAssignCodegenStaticOffset::new(*s);
                res.make_single_function_hook_assign(self, ffi, &class_data, &delegate_type)?
            },
            HookEntry::Dyn(d) => {
                match d.shared_scan {
                    Some(_) => {
                        self.uses_shared_scans = true;
                        let res = HookAssignCodegenDynamicOffsetSharedScans::new(d);
                        res.make_single_function_hook_assign(self, ffi, &class_data, &delegate_type)?
                    },
                    None => {
                        let res = HookAssignCodegenDynamicOffset::new(d);
                        res.make_single_function_hook_assign(self, ffi, &class_data, &delegate_type)?
                    }
                }
            },
            HookEntry::Delayed => {
                let res = HookAssignCodegenUserDefined::new();
                res.make_single_function_hook_assign(self, ffi, &class_data, &delegate_type)?
            }
        })
    }

    fn generate_hook_entry_block_static(
        &mut self, 
        ffi: &ReloadedHookClass,
        static_builder: &HookBootstrapStaticState,
        entry: &HookEntry) 
        -> Result<String, Box<dyn Error>> {
        Ok(match entry {
            HookEntry::Static(s) => {
                let res = HookAssignCodegenStaticOffset::new(*s);
                res.make_single_static_hook_assign(self, ffi, static_builder)?
            },
            HookEntry::Dyn(d) => {
                match d.shared_scan {
                    Some(_) => {
                        self.uses_shared_scans = true;
                        let res = HookAssignCodegenDynamicOffsetSharedScans::new(d);
                        res.make_single_static_hook_assign(self, ffi, static_builder)?
                    },
                    None => {
                        let res = HookAssignCodegenDynamicOffset::new(d);
                        res.make_single_static_hook_assign(self, ffi, static_builder)?
                    }
                }
            },
            HookEntry::Delayed => {
                let res = HookAssignCodegenUserDefined::new();
                res.make_single_static_hook_assign(self, ffi, static_builder)?
            }
        })
    }

    fn generate_conditional_first_string(&self, str: &str) -> String {
        let get_hash_function = format!("{}.get_executable_hash()", self.ffi_utility_class());
        let mut out = String::new();
        out.push_str(&format!("if ({} == {}.Mod.{}) ", &get_hash_function, self.package.get_mod_id(), str));
        out.push_str("\x7b\n            ");
        out
    }
    fn generate_conditional_first_int(&self, val: u64) -> String {
        let get_hash_function = format!("{}.get_executable_hash()", self.ffi_utility_class());
        let mut out = String::new();
        out.push_str(&format!("if ({} == {}) ", &get_hash_function, val));
        out.push_str("\x7b\n            ");
        out
    }
    fn generate_conditional_string(&self, str: &str) -> String {
        let get_hash_function = format!("{}.get_executable_hash()", self.ffi_utility_class());
        let mut out = String::new();
        out.push_str("\x7d");
        out.push_str(&format!("           else if ({} == {}.Mod.{}) ", &get_hash_function, self.package.get_mod_id(), str));
        out.push_str("\x7b\n            ");
        out
    }
    fn generate_conditional_int(&self, val: u64) -> String {
        let get_hash_function = format!("{}.get_executable_hash()", self.ffi_utility_class());
        let mut out = String::new();
        out.push_str("\x7d");
        out.push_str(&format!("           else if ({} == {}) ", &get_hash_function, val));
        out.push_str("\x7b\n            ");
        out
    }
    fn generate_conditional_default(&self) -> String {
        let mut out = String::new();
        out.push_str("\x7d");
        out.push_str("           else ");
        out.push_str("\x7b\n            ");
        out
    }

    fn generate_hook_c_function_for_function(
        &mut self,
        hook_parm: &riri_mod_tools_impl::riri_hook::HookInfo,
        ffi: &ReloadedHookClass,
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str
    ) -> Result<String, Box<dyn Error>> {
        let mut hook_assign = String::new();
        if hook_parm.0.len() == 1 {
            hook_assign.push_str(&self.generate_hook_entry_block_function(
                ffi, &class_data, &delegate_type, &hook_parm.0[0].1)?);
        } else {
            let mut add_ending_brace = true;
            for (i, hook_entry) in hook_parm.0.iter().enumerate() {
                match &hook_entry.0 {
                    riri_mod_tools_impl::hook_parse::HookConditional::None 
                    => panic!("None is not supported in multi hooks"),
                    riri_mod_tools_impl::hook_parse::HookConditional::HashNamed(s)
                    => {
                        hook_assign.push_str(&if i == 0 { 
                            self.generate_conditional_first_string(s)
                        } else { 
                            self.generate_conditional_string(s) 
                        });
                    },
                    riri_mod_tools_impl::hook_parse::HookConditional::HashNum(v)
                    => {
                        hook_assign.push_str(&if i == 0 { 
                            self.generate_conditional_first_int(*v) 
                        } else { 
                            self.generate_conditional_int(*v) 
                        });
                    },
                    riri_mod_tools_impl::hook_parse::HookConditional::Default
                    => {
                        if i > 0 { 
                            hook_assign.push_str(&self.generate_conditional_default());
                        } else { 
                            add_ending_brace = false 
                        }
                    },
                }
                hook_assign.push_str(&self.generate_hook_entry_block_function(
                    ffi, &class_data, &delegate_type, &hook_entry.1)?);
            }
            if add_ending_brace {
                hook_assign.push_str("\x7d\n");
            }
        }
        hook_assign.push_str("            ");
        Ok(hook_assign)
    }

    fn generate_hook_entry_block_inline_function(
        &mut self, 
        ffi: &ReloadedHookClass, 
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str,
        hook: &HookEntry,
        assemble: &AssemblyFunctionHookData,
        cond: &HookConditional)
        -> Result<String, Box<dyn Error>> {
        Ok(match hook {
            HookEntry::Static(s) => {
                let res = HookAssignCodegenStaticOffset::new(*s);
                res.make_function_hook_assign_assembly(self, ffi, &class_data, &delegate_type, &assemble, cond)?
            },
            HookEntry::Dyn(d) => {
                match d.shared_scan {
                    Some(_) => {
                        self.uses_shared_scans = true;
                        let res = HookAssignCodegenDynamicOffsetSharedScans::new(d);
                        res.make_function_hook_assign_assembly(self, ffi, &class_data, &delegate_type, &assemble, cond)?
                    },
                    None => {
                        let res = HookAssignCodegenDynamicOffset::new(d);
                        res.make_function_hook_assign_assembly(self, ffi, &class_data, &delegate_type, &assemble, cond)?
                    }
                }
            },
            HookEntry::Delayed => {
                let res = HookAssignCodegenUserDefined::new();
                res.make_function_hook_assign_assembly(self, ffi, &class_data, &delegate_type, &assemble, cond)?
            }
        })
    }

    fn generate_hook_assembly_function(
        &mut self,
        hook_parm: &AssemblyFunctionHook,
        ffi: &ReloadedHookClass,
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str
    ) -> Result<String, Box<dyn Error>> {
        let mut hook_assign = String::new();
        if hook_parm.hook_info.0.len() == 1 {
            hook_assign.push_str(&self.generate_hook_entry_block_inline_function(
                ffi, &class_data, &delegate_type, 
                &hook_parm.hook_info.0[0].1,
            &hook_parm.data[0],
                &hook_parm.hook_info.0[0].0
            )?);
        } else {
            let mut add_ending_brace = true;
            for (i, (hook, asm)) in hook_parm.hook_info.0.iter().zip(hook_parm.data.iter()).enumerate() {
                match &hook.0 {
                    HookConditional::None  => panic!("None is not supported in multi hooks"),
                    HookConditional::HashNamed(s) => {
                        hook_assign.push_str(&if i == 0 { 
                            self.generate_conditional_first_string(s)
                        } else { 
                            self.generate_conditional_string(s) 
                        });
                    },
                    HookConditional::HashNum(v) => {
                        hook_assign.push_str(&if i == 0 { 
                            self.generate_conditional_first_int(*v) 
                        } else { 
                            self.generate_conditional_int(*v) 
                        });
                    },
                    HookConditional::Default => {
                        if i > 0 { 
                            hook_assign.push_str(&self.generate_conditional_default());
                        } else { 
                            add_ending_brace = false 
                        }
                    },
                } 
                hook_assign.push_str(&self.generate_hook_entry_block_inline_function(
                    ffi, &class_data, &delegate_type, &hook.1, asm, &hook.0)?);
            }
            if add_ending_brace {
                hook_assign.push_str("\x7d\n");
            }
        }
        hook_assign.push_str("            ");
        Ok(hook_assign)
    }

    pub fn generate_init_function_bootstrap(
        &self,
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str,
    ) -> String {
        let mut user_method = String::new();
        user_method.push_str("[UnmanagedCallersOnly(CallConvs = [ typeof(System.Runtime.CompilerServices.CallConvStdcall) ])]\n");
        user_method.push_str(&format!(
            "\t\tpublic static unsafe void UserDefined_{}(nuint addr)\n", class_data.get_fn_name()));
        user_method.push_str("\t\t\x7b\n");
        user_method.push_str(&format!(
            "\t\t\t_instance!._{} = _hooks!.CreateHook<{}>({}, (long)addr).Activate();\n",
            class_data.get_fn_name(), class_data.get_delegate_path(), class_data.get_fn_path()));
        user_method.push_str(&format!("\t\t\t{}.{}(({})_instance!._{}.OriginalFunctionWrapperAddress);\n",
          class_data.get_class_path(), 
          &Reloaded2CSharpHook::make_hook_set_string(&class_data.get_fn_name().to_ascii_uppercase()),
          delegate_type,
          class_data.get_fn_name()
        ));
        user_method.push_str("\t\t\x7d\n");
        user_method
    }

    // - Add an IHook<DelegateType> for the target C# hook
    // - Use the SigScan function to set the value of that C# hook, then get the
    // OriginalFunctionWrapperAddress to pass into the Rust hook set function. The Sigscan will
    // need a reference to Rust's hooked function and the original function pointer set.
    // - Delegate and function are already defined in ReloadedFFI.Hooks
    pub fn generate_hook_bootstrap(&mut self, ffi: &ReloadedHookClass) -> Result<String, Box<dyn Error>> {
        let mut hook_decl = String::new();
        let mut hook_assign = String::new();
        let mut hook_methods = String::new();
        let mut loader_init_call = String::new();

        for item in &ffi.eval.file.items {
            match item {
                syn::Item::Fn(f) => {
                    let fn_name = f.sig.ident.to_string();
                    // println!("Found function: {}", &fn_name);
                    let hook_parm = match ffi.eval.params.get(&fn_name) {
                        Some(v) => v,
                        None => continue
                    };
                    let delegate_path = format!("{}.{}.{}Delegate", &self.ffi_hook_namespace(), &ffi.csharp_class_name(), &fn_name);
                    // Create a delegate type to cast the function pointer
                    let mut delegate_type = "delegate* unmanaged[Stdcall]<".to_owned();
                    for input in &f.sig.inputs {
                        if let syn::FnArg::Typed(t) = input {
                            delegate_type.push_str(&format!("{}, ", Utils::to_csharp_typename(&t.ty)?));
                        }
                    }
                    match &f.sig.output {
                        syn::ReturnType::Default => delegate_type.push_str("void"),
                        syn::ReturnType::Type(_, t) => delegate_type.push_str(&Utils::to_csharp_typename(&t)?)
                    };
                    delegate_type.push_str(">");
                    let class_data = HookBootstrapFunctionState::new(fn_name, delegate_path);
                    match &hook_parm {
                        SourceFileEvaluationType::CFunction(hook_parm) => {
                            hook_decl.push_str(&format!("private Reloaded.Hooks.Definitions.IHook<{}>? _{};\n", 
                                class_data.get_delegate_path(), class_data.get_fn_name()));
                            hook_assign.push_str(&self.generate_hook_c_function_for_function(
                                hook_parm, ffi, &class_data, &delegate_type)?);
                            for (_, en) in &hook_parm.0 {
                                match en {
                                    HookEntry::Delayed => hook_methods.push_str(&self.generate_init_function_bootstrap(&class_data, &delegate_type)),
                                    _ => ()
                                }
                            }
                        },
                        SourceFileEvaluationType::Inline(hook_parm) => {
                            hook_decl.push_str(&format!("private Reloaded.Hooks.Definitions.IAsmHook? _{}_ASM;\n", 
                                class_data.get_fn_name()));
                            for en in &hook_parm.hook_info.0 {
                                hook_decl.push_str(&format!("private Reloaded.Hooks.Definitions.IReverseWrapper<{}>? _{}_WRAP{};\n", 
                                    class_data.get_delegate_path(), class_data.get_fn_name(), en.0));
                            }
                            hook_assign.push_str(&self.generate_hook_assembly_function(
                                hook_parm, ffi, &class_data, &delegate_type)?);
                        },
                        SourceFileEvaluationType::InitFunction(hook_parm) => {
                            match hook_parm.get_state() {
                                SourceFileInitializeState::ModuleLoaded => 
                                    hook_assign.push_str(&InitFunction::make_init_function_call::<P>(self, ffi, &class_data, &delegate_type)?),
                                SourceFileInitializeState::ModLoaderInitialized => 
                                    loader_init_call.push_str(&InitFunction::make_init_function_call::<P>(self, ffi, &class_data, &delegate_type)?),
                                SourceFileInitializeState::ModLoaded => ()
                            }
                        }
                    }
                },
                syn::Item::Static(s) => {
                    let static_name = s.ident.to_string();
                    let hook_parm = match ffi.eval.params.get(&static_name) {
                        Some(v) => v,
                        None => continue
                    };
                    let inner_type = match crate::utils::generic_type_get_inner(&s.ty)? {
                        Some(t) => t,
                        None => return Err(Box::new(MacroParseError("No generic argument was found".to_owned())))
                    };
                    let static_builder = HookBootstrapStaticState::new(
                        riri_mod_tools_impl::csharp::Utils::to_csharp_typename(&inner_type)?, 
                        static_name
                    );
                    if let riri_mod_tools_impl::riri_hook::SourceFileEvaluationType::CFunction(hook_parm) = &hook_parm {
                        if hook_parm.0.len() == 1 {
                            hook_assign.push_str(&self.generate_hook_entry_block_static(
                                ffi, &static_builder, &hook_parm.0[0].1)?);
                        } else {
                            let mut add_ending_brace = true;
                            for (i, hook_entry) in hook_parm.0.iter().enumerate() {
                                match &hook_entry.0 {
                                    HookConditional::None => panic!("None is not supported in multi hooks"),
                                    HookConditional::HashNamed(s) => {
                                        hook_assign.push_str(&if i == 0 { 
                                            self.generate_conditional_first_string(s)
                                        } else { 
                                            self.generate_conditional_string(s) 
                                        });
                                    },
                                    HookConditional::HashNum(v) => {
                                        hook_assign.push_str(&if i == 0 { 
                                            self.generate_conditional_first_int(*v) 
                                        } else { 
                                            self.generate_conditional_int(*v) 
                                        });
                                    },
                                    HookConditional::Default => {
                                        if i > 0 { 
                                            hook_assign.push_str(&self.generate_conditional_default());
                                        } else { 
                                            add_ending_brace = false 
                                        }
                                    },
                                }
                                hook_assign.push_str(&self.generate_hook_entry_block_static(
                                    ffi, &static_builder, &hook_entry.1)?);
                            }
                            if add_ending_brace {
                                hook_assign.push_str("\x7d\n");
                            }
                        }
                        hook_assign.push_str("            ");
                    } else {
                        return Err(Box::new(MacroParseError("Inline hook type is not supported for statics".to_owned())))
                    }
                },
                _ => continue
            };
        }

        let mut out = crate::utils::SourceWriter::new();
        out.writeln("// These hook definitions were automatically generated.");
        out.fmtln(format_args!("// DO NOT EDIT THIS. It will get overwritten if you rebuild {}!", self.package.Name))?;
        out.writeln("#nullable enable");
        out.fmtln(format_args!("namespace {}", self.package.get_mod_id()))?;
        out.indent()?;
        out.writeln("public unsafe partial class Mod\n");
        out.indent()?;
        // class definitions for C-style and assmebly hook storage
        out.writeln(&hook_decl);

        // called when the module is initialized
        out.fmtln(format_args!("public void {}()\n", ffi.csharp_register_hooks()))?;
        out.indent()?;
        out.writeln(&hook_assign);
        out.unindent()?;
        out.writeln(&hook_methods);
        // called when mod loader is finished initializing (all mods are loaded) 
        out.fmtln(format_args!("public void {}()\n", ffi.csharp_mod_loader_init()))?;
        out.indent()?;
        out.writeln(&loader_init_call);
        for _ in 0..3 { out.unindent()?; }
        Ok(out.submit())
    }

    fn cshookgen_get_hash(file_name: &Path) -> u64 {
        let stem = file_name.to_str().unwrap().split_once(".").unwrap().0;
        u64::from_str_radix(stem, 16).unwrap()
    }

    pub fn evaluate_hooks(
        &mut self, cb: fn(&mut Self, ReloadedHookClass) -> Result<(), Box<dyn Error>>) 
        -> Result<HookEvaluationResult, Box<dyn Error>> {
        // look for timestamp if it exists
        let timestamp_file = self.middata.join("timestamp");
        let last_build_time = match fs::metadata(&timestamp_file) {
            Ok(v) => Some(v.modified().unwrap()),
            Err(_) => None
        };
        for cs_file in WalkDir::new(&self.riri_hook_dir).into_iter()
            .filter(|f| f.is_ok() && utils::is_csharp_source(f.as_ref().unwrap())) {
            if let Ok(f) = cs_file {
                let name: &Path = f.file_name().as_ref();
                let tgt_hash = Self::cshookgen_get_hash(name);
                if !self.csharp_files.insert(tgt_hash) {
                    return Err(Box::new(MacroParseError(
                        format!("File hash collision: {} made duplicate hash {:X}, please rename this!"
                        , name.to_str().unwrap(), tgt_hash
                    ))))
                }
            }
        }
        let rust_src = self.base_path.as_ref().join("src");
        for src_file in WalkDir::new(&rust_src).into_iter()
            .filter(|f| f.is_ok() && utils::is_rust_source(f.as_ref().unwrap())) {
            if let Ok(f) = src_file {
                if self.should_ignore(&f) { continue; }
                let new_rs = HookSourceFile::new(f, &rust_src);
                // doesn't matter if it exists or is new, we're overwriting it anyway
                self.csharp_files.remove(&new_rs.hash);
                if new_rs.file_is_modified(&last_build_time) {
                    self.rust_files.push(new_rs);
                }
            }
        }
        // delete C# files that have no rust source association
        for cs_orphan in &self.csharp_files {
            fs::remove_file(self.get_csharp_hook_path(*cs_orphan))?;
            // fs::remove_file(self.get_csharp_bootstrap_path(*cs_orphan))?;
        }
        // Generate Rust/C#: 
        let mut evaluated_files: Vec<ReloadedHookClass> = vec![];
        for src in &self.rust_files {
            let hash = src.hash;
            evaluated_files.push(ReloadedHookClass {
                eval: Self::evaluate_rust_file(&src.path)?,
                cs_path: self.get_csharp_hook_path(hash),
                hash
            });
        }
        // add code to declare and register R2 hooks
        let mut evaluated_files_full: Vec<(String, ReloadedHookClass)> = vec![];
        for evaluated_file in evaluated_files { 
            evaluated_files_full.push((self.generate_hook_bootstrap(&evaluated_file)?, evaluated_file));
        }
        // call csbindgen to generate function imports and structs
        let mut call_hook_registers = vec![];
        let mut loader_initialized_files = vec![];
        for evaluated_file in evaluated_files_full {
            call_hook_registers.push(evaluated_file.1.csharp_register_hooks());
            loader_initialized_files.push(evaluated_file.1.csharp_mod_loader_init());
            let cs_file = evaluated_file.1.cs_path.to_str().unwrap().to_owned();
            cb(self, evaluated_file.1)?;
            let mut cs_file = fs::OpenOptions::new().append(true).open(cs_file)?;
            cs_file.write(evaluated_file.0.as_bytes())?;
        }
        Ok(HookEvaluationResult::new(call_hook_registers, loader_initialized_files))
    }

    pub fn generate_mod_main(&self, evaluation: HookEvaluationResult) -> Result<(), Box<dyn Error>> {
        pub fn into_toml_array<'a, T>(arr: T) -> toml::value::Array 
        where T: Into<&'a [String]> {
            arr.into().iter().map(|s| toml::Value::String(s.to_string())).collect()
        }

        let mut mod_file = fs::File::create(self.middata.join("Mod.g.cs"))?;
        let mut utils_file = fs::File::create(self.middata.join("Utils.g.cs"))?;
        let logger_prefix = match &self.package.LoggerPrefix {
            Some(v) => v.to_owned(),
            None => self.package.get_mod_id().to_owned()
        };
        let logger_color = match &self.package.LoggerColor {
            Some(v) => v.to_owned(),
            None => format!("LimeGreen")
        };
        let mut hbs = Handlebars::new();
        hbs.register_template_string("main", crate::hbs::mod_main::FILE)?;
        hbs.register_template_string("utils", crate::hbs::ffi_builtin::FILE)?;
        let mut data = toml::Table::new();
        data.insert("mod_id".to_owned(), toml::Value::String(self.package.get_mod_id().to_owned()));
        data.insert("mod_name".to_owned(), toml::Value::String(self.package.Name.to_owned()));
        data.insert("dll_name".to_owned(), toml::Value::String(self.get_output_dll_name()?));
        data.insert("logger_prefix".to_owned(), toml::Value::String(logger_prefix));
        data.insert("logger_color".to_owned(), toml::Value::String(logger_color));
        data.insert("uses_shared_scans".to_owned(), toml::Value::Boolean(self.uses_shared_scans));
        data.insert("utility_namespace".to_owned(), toml::Value::String(self.ffi_utility_class()));
        data.insert("ffi_namespace".to_owned(), toml::Value::String(self.ffi_namespace()));
        data.insert("exports_interfaces".to_owned(), toml::Value::Boolean(false));

        let register_hook_modules = toml::Value::Array(into_toml_array(evaluation.get_register_hook_functions()));
        let mod_initialized_modules = toml::Value::Array(into_toml_array(evaluation.get_loader_initialized_functions()));
        data.insert("register_hook_fn".to_owned(), register_hook_modules);
        data.insert("loader_init_fn".to_owned(), mod_initialized_modules);
        mod_file.write(hbs.render("main", &data)?.as_bytes())?;
        utils_file.write(hbs.render("utils", &data)?.as_bytes())?;
        Ok(())
    }

    pub fn evaluate_reloaded_interfaces(&mut self) {

    }
    pub fn update_timestamp(&self) -> Result<(), Box<dyn Error>> {
        let timestamp_file = self.middata.join("timestamp");
        fs::write(&timestamp_file, [0xff])?;
        Ok(())
    }

    pub fn copy_files_to_output<U: AsRef<Path>>(&self, target: U) -> Result<(), Box<dyn Error>> {
        for gen_file in WalkDir::new(&self.middata).into_iter().filter(|f| f.is_ok() 
            && f.as_ref().unwrap().file_name().to_str().unwrap() != "timestamp")  {
            if let Ok(f) = gen_file {
                println!("{}", f.path().to_str().unwrap());
            }
        }
        Ok(())
    }
}