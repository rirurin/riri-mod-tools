#![allow(dead_code, unused_variables)]
use crate::{
    mod_package::{ self, reloaded3ririext },
    utils
};
use handlebars::Handlebars;
use std::{
    error::Error, fs::{ self, Metadata }, io::Write, path::{ Path, PathBuf }, time::SystemTime
};

use twox_hash::XxHash3_64;
use walkdir::{ DirEntry, WalkDir };

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

pub struct HookEvaluator<'a, P: AsRef<Path>> {
    pub package: &'a reloaded3ririext::Package,
    pub cargo: &'a mod_package::CargoInfo,
    csharp_files: std::collections::HashSet<u64>,
    rust_files: Vec<HookSourceFile>,
    delegate_fnptr: std::collections::HashMap<String, String>,
    // target paths
    base_path: P,
    middata: PathBuf,
    riri_hook_dir: PathBuf,
    // riri_hook_bootstrap: PathBuf,
    r2_interface_dir: PathBuf,
    // options
    ignore_files: std::collections::HashSet<PathBuf>,
    // Mod.g.cs storage
    mod_hook_declarations: String,
    mod_hook_set: String,
    uses_shared_scans: bool

}

#[derive(Debug)]
pub struct MacroParseError(String);
impl Error for MacroParseError { }
impl std::fmt::Display for MacroParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing macros during codegen: {}", &self.0)
    }
}

pub struct ReloadedHookClass {
    // pub file: syn::File,
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
}

type SourceFileEvaluationParamMap = std::collections::HashMap<String, riri_mod_tools_impl::riri_hook::HookInfo>;

#[derive(Debug)]
pub struct SourceFileEvaluationResult {
    pub file: syn::File,
    params: SourceFileEvaluationParamMap
}
impl SourceFileEvaluationResult {
    pub fn new(file: syn::File, params: SourceFileEvaluationParamMap) -> Self {
        Self { file, params }
    }
}

pub struct HookBootstrapClassData {
    fn_name: String,
    delegate_path: String,
    fn_path: std::ptr::NonNull<str>,
    class_path: std::ptr::NonNull<str>,
    _pinned: std::marker::PhantomPinned
}

impl HookBootstrapClassData {
    fn new(fn_name: String, delegate_path: String) -> std::pin::Pin<Box<Self>> {
        let mut new = Box::new(HookBootstrapClassData {
            fn_name, delegate_path,
            fn_path: std::ptr::NonNull::from(""),
            class_path: std::ptr::NonNull::from(""),
            _pinned: std::marker::PhantomPinned
        });
        new.fn_path = std::ptr::NonNull::from(&new.delegate_path[..new.delegate_path.len()-8]);
        new.class_path = std::ptr::NonNull::from(&new.delegate_path[..new.delegate_path.len()-9-new.fn_name.len()]);
        Box::into_pin(new)
    }
    fn get_fn_name(&self) -> &str { &self.fn_name }
    fn get_delegate_path(&self) -> &str { &self.delegate_path }
    // SAFETY: fn_path has the same lifetime as the struct
    fn get_fn_path(&self) -> &str { unsafe { self.fn_path.as_ref() } }
    // SAFETY: class_path has the same lifetime as the struct
    fn get_class_path(&self) -> &str { unsafe { self.class_path.as_ref() } }
}

trait HookAssignCodegen {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass, 
        class: &HookBootstrapClassData, delegate_type: &str) 
        -> Result<String, Box<dyn Error>>;
    // fn make_single_static_hook_assign();
    // fn make_single_class_hook_assign();
}

fn get_resolve_function_path(fn_name: &Option<String>, util_namespace: &str, hook_namespace: &str) -> String {
    // TODO: Try to decouple this. This currently *has* to stay in sync with extern methods defined
    // in riri-mod-tools-rt so we can generate the right path in codegen (builtins go into the
    // ReloadedFFI.Utilites namespace, while anything else goes into the hook class's namespace.
    // This sucks.)
    fn get_resolve_function_path_inner(fn_name: &str, util_namespace: &str, hook_namespace: &str) -> String {
        match fn_name {
            "get_address" |
            "get_address_may_thunk" |
            "get_indirect_address_short" |
            "get_indirect_address_short2" |
            "get_indirect_address_long" |
            "get_indirect_address_long4" => {
                format!("{}.{}(x);", util_namespace, fn_name)
            },
            _ => format!("{}.{}(x);", hook_namespace, fn_name)
        }
    }
    match fn_name {
        Some(v) => get_resolve_function_path_inner(v.as_str(), util_namespace, hook_namespace),
        None => get_resolve_function_path_inner("get_address_may_thunk", util_namespace, hook_namespace)
    } 
}

