//! An attribute macro that allows a convenient method for retrieving global statics and functions
//! and hooking onto functions. The macro's behavior depends on the mod framework being targeted -
//! currently this is only designed to work in tandem with the Reloaded-II mod loader: this hook
//! here generates Rust code as well as significant C# code to take advantage of Reloaded-II's
//! hooking and signature scanning libraries.
//!
//! Hooking style based on the implementation used in skyline-rs:
//! https://github.com/ultimate-research/skyline-rs
//!

// riri_hook syntax WIP
// #[cfg(target_arch ="x86_64", target_os = "windows", target_pointer_width = "64")]
// #[riri_hook(
//      static_offset(0x1000), 
//      static_offset( XRD759_STEAM_102 => { 0x1000 } ) 
//      dyn_offset(
//          XRD759_STEAM_109 => (
//              signature = "48 83 EC 28 8B 44 24 ?? 89 05 ?? ?? ?? ??",
//              resolve_type = RiriHookType::Direct,
//              calling_convention = RiriHookCallingConvention::Microsoft,
//              shared_scan = RyoTuneSharedScanType::Register
//          ),
//      )
// )]
//

#![allow(dead_code, unused_imports, unused_mut, unused_variables)]
use proc_macro2::{
    Span as Span2,
    TokenStream as TokenStream2
};
use quote::{
    format_ident,
    quote,
    ToTokens
};
use std::{
    borrow::{ Borrow, BorrowMut },
    env::{ var, var_os },
    mem::MaybeUninit
};
use syn::{
    self,
    parse::{ Parse, Parser, ParseStream },
    punctuated::Punctuated,
    spanned::Spanned,
    Token
};

use crate::{
    csharp,
    hook_codegen::{  
        CppClassMethodGenerator,
        HookFramework,
        Reloaded2CSharpHook 
    },
    hook_parse::{
        AssemblyFunctionHook,
        CppClassMethods,
        HookConditional,
        HookEntry,
        HookInfoParam,
        HookParseTools,
        StaticVarHook
    }
};

#[derive(Debug)]
pub struct HookInfo(pub Vec<(HookConditional, HookEntry)>);
impl HookInfo {
    pub(crate) fn new(entries: Vec<(HookConditional, HookEntry)>) -> Self {
        Self(entries)
    }
}

pub(crate) enum HookItemType {
    Function(syn::ItemFn),
    Static(StaticVarHook),
    CppClass(syn::ItemStruct),
}

impl quote::ToTokens for HookItemType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            HookItemType::Function(f) => f.to_tokens(tokens),
            HookItemType::Static(s) => s.to_tokens(tokens),
            HookItemType::CppClass(c) => c.to_tokens(tokens),
        }
    }
}

impl HookItemType {
    fn get_name(&self) -> String {
        match self {
            HookItemType::Function(f) => f.sig.ident.to_string(),
            HookItemType::Static(s) => s.name.to_string(),
            HookItemType::CppClass(c) => c.ident.to_string()
        }
    }
}


// #[riri_hook_fn]
pub fn riri_hook_fn_impl(input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    // Parse macro
    let mut target: HookItemType = match syn::parse2(annotated_item) {
        Ok(n) => HookItemType::Function(n),
        Err(e) => return TokenStream2::from(e.to_compile_error())
    }; 
    let args: HookInfo = match syn::parse2(input) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let mut transformer = Reloaded2CSharpHook::new();
    // Code generation
    let transformed = match transformer.codegen_rust(&mut target) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    transformed
}

struct HookFunctionBuildScriptItems {
    original_function_ptr: syn::ItemStatic,
    set_original_function: syn::ItemFn,
    hooked_function: syn::ItemFn
}

impl Parse for HookFunctionBuildScriptItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(HookFunctionBuildScriptItems {
            original_function_ptr: input.parse()?,
            set_original_function: input.parse()?,
            hooked_function: input.parse()?,
        })
    }
}
impl HookFunctionBuildScriptItems {
    fn to_items(self) -> Vec<syn::Item> {
        vec![
            syn::Item::Static(self.original_function_ptr),
            syn::Item::Fn(self.set_original_function),
            syn::Item::Fn(self.hooked_function)
        ]
    }
}

#[derive(Debug)]
pub enum SourceFileEvaluationType {
    CFunction(HookInfo),
    Inline(AssemblyFunctionHook)
}

pub struct HookBuildScriptResult {
    pub name: String,
    pub items: Vec<syn::Item>,
    // pub args: HookInfo
    pub args: SourceFileEvaluationType
}

pub fn riri_hook_fn_build(input: TokenStream2, annotated_item: syn::ItemFn) -> syn::Result<HookBuildScriptResult> {
    let mut target = HookItemType::Function(annotated_item);
    let args = SourceFileEvaluationType::CFunction(syn::parse2(input)?);
    let mut transformer = Reloaded2CSharpHook::new();
    let transformed = transformer.codegen_rust(&mut target)?;
    // parse back into items to inject into file
    Ok(HookBuildScriptResult {
        name: target.get_name(),
        items: HookFunctionBuildScriptItems::parse.parse2(transformed)?.to_items(),
        args
    })
}

