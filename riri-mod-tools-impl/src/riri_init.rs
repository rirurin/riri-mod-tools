use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::riri_hook::{
    HookBuildScriptResult, 
    SourceFileInitializeFunction,
    SourceFileInitializeState,
    SourceFileEvaluationType
};
use proc_macro2::{
    Span,
    TokenStream as TokenStream2
};
use syn::{Item, ItemFn, parse::{Parse, Parser}};
use quote::{quote, ToTokens};

fn get_transformed_function(func: ItemFn) -> (String, TokenStream2) {
    let name = syn::Ident::new(func.sig.ident.to_string().as_ref(), Span::call_site());
    let block = &func.block;
    let fn_target_abi = quote! { extern "C" };
    let tokens = quote! {
        #[no_mangle]
        pub unsafe #fn_target_abi fn #name() {
            #block
        }
    };
    (name.to_string(), tokens)
}

// #[riri_init_fn]
pub fn riri_init_fn_impl(_input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    // Parse macro
    let target: ItemFn = match syn::parse2(annotated_item) {
        Ok(n) => n, Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let (_, res) = get_transformed_function(target);
    res
}

pub fn riri_init_fn_build(annotated_item: ItemFn) -> syn::Result<HookBuildScriptResult> {
    let (name, res) = get_transformed_function(annotated_item);
    Ok(HookBuildScriptResult {
        name: name.clone(), items: vec![Item::Fn(ItemFn::parse.parse2(res)?)],
        args: SourceFileEvaluationType::InitFunction(SourceFileInitializeFunction::new(
            name.clone(), SourceFileInitializeState::ModuleLoaded
        ))
    })
}

// #[riri_mods_loaded_fn]
pub fn riri_mods_loaded_fn_impl(_input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    // Parse macro
    let target: ItemFn = match syn::parse2(annotated_item) {
        Ok(n) => n, Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let (_, res) = get_transformed_function(target);
    res
}

pub fn riri_mods_loaded_fn_build(annotated_item: ItemFn) -> syn::Result<HookBuildScriptResult> {
    let (name, res) = get_transformed_function(annotated_item);
    Ok(HookBuildScriptResult {
        name: name.clone(), items: vec![Item::Fn(ItemFn::parse.parse2(res)?)],
        args: SourceFileEvaluationType::InitFunction(SourceFileInitializeFunction::new(
            name.clone(), SourceFileInitializeState::ModLoaderInitialized
        ))
    })
}

fn get_transformed_function_mod_loading(func: &ItemFn) -> (String, TokenStream2) {
    // Make link function to
    let inner_name = func.sig.ident.to_string();
    let link_func = syn::Ident::new(&format!("{}_LINK", func.sig.ident.to_string()), Span::call_site());
    let body_func = syn::Ident::new(inner_name.as_ref(), Span::call_site());
    let fn_target_abi = quote! { extern "C" };
    let link_tokens = quote! {
         #[no_mangle]
        pub unsafe #fn_target_abi fn #link_func(ptr: isize) {
            let mod_config = riri_mod_tools_rt::reloaded::r#mod::interfaces::IModConfig::new_unchecked(riri_mod_tools_rt::system::Object::new_unchecked(ptr as usize));
            #body_func(mod_config);
        }
    };
    (link_func.to_string(), link_tokens)
}

#[derive(Debug)]
pub struct ModLoadingFunctionSignatureError;
impl Error for ModLoadingFunctionSignatureError {}

impl Display for ModLoadingFunctionSignatureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

// #[riri_mod_loading_fn]
pub fn riri_mod_loading_fn_impl(_input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    let target: ItemFn = match syn::parse2(annotated_item) {
        Ok(n) => n, Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let args: Vec<&syn::FnArg> = target.sig.inputs.iter().collect();
    let err = TokenStream2::from(syn::Error::new(Span::call_site(), ModLoadingFunctionSignatureError).to_compile_error());
    if args.len() != 1 { return err }
    if let syn::FnArg::Typed(ty) = args[0] {
        if let syn::Type::Path(p) = ty.ty.as_ref() {
            if let Some(ident) = p.path.get_ident() {
                if ident.to_string() != "IModConfig" { return err; }
            } else { return err }
        } else { return err }
    } else { return err }
    let (_, res) = get_transformed_function_mod_loading(&target);
    let target_tk = target.to_token_stream();
    let out = quote! {
        #res
        #target_tk
    };
    println!("{}", out);
    out
}

pub fn riri_mod_loading_fn_build(annotated_item: ItemFn) -> syn::Result<HookBuildScriptResult> {
    let (name, res) = get_transformed_function_mod_loading(&annotated_item);
    Ok(HookBuildScriptResult {
        name: name.clone(), items: vec![Item::Fn(ItemFn::parse.parse2(res)?), Item::Fn(annotated_item)],
        args: SourceFileEvaluationType::InitFunction(SourceFileInitializeFunction::new(
            name.clone(), SourceFileInitializeState::ModLoaded
        ))
    })
}