struct HookAssignCodegenStaticOffset(riri_mod_tools_impl::hook_parse::StaticOffset);
impl HookAssignCodegen for HookAssignCodegenStaticOffset {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        class: &HookBootstrapClassData, delegate_type: &str
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        hook_assign.push_str(&format!("var addr = {}\n",
            get_resolve_function_path(&None, &evaluator.ffi_utility_class(), &hooks_class)));
        hook_assign.push_str(&format!("            _{} = _hooks!.CreateHook<{}>({}, addr).Activate();\n",
            class.get_fn_name(), class.get_delegate_path(), class.get_fn_path()));
        hook_assign.push_str(&format!("            {}.{}({})_{}.OriginalFunctionWrapperAddress);\n",
          &ffi.csharp_class_name(), 
          &riri_mod_tools_impl::hook_codegen::Reloaded2CSharpHook::make_hook_set_string(&class.get_fn_name().to_ascii_uppercase()),
          delegate_type,
          class.get_fn_name()));
        Ok(hook_assign)
    }
}
impl HookAssignCodegenStaticOffset {
    fn new(parm: riri_mod_tools_impl::hook_parse::StaticOffset) -> Self { Self(parm) }
}

struct HookAssignCodegenDynamicOffset<'a>(&'a riri_mod_tools_impl::hook_parse::DynamicOffset);
impl<'a> HookAssignCodegen for HookAssignCodegenDynamicOffset<'a> {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        class: &HookBootstrapClassData, delegate_type: &str
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        // match class.hook_parm
        hook_assign.push_str(&format!("SigScan(\"{}\", \"{}\", x => ",
            self.0.sig, class.get_fn_name()));
        hook_assign.push_str("\x7b\n");
        hook_assign.push_str(&format!("                var addr = {}\n",
            get_resolve_function_path(&self.0.resolve_type, &evaluator.ffi_utility_class(), &hooks_class)));
        hook_assign.push_str(&format!("                _{} = _hooks!.CreateHook<{}>({}, (long)addr).Activate();\n",
            class.get_fn_name(), class.get_delegate_path(), class.get_fn_path()));
        hook_assign.push_str(&format!("                {}.{}(({})_{}.OriginalFunctionWrapperAddress);\n",
          class.get_class_path(), 
          &riri_mod_tools_impl::hook_codegen::Reloaded2CSharpHook::make_hook_set_string(&class.get_fn_name().to_ascii_uppercase()),
          delegate_type,
          class.get_fn_name()
        ));
        hook_assign.push_str("            \x7d);\n");
        Ok(hook_assign)
    }
}
impl<'a> HookAssignCodegenDynamicOffset<'a> {
    fn new(parm: &'a riri_mod_tools_impl::hook_parse::DynamicOffset) -> Self { Self(parm) }
}

struct HookAssignCodegenDynamicOffsetSharedScans<'a>(&'a riri_mod_tools_impl::hook_parse::DynamicOffset);
impl<'a> HookAssignCodegen for HookAssignCodegenDynamicOffsetSharedScans<'a> {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        class: &HookBootstrapClassData, delegate_type: &str
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let shared_scan_state = self.0.shared_scan.unwrap();
        let mut hook_assign = String::new();
        if shared_scan_state == riri_mod_tools_impl::hook_parse::RyoTuneSharedScan::Produce {
            hook_assign.push_str(&format!("_sharedScan.AddScan<{}>({});",
                class.get_delegate_path(), self.0.sig));
        }
        hook_assign.push_str(&format!("_sharedScan.CreateListener<{}>(x => )",
            class.get_delegate_path()));