// #[riri_hook_static]
fn get_riri_hook_macro_inner(target: syn::ItemMacro) -> syn::Result<StaticVarHook> {
    if target.mac.path.is_ident("riri_static") { 
        return Ok(syn::parse2(target.mac.tokens)?)
    } 
    Err(syn::Error::new(target.span(), "riri_static_hook should only annotate riri_hook! instances"))
}

fn get_riri_hook_macro(item: TokenStream2) -> syn::Result<StaticVarHook> {
    match syn::parse2(item) {
        Ok(v) => get_riri_hook_macro_inner(v),
        Err(e) => Err(e)
    }
}

pub fn riri_hook_static_impl(input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    let mut target: HookItemType = match get_riri_hook_macro(annotated_item) {
        Ok(n) => HookItemType::Static(n),
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let args: HookInfo = match syn::parse2(input) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let mut transformer = Reloaded2CSharpHook::new();
    let transformed = match transformer.codegen_rust(&mut target) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    transformed
}
struct HookStaticBuildScriptItems {
    global_static: syn::ItemStatic,
    set_static_fn: syn::ItemFn
}
impl Parse for HookStaticBuildScriptItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(HookStaticBuildScriptItems {
            global_static: input.parse()?,
            set_static_fn: input.parse()?
        })
    }
}
impl HookStaticBuildScriptItems {
    fn to_items(self) -> Vec<syn::Item> {
        vec! [
            syn::Item::Static(self.global_static),
            syn::Item::Fn(self.set_static_fn)
        ]
    }
}
pub fn riri_hook_static_build(input: TokenStream2, annotated_item: syn::ItemMacro) -> syn::Result<HookBuildScriptResult> {
    let mut target = HookItemType::Static(get_riri_hook_macro_inner(annotated_item)?);
    let args = SourceFileEvaluationType::CFunction(syn::parse2(input)?);
    let mut transformer = Reloaded2CSharpHook::new();
    let transformed = transformer.codegen_rust(&mut target)?;
    Ok(HookBuildScriptResult {
        name: target.get_name(),
        items: HookStaticBuildScriptItems::parse.parse2(transformed)?.to_items(),
        args
    })
}

// riri_static!
pub fn riri_static(input: TokenStream2) -> TokenStream2 {
    TokenStream2::from(syn::Error::new(input.span(), "riri_static! should be annotated using riri_hook_static").to_compile_error())
}

// #[cpp_class]
pub fn cpp_class_impl(input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    let mut target: HookItemType = match syn::parse2(annotated_item) {
        Ok(n) => HookItemType::CppClass(n),
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let args: HookInfo = match syn::parse2(input) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let mut transformer = Reloaded2CSharpHook::new();
    let transformed = match transformer.codegen_rust(&mut target) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    transformed
}

// original_function!
pub fn original_function_impl(input: TokenStream2) -> TokenStream2 {
    TokenStream2::from(syn::Error::new(input.span(), "original_function! should only be included in hooked functions").to_compile_error())
}

// #[cpp_class_methods(path = "prefix", auto_drop = 0)]
pub fn cpp_class_methods_impl(input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    let mut target: syn::ItemImpl = match syn::parse2(annotated_item) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let args: CppClassMethods = match syn::parse2(input) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let codegen = CppClassMethodGenerator;
    codegen.codegen_rust(&target, &args);
    TokenStream2::new()
}

pub fn vtable_method_impl(input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    TokenStream2::from(syn::Error::new(input.span(), "vtable_method should only be added inside of an implementation annotated with cpp_class_methods!").to_compile_error())
}

// #[riri_hook_inline_fn]
pub fn riri_hook_inline_fn_impl(input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    // Parse macro
    let mut target: HookItemType = match syn::parse2(annotated_item) {
        Ok(n) => HookItemType::Function(n),
        Err(e) => return TokenStream2::from(e.to_compile_error())
    }; 
    let args: AssemblyFunctionHook = match syn::parse2(input) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let mut transformer = Reloaded2CSharpHook::new();
    // Code generation
    let transformed = match transformer.codegen_rust(&mut target) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    transformed
}

pub fn riri_hook_inline_fn_build(input: TokenStream2, annotated_item: syn::ItemFn) -> syn::Result<HookBuildScriptResult> {
    let mut target = HookItemType::Function(annotated_item);
    let args= SourceFileEvaluationType::Inline(syn::parse2(input)?);
    let mut transformer = Reloaded2CSharpHook::new();
    let transformed = transformer.codegen_rust(&mut target)?;
    // parse back into items to inject into file
    Ok(HookBuildScriptResult {
        name: target.get_name(),
        items: HookFunctionBuildScriptItems::parse.parse2(transformed)?.to_items(),
        args
    })
}