#![allow(dead_code, unused_variables)]
use crate::reloaded_codegen::{ HookEvaluator, ReloadedHookClass };
use std::{
    collections::HashMap,
    error::Error,
    marker::PhantomPinned, 
    path::Path,
    pin::Pin,
    ptr::NonNull,
};
use riri_mod_tools_impl::{
    hook_codegen::Reloaded2CSharpHook,
    hook_parse::{
        AssemblyFunctionHook,
        AssemblyFunctionHookData,
        DynamicOffset,
        HookConditional,
        HookEntry,
        StaticOffset,
        RyoTuneSharedScan
    },
    riri_hook::SourceFileEvaluationType,
};

pub(crate) type SourceFileEvaluationParamMapEx = HashMap<String, SourceFileEvaluationType>;

#[derive(Debug)]
pub struct SourceFileEvaluationResult {
    pub file: syn::File,
    pub(crate) params: SourceFileEvaluationParamMapEx
}
impl SourceFileEvaluationResult {
    pub fn new(file: syn::File, params: SourceFileEvaluationParamMapEx) -> Self {
        Self { file, params }
    }
    pub fn get_assembly_evaluation(&self, name: &str) -> Option<&AssemblyFunctionHook> {
        match self.params.get(name) {
            Some(v) => 
                if let SourceFileEvaluationType::Inline(i) = v {
                    Some(i)
                } else { return None },
            None => return None
        }
    }
}

pub struct HookBootstrapFunctionState {
    fn_name: String,
    delegate_path: String,
    fn_path: NonNull<str>,
    class_path: NonNull<str>,
    _pinned: PhantomPinned
}

impl HookBootstrapFunctionState {
    pub(crate) fn new(fn_name: String, delegate_path: String) -> Pin<Box<Self>> {
        let mut new = Box::new(HookBootstrapFunctionState {
            fn_name, delegate_path,
            fn_path: NonNull::from(""),
            class_path: NonNull::from(""),
            _pinned: PhantomPinned
        });
        new.fn_path = NonNull::from(&new.delegate_path[..new.delegate_path.len()-8]);
        new.class_path = NonNull::from(&new.delegate_path[..new.delegate_path.len()-9-new.fn_name.len()]);
        Box::into_pin(new)
    }
    pub(crate) fn get_fn_name(&self) -> &str { &self.fn_name }
    pub(crate) fn get_delegate_path(&self) -> &str { &self.delegate_path }
    // SAFETY: fn_path has the same lifetime as the struct
    pub(crate) fn get_fn_path(&self) -> &str { unsafe { self.fn_path.as_ref() } }
    // SAFETY: class_path has the same lifetime as the struct
    pub(crate) fn get_class_path(&self) -> &str { unsafe { self.class_path.as_ref() } }
}

pub struct HookBootstrapStaticState {
    static_type: String,
    static_name: String
}

impl HookBootstrapStaticState {
    pub(crate) fn new(static_type: String, static_name: String) -> Self {
        Self { static_type, static_name }
    }
}

pub struct HookBootstrapClassState;

pub(crate) trait HookAssignCodegen {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass, 
        class: &HookBootstrapFunctionState, delegate_type: &str) 
        -> Result<String, Box<dyn Error>>;
    fn make_function_hook_assign_assembly<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass, 
        class: &HookBootstrapFunctionState, delegate_type: &str,
        assemble_info: &AssemblyFunctionHookData,
        cond: &HookConditional) 
        -> Result<String, Box<dyn Error>>;
    fn make_single_static_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        state: &HookBootstrapStaticState
    ) -> Result<String, Box<dyn Error>>;
    // fn make_single_class_hook_assign();
}

fn get_resolve_function_path(fn_name: &Option<String>, util_namespace: &str, hook_namespace: &str, input_value: Option<String>, cast: bool) -> String {
    // TODO: Try to decouple this. This currently *has* to stay in sync with extern methods defined
    // in riri-mod-tools-rt so we can generate the right path in codegen (builtins go into the
    // ReloadedFFI.Utilites namespace, while anything else goes into the hook class's namespace.
    // This sucks.)
    fn get_resolve_function_path_inner(fn_name: &str, util_namespace: &str, hook_namespace: &str, input_value: Option<String>, cast: bool) -> String {
        let inner_value = match input_value {
            Some(v) => v,
            None => "x".to_owned()
        };
        let cast_type = match cast {
            true => "(nuint)",
            false => ""
        };
        match fn_name {
            "get_address" |
            "get_address_may_thunk" |
            "get_indirect_address_short" |
            "get_indirect_address_short2" |
            "get_indirect_address_long" |
            "get_indirect_address_long4" => {
                format!("{}.{}({}{});", util_namespace, fn_name, cast_type, inner_value)
            },
            _ => format!("{}.{}({}{});", hook_namespace, fn_name, cast_type, inner_value)
        }
    }
    match fn_name {
        Some(v) => get_resolve_function_path_inner(v.as_str(), util_namespace, hook_namespace, input_value, cast),
        None => get_resolve_function_path_inner("get_address_may_thunk", util_namespace, hook_namespace, input_value, cast)
    } 
}

