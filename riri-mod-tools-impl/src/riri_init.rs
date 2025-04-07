use crate::riri_hook::{ HookBuildScriptResult, SourceFileEvaluationType };
use proc_macro2::{
    Span,
    TokenStream as TokenStream2
};
use syn::{
    Item, ItemFn,
    parse::{ Parse, Parser }
};
use quote::quote;

// pub struct InitFunctionParameters {
//     name: String,
// }

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
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let (_, res) = get_transformed_function(target);
    res
}

pub fn riri_init_fn_build(annotated_item: ItemFn) -> syn::Result<HookBuildScriptResult> {
    let (name, res) = get_transformed_function(annotated_item);
    Ok(HookBuildScriptResult {
        name: name.clone(), items: vec![Item::Fn(ItemFn::parse.parse2(res).unwrap())],
        args: SourceFileEvaluationType::InitFunction(name.clone())
    })
}