        hook_assign.push_str("\x7b\n");
        hook_assign.push_str(&format!("                var addr = {}\n",
            get_resolve_function_path(&self.0.resolve_type, &evaluator.ffi_utility_class(), &hooks_class)));
        hook_assign.push_str(&format!("                _{} = _hooks!.CreateHook<{}>({}, addr).Activate();\n",
            class.get_fn_name(), class.get_delegate_path(), class.get_fn_path()));
        hook_assign.push_str(&format!("                {}.{}({})_{}.OriginalFunctionWrapperAddress);\n",
          class.get_class_path(), 
          &riri_mod_tools_impl::hook_codegen::Reloaded2CSharpHook::make_hook_set_string(&class.get_fn_name().to_ascii_uppercase()),
          delegate_type,
          class.get_fn_name()
        ));
        hook_assign.push_str("            \x7d);\n");
        Ok(hook_assign)
    }
}
impl<'a> HookAssignCodegenDynamicOffsetSharedScans<'a> {
    fn new(parm: &'a riri_mod_tools_impl::hook_parse::DynamicOffset) -> Self { Self(parm) }
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

    pub fn evaluate_rust_file<T: AsRef<Path>>(path: T) -> Result<SourceFileEvaluationResult, Box<dyn Error>> {
        let src_str = fs::read_to_string(path.as_ref())?;
        let mut src_syntax = syn::parse_file(&src_str)?;
        let mut mutated_items: Vec<(usize, riri_mod_tools_impl::riri_hook::HookBuildScriptResult)> = vec![];
        for (i, src_item) in src_syntax.items.iter_mut().enumerate() {
            // evaluate #[riri_hook], make FFI bindings into ReloadedFFI.Hooks.[xxhash64]
            match src_item {
                syn::Item::Fn(f) => {
                    let fn_attr_pos = f.attrs.iter().position(|f| f.path().is_ident("riri_hook_fn"));
                    if let Some(p) = fn_attr_pos {
                        let insertion = riri_mod_tools_impl::riri_hook::riri_hook_fn_build(
                            f.attrs.remove(p).meta.require_list()?.tokens.clone(),
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
        let mut items: SourceFileEvaluationParamMap = std::collections::HashMap::new();
        // Attach mutated items
        for mutated_item in &mut mutated_items {
            // items.insert(mutated_item.1.name, mutated_item.1.args);
            src_syntax.items[mutated_item.0] = mutated_item.1.items.remove(0);
            if mutated_item.1.items.len() > 1 { 
                src_syntax.items.extend_from_slice(mutated_item.1.items.as_slice());
                src_syntax.items[mutated_item.0+1..].rotate_right(mutated_item.1.items.len());
            }
        }
        // Move name and args to evaluation result
        for mutated_item in mutated_items {
            items.insert(mutated_item.1.name, mutated_item.1.args);
        }
        // Ok(src_syntax)
        Ok(SourceFileEvaluationResult::new(src_syntax, items))
    } 
    // TODO: Handle multiple cases with if-else chain

    // - Add an IHook<DelegateType> for the target C# hook
    // - Use the SigScan function to set the value of that C# hook, then get the
    // OriginalFunctionWrapperAddress to pass into the Rust hook set function. The Sigscan will
    // need a reference to Rust's hooked function and the original function pointer set.
    // - Delegate and function are already defined in ReloadedFFI.Hooks
    pub fn generate_hook_bootstrap(&mut self, ffi: &ReloadedHookClass) -> Result<String, Box<dyn Error>> {
        let mut hook_decl = String::new();
        let mut hook_assign = String::new();
        for item in &ffi.eval.file.items {
            match item {
                syn::Item::Fn(f) => {
                    let fn_name = f.sig.ident.to_string();
                    let hook_parm = match ffi.eval.params.get(&fn_name) {
                        Some(v) => v,
                        // None => return Err(Box::new(MacroParseError(format!("Could not find matching parameter for function {}", fn_name))))
                        None => continue
                    };
                    let delegate_path = format!("{}.{}.{}Delegate", &self.ffi_hook_namespace(), &ffi.csharp_class_name(), &fn_name);
                    // let fn_path = &delegate_path[..delegate_path.len()-8];
                    if hook_parm.0.len() > 1 {
                        todo!("Multiple hook entries TODO!");
                    }
                    println!("type: {:?}", f.sig);
                    // Create a delegate type to cast the function pointer
                    // f.sig.output
                    let mut delegate_type = "delegate* unmanaged[Stdcall]<".to_owned();
                    for input in &f.sig.inputs {
                        if let syn::FnArg::Typed(t) = input {
                            delegate_type.push_str(&format!("{}, ", riri_mod_tools_impl::csharp::Utils::to_csharp_typename(&t.ty)?));
                        }
                    }
                    match &f.sig.output {
                        syn::ReturnType::Default => delegate_type.push_str("void"),
                        syn::ReturnType::Type(_, t) => delegate_type.push_str(&riri_mod_tools_impl::csharp::Utils::to_csharp_typename(&t)?)
                    };
                    delegate_type.push_str(">");
                    let class_data = HookBootstrapClassData::new(fn_name, delegate_path);
                    hook_decl.push_str(&format!("private Reloaded.Hooks.Definitions.IHook<{}>? _{};\n", 
                        class_data.get_delegate_path(), class_data.get_fn_name()));
                    hook_assign = match &hook_parm.0[0] {
                        riri_mod_tools_impl::hook_parse::HookEntry::Static(s) => {
                            let res = HookAssignCodegenStaticOffset::new(*s);
                            res.make_single_function_hook_assign(self, ffi, &class_data, &delegate_type)?
                        },
                        riri_mod_tools_impl::hook_parse::HookEntry::Dyn(d) => {
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
                        }
                    };
                },
                syn::Item::Static(s) => {

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
        out.writeln(&hook_decl);
        out.fmtln(format_args!("public void {}()\n", ffi.csharp_register_hooks()))?;
        out.indent()?;
        out.writeln(&hook_assign);
        for _ in 0..3 {
            out.unindent()?;
        }
        Ok(out.submit())
    }

    fn cshookgen_get_hash(file_name: &Path) -> u64 {
        let stem = file_name.to_str().unwrap().split_once(".").unwrap().0;
        u64::from_str_radix(stem, 16).unwrap()
    }

    pub fn evaluate_hooks(
        &mut self, cb: fn(&mut Self, ReloadedHookClass) -> Result<(), Box<dyn Error>>) 
        -> Result<Vec<String>, Box<dyn Error>> {
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
        for evaluated_file in evaluated_files_full {
            // call_hook_registers.push_str(&format!("                {}();", evaluated_file.1.csharp_register_hooks()));
            call_hook_registers.push(evaluated_file.1.csharp_register_hooks());
            let cs_file = evaluated_file.1.cs_path.to_str().unwrap().to_owned();
            cb(self, evaluated_file.1)?;
            let mut cs_file = fs::OpenOptions::new().append(true).open(cs_file)?;
            cs_file.write(evaluated_file.0.as_bytes())?;
        }
        Ok(call_hook_registers)
    }

    pub fn generate_mod_main(&self, call_hooks: Vec<String>) -> Result<(), Box<dyn Error>> {
        pub fn into_toml_array(arr: Vec<String>) -> toml::value::Array {
            let mut out = toml::value::Array::with_capacity(arr.len());
            for s in arr { out.push(toml::Value::String(s)); }
            out
        }

        let mut mod_file = fs::File::create(self.middata.join("Mod.g.cs"))?;
        let mut utils_file = fs::File::create(self.middata.join("Utils.g.cs"))?;
        let logger_prefix = match &self.package.LoggerPrefix {
            Some(v) => v.to_owned(),
            None => self.package.get_mod_id().to_owned()
        };
        let mut hbs = Handlebars::new();
        hbs.register_template_string("main", crate::hbs::mod_main::FILE)?;
        hbs.register_template_string("utils", crate::hbs::ffi_builtin::FILE)?;
        let mut data = toml::Table::new();
        data.insert("mod_id".to_owned(), toml::Value::String(self.package.get_mod_id().to_owned()));
        data.insert("mod_name".to_owned(), toml::Value::String(self.package.Name.to_owned()));
        data.insert("dll_name".to_owned(), toml::Value::String(self.get_output_dll_name()?));
        data.insert("logger_prefix".to_owned(), toml::Value::String(logger_prefix));
        data.insert("uses_shared_scans".to_owned(), toml::Value::Boolean(self.uses_shared_scans));
        data.insert("utility_namespace".to_owned(), toml::Value::String(self.ffi_utility_class()));
        data.insert("ffi_namespace".to_owned(), toml::Value::String(self.ffi_namespace()));
        data.insert("exports_interfaces".to_owned(), toml::Value::Boolean(false));
        data.insert("register_hook_fn".to_owned(), toml::Value::Array(into_toml_array(call_hooks)));
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