pub(crate) struct HookAssignCodegenStaticOffset(StaticOffset);
impl HookAssignCodegen for HookAssignCodegenStaticOffset {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        class: &HookBootstrapFunctionState, delegate_type: &str
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        hook_assign.push_str(&format!("var addr_{} = {}\n",
            &class.get_fn_name(),
            get_resolve_function_path(
                &None, &evaluator.ffi_utility_class(), 
                &hooks_class, Some(format!("0x{:x}", self.0.0)), false
        )));
        hook_assign.push_str(&format!("            _{} = _hooks!.CreateHook<{}>({}, (long)addr_{}).Activate();\n",
            class.get_fn_name(), class.get_delegate_path(), class.get_fn_path(), class.get_fn_name()));
        hook_assign.push_str(&format!("            {}.{}.{}(({})_{}.OriginalFunctionWrapperAddress);\n",
          &evaluator.ffi_hook_namespace(),
          &ffi.csharp_class_name(), 
          &Reloaded2CSharpHook::make_hook_set_string(&class.get_fn_name().to_ascii_uppercase()),
          delegate_type,
          class.get_fn_name()));
        Ok(hook_assign)
    }
    fn make_function_hook_assign_assembly<P: AsRef<Path>>(
            &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
            class: &HookBootstrapFunctionState, delegate_type: &str,
            assemble_info: &AssemblyFunctionHookData,
            cond: &HookConditional
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        // Address resolver
        hook_assign.push_str(&format!("var addr_{} = {}\n",
            &class.get_fn_name(),
            get_resolve_function_path(
                &None, &evaluator.ffi_utility_class(), 
                &hooks_class, Some(format!("0x{:x}", self.0.0)), false
        )));
        // Build assembly glue
        hook_assign.push_str(&format!("            string[] function_{} = \n", &class.get_fn_name()));
        hook_assign.push_str("            \x7b\n");
        hook_assign.push_str("                \"use64\",\n");
        if let Some(v) = &assemble_info.asm_insert_before {
            hook_assign.push_str(v);
        }
        hook_assign.push_str(&format!("                $\"{{_hooks!.Utilities.GetAbsoluteCallMnemonics({}, out _{}_WRAP{})}}\",\n",
            &class.get_fn_path(), &class.get_fn_name(), cond));
        if let Some(v) = &assemble_info.asm_insert_after {
            hook_assign.push_str(v);
        }
        hook_assign.push_str("            \x7d;\n");
        // Assembly hook
        let exec_mode: &str = assemble_info.execute_mode.into();
        hook_assign.push_str(&format!("            _{}_ASM = _hooks!.CreateAsmHook(function_{}, (long)addr_{}, Reloaded.Hooks.Definitions.Enums.AsmHookBehaviour.{}).Activate();\n",
            class.get_fn_name(), class.get_fn_name(), class.get_fn_name(), exec_mode));
        Ok(hook_assign)
    }
    fn make_single_static_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        state: &HookBootstrapStaticState
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        hook_assign.push_str(&format!("var addr_{} = {}\n",
            state.static_name,
            get_resolve_function_path(
                &None, &evaluator.ffi_utility_class(), 
                &hooks_class, Some(format!("0x{:x}", self.0.0)), false
        )));
        hook_assign.push_str(&format!("{}.{}.{}(({})addr_{});\n", 
            &evaluator.ffi_hook_namespace(),
            &ffi.csharp_class_name(),
            &Reloaded2CSharpHook::make_hook_set_string(&state.static_name), 
            &state.static_type,
            &state.static_name
        ));
        Ok(hook_assign)
    }

}
impl HookAssignCodegenStaticOffset {
    pub(crate) fn new(parm: StaticOffset) -> Self { Self(parm) }
}

pub(crate) struct HookAssignCodegenDynamicOffset<'a>(&'a DynamicOffset);
impl<'a> HookAssignCodegen for HookAssignCodegenDynamicOffset<'a> {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        class: &HookBootstrapFunctionState, delegate_type: &str
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        // match class.hook_parm
        hook_assign.push_str(&format!("SigScan(\"{}\", \"{}\", x => ",
            self.0.sig, class.get_fn_name()));
        hook_assign.push_str("\x7b\n");
        hook_assign.push_str(&format!("                var addr = {}\n",
            get_resolve_function_path(
                &self.0.resolve_type, &evaluator.ffi_utility_class(), 
                &hooks_class, None, false
        )));
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
    fn make_function_hook_assign_assembly<P: AsRef<Path>>(
            &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
            class: &HookBootstrapFunctionState, delegate_type: &str,
            assemble_info: &AssemblyFunctionHookData,
            cond: &HookConditional
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        hook_assign.push_str(&format!("SigScan(\"{}\", \"{}\", x => ",
            self.0.sig, class.get_fn_name()));
        hook_assign.push_str("\x7b\n");
        // Address resolver
        hook_assign.push_str(&format!("                var addr = {}\n",
            get_resolve_function_path(
                &self.0.resolve_type, &evaluator.ffi_utility_class(), 
                &hooks_class, None, false
        )));
        // Build assembly glue
        hook_assign.push_str("            string[] function = \n");
        hook_assign.push_str("            \x7b\n");
        hook_assign.push_str("                \"use64\",\n");
        if let Some(v) = &assemble_info.asm_insert_before {
            hook_assign.push_str(v);
        }
        hook_assign.push_str(&format!("                $\"{{_hooks!.Utilities.GetAbsoluteCallMnemonics({}, out _{}_WRAP{})}}\",\n",
            &class.get_fn_path(), &class.get_fn_name(), cond));
        if let Some(v) = &assemble_info.asm_insert_after {
            hook_assign.push_str(v);
        }
        hook_assign.push_str("            \x7d;\n");
        // Assembly hook
        let exec_mode: &str = assemble_info.execute_mode.into();
        hook_assign.push_str(&format!("            _{}_ASM = _hooks!.CreateAsmHook(function, (long)addr, Reloaded.Hooks.Definitions.Enums.AsmHookBehaviour.{}).Activate();\n",
            class.get_fn_name(), exec_mode));
        hook_assign.push_str("            \x7d);\n");
        Ok(hook_assign)
    }
    fn make_single_static_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        state: &HookBootstrapStaticState
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        hook_assign.push_str(&format!("SigScan(\"{}\", \"{}\", x => ",
            self.0.sig, &state.static_name.to_ascii_lowercase()));
        hook_assign.push_str("\x7b\n");
        hook_assign.push_str(&format!("                var addr = {}\n",
            get_resolve_function_path(
                &self.0.resolve_type, &evaluator.ffi_utility_class(), 
                &hooks_class, None, false
        )));
        hook_assign.push_str(&format!("                {}.{}.{}(({})addr);\n",
            &evaluator.ffi_hook_namespace(),
            &ffi.csharp_class_name(),
            &Reloaded2CSharpHook::make_hook_set_string(&state.static_name), 
            &state.static_type
        ));
        hook_assign.push_str("            \x7d);\n");
        Ok(hook_assign)
    }
}
impl<'a> HookAssignCodegenDynamicOffset<'a> {
    pub(crate) fn new(parm: &'a DynamicOffset) -> Self { Self(parm) }
}

pub(crate) struct HookAssignCodegenDynamicOffsetSharedScans<'a>(&'a DynamicOffset);
impl<'a> HookAssignCodegen for HookAssignCodegenDynamicOffsetSharedScans<'a> {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        class: &HookBootstrapFunctionState, delegate_type: &str
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let shared_scan_state = self.0.shared_scan.unwrap();
        let mut hook_assign = String::new();
        if shared_scan_state == RyoTuneSharedScan::Produce {
            hook_assign.push_str(&format!("_sharedScans!.AddScan<{}>({});\n",
                class.get_delegate_path(), self.0.sig));
        }
        hook_assign.push_str(&format!("_sharedScans!.CreateListener<{}>(x => ",
            class.get_delegate_path()));

        hook_assign.push_str("\x7b\n");
        // Shared Scans uses absolute addresses, convert to relative address
        hook_assign.push_str(&format!("                var addr_relative = x - _baseAddress;\n"));
        hook_assign.push_str(&format!("                var addr = {}\n",
            get_resolve_function_path(
                &self.0.resolve_type, &evaluator.ffi_utility_class(), 
                &hooks_class, Some("addr_relative".to_string()), true
        )));
        hook_assign.push_str(&format!("                _{} = _hooks!.CreateHook<{}>({}, addr).Activate();\n",
            class.get_fn_name(), class.get_delegate_path(), class.get_fn_path()));
        hook_assign.push_str(&format!("                {}.{}(({})_{}.OriginalFunctionWrapperAddress);\n",
          class.get_class_path(), 
          &Reloaded2CSharpHook::make_hook_set_string(&class.get_fn_name().to_ascii_uppercase()),
          delegate_type,
          class.get_fn_name()
        ));
        hook_assign.push_str("            \x7d);\n");
        Ok(hook_assign)
    }
    fn make_function_hook_assign_assembly<P: AsRef<Path>>(
            &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
            class: &HookBootstrapFunctionState, delegate_type: &str,
            assemble_info: &AssemblyFunctionHookData,
            cond: &HookConditional
        ) -> Result<String, Box<dyn Error>> {
        todo!("...")
    }
    fn make_single_static_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
        state: &HookBootstrapStaticState
        ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let shared_scan_state = self.0.shared_scan.unwrap();
        let mut hook_assign = String::new();
        if shared_scan_state == RyoTuneSharedScan::Produce {
            hook_assign.push_str(&format!("_sharedScans!.AddScan(\"{}\", {});\n",
                &state.static_name, self.0.sig));
        }
        hook_assign.push_str(&format!("_sharedScans!.CreateListener(\"{}\", x => ",
            &state.static_name));
        hook_assign.push_str("\x7b\n");
        hook_assign.push_str(&format!("                var addr_relative = x - _baseAddress;\n"));
        hook_assign.push_str(&format!("                var addr = {}\n",
            get_resolve_function_path(
                &self.0.resolve_type, &evaluator.ffi_utility_class(), 
                &hooks_class, Some("addr_relative".to_string()), true
        )));
        hook_assign.push_str(&format!("                {}.{}.{}(({})addr);\n",
            &evaluator.ffi_hook_namespace(),
            &ffi.csharp_class_name(),
            &Reloaded2CSharpHook::make_hook_set_string(&state.static_name), 
            &state.static_type
        ));
        hook_assign.push_str("            \x7d);\n");
        Ok(hook_assign)
    }
}
impl<'a> HookAssignCodegenDynamicOffsetSharedScans<'a> {
   pub(crate) fn new(parm: &'a DynamicOffset) -> Self { Self(parm) }
}

pub(crate) struct HookAssignCodegenMultiple(Vec<HookEntry>);

pub(crate) trait ModEventFunction {
    fn make_function_call<P: AsRef<Path>>(
        evaluator: &HookEvaluator<P>,
        ffi: &ReloadedHookClass,
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str
    ) -> Result<String, Box<dyn Error>>;
}

pub struct InitFunction;
impl InitFunction {
    pub(crate) fn make_init_function_call<P: AsRef<Path>>(
        evaluator: &HookEvaluator<P>,
        ffi: &ReloadedHookClass,
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str
    ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        Ok(format!("{}.{}();\n", hooks_class, class_data.get_fn_name()))
    }
}

impl ModEventFunction for InitFunction {
    fn make_function_call<P: AsRef<Path>>(
        evaluator: &HookEvaluator<P>,
        ffi: &ReloadedHookClass,
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str
    ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        Ok(format!("{}.{}();\n", hooks_class, class_data.get_fn_name()))
    }
}

pub struct ModLoadingFunction;

impl ModEventFunction for ModLoadingFunction {
    fn make_function_call<P: AsRef<Path>>(
        evaluator: &HookEvaluator<P>,
        ffi: &ReloadedHookClass,
        class_data: &HookBootstrapFunctionState,
        delegate_type: &str
    ) -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        Ok(format!("{}.{}(confHandle);\n", hooks_class, class_data.get_fn_name()))
    }
}

pub(crate) struct HookAssignCodegenUserDefined;
impl HookAssignCodegenUserDefined {
    pub(crate) fn new() -> Self { Self }
}
impl HookAssignCodegen for HookAssignCodegenUserDefined {
    fn make_single_function_hook_assign<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass, 
        class: &HookBootstrapFunctionState, delegate_type: &str) 
        -> Result<String, Box<dyn Error>> {
        let hooks_class = format!("{}.{}", &evaluator.ffi_hook_namespace(), &ffi.csharp_class_name());
        let mut hook_assign = String::new();
        hook_assign.push_str(&format!("{}.{}(&UserDefined_{});\n",
            &hooks_class, 
            Reloaded2CSharpHook::make_user_set_string(&class.fn_name.to_ascii_uppercase()),
            &class.get_fn_name()

        ));
        Ok(hook_assign)
    }
    fn make_function_hook_assign_assembly<P: AsRef<Path>>(
        &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass, 
        class: &HookBootstrapFunctionState, delegate_type: &str,
        assemble_info: &AssemblyFunctionHookData,
        cond: &HookConditional) 
        -> Result<String, Box<dyn Error>> {
        Ok(String::new())
    }
    fn make_single_static_hook_assign<P: AsRef<Path>>(
            &self, evaluator: &HookEvaluator<P>, ffi: &ReloadedHookClass,
            state: &HookBootstrapStaticState
        ) -> Result<String, Box<dyn Error>> {
        Ok(String::new())
    }
}