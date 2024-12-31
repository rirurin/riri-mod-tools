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
    hook_parse::{
        CppClassMethods,
        StaticVarHook
    },
    riri_hook::HookItemType
};

// Code generators

pub(crate) trait HookFramework {
    fn codegen_rust_function(&mut self, f: &mut syn::ItemFn) -> syn::Result<TokenStream2>;
    fn codegen_rust_static(&mut self, s: &mut StaticVarHook) -> syn::Result<TokenStream2>;
    fn codegen_rust_class(&mut self, c: &mut syn::ItemStruct) -> syn::Result<TokenStream2>;
    fn codegen_rust(&mut self, h: &mut HookItemType) -> syn::Result<TokenStream2> {
        match h {
            HookItemType::Function(f) => self.codegen_rust_function(f),
            HookItemType::Static(s) => self.codegen_rust_static(s),
            HookItemType::CppClass(c) => self.codegen_rust_class(c)
        }
    }
}

pub fn make_dummy_item() -> syn::Item {
    syn::Item::Verbatim(TokenStream2::new())
}

pub struct Reloaded2CSharpHook {
    hook_set_name: MaybeUninit<syn::Ident>,
    r2_hooks_path: MaybeUninit<Box<std::path::PathBuf>>,
    parameters: Vec<String>,
    return_type: MaybeUninit<String>
}
impl Reloaded2CSharpHook {
    pub fn new() -> Self {
        Reloaded2CSharpHook {
            hook_set_name: MaybeUninit::uninit(),
            r2_hooks_path: MaybeUninit::uninit(),
            parameters: vec![], 
            return_type: MaybeUninit::uninit()
        }
    }
    pub const R2_INTEROP_CLASS: &'static str = "Reloaded2Interop";

    pub fn make_hook_set_string(s: &str) -> String { format!("__HOOK_SET_{}", s) }
    pub fn make_hook_og_fn_string(s: &str) -> String { format!("__HOOK_OGFN_{}", s) }
    pub fn make_vtbl_ptr_string(s: &str) -> String { format!("__VTBL_PTR_{}", s) }
}
impl Reloaded2CSharpHook {
    fn create_set_function(&mut self, ty: &TokenStream2, set_name: &str, hooked_name: &syn::Ident) -> TokenStream2 {
        self.hook_set_name.write(syn::Ident::new(Self::make_hook_set_string(set_name).as_str(), hooked_name.span()));
        let set_cell_fn_name = unsafe { self.hook_set_name.assume_init_ref() };
        let fn_target_abi = quote! { extern "C" };
        quote! {
            #[no_mangle]
            #[doc(hidden)]
            pub unsafe #fn_target_abi fn #set_cell_fn_name(cb: #ty) {
                let _ = #hooked_name.set(cb);
            }
        }
    }
    fn replace_original_function_unchecked(expr: &syn::ExprMacro, fn_name: &syn::Ident) -> Option<syn::Result<syn::Expr>> {
        if expr.mac.path.is_ident("original_function") {
            let arg_tokens = &expr.mac.tokens;
            Some(Ok(syn::Expr::parse.parse2(quote! { (#fn_name.get().unwrap())(#arg_tokens) }).unwrap()))
        } else {
            None
        }
    }
    fn replace_original_function(expr: &syn::ExprMacro, fn_name: &syn::Ident) -> Option<syn::Result<syn::Expr>> {
        if expr.mac.path.is_ident("original_function") {
            let args: Vec<syn::Ident> = 
                Punctuated::<syn::Ident, Token![,]>::parse_terminated
                .parse2(expr.mac.tokens.clone()).unwrap()
                .into_iter().collect();
            Some(Ok(syn::Expr::parse.parse2(quote! { (#fn_name.get().unwrap())(#(#args),*) }).unwrap()))
        } else {
            None
        }
    }
    fn check_expression_statement(expr: &mut syn::Expr, fn_name: &syn::Ident) -> Option<syn::Result<syn::Expr>> {
        match expr {
            syn::Expr::Macro(m) => match Self::replace_original_function_unchecked(&m, fn_name) {
                Some(v) => Some(v),
                None => None
            },
            _ => None
        }
    }
    fn get_type_name(ty: &syn::Type) -> syn::Result<TokenStream2> {
        match ty {
            syn::Type::Path(p) => Ok(p.path.to_token_stream()),
            syn::Type::Ptr(p) => {
                let p_p = if let Some(v) = &p.const_token {
                    quote! { *#v }
                } else if let Some(v) = &p.mutability {
                    quote! { *#v }
                } else {
                    return Err(syn::Error::new(p.span(), "Pointer type isn't const or mut (what?)"))
                };
                let p_ty = Self::get_type_name(&p.elem)?;
                Ok(quote! { #p_p #p_ty })
            },
            _ => Err(syn::Error::new(ty.span(), "Field type is not supported"))
        }
    }
}
impl HookFramework for Reloaded2CSharpHook {
    fn codegen_rust_function(&mut self, f: &mut syn::ItemFn) -> syn::Result<TokenStream2> {
        if f.sig.generics.params.len() > 0 {
            return Err(syn::Error::new(f.span(), "Generic type and lifetime arguments aren't supported for hookable functions"))
        }
        // create OnceCell to store original function
        let fn_name_upper = {
            let mut s = f.sig.ident.to_string();
            s.make_ascii_uppercase();
            s // we're naming globals, so make this uppercase
        };
        
        let ptr_fn_name = syn::Ident::new(Self::make_hook_og_fn_string(&fn_name_upper).as_str(), f.span());
        let fn_args: Vec<&syn::FnArg> = f.sig.inputs.borrow().into_iter().collect();
        let mut fn_args_tk: Vec<TokenStream2> = vec![];
        for fn_arg in fn_args {
            match fn_arg {
                syn::FnArg::Typed(t) => fn_args_tk.push(t.ty.to_token_stream()),
                syn::FnArg::Receiver(_) => return Err(syn::Error::new(f.span(), "Self argument isn't supported for hookable functions"))
            }
        }
        let fn_ret_tk = match &f.sig.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, t) => t.to_token_stream()
        };
        let fn_ty = quote! { extern "C" fn (#(#fn_args_tk),*) -> #fn_ret_tk };
        let fn_og_tk = quote! {
            #[doc(hidden)]
            pub static #ptr_fn_name: ::std::sync::OnceLock<#fn_ty> = ::std::sync::OnceLock::new();
        };
        let fn_set_og_tk = self.create_set_function(&fn_ty, &fn_name_upper, &ptr_fn_name);
        // add extern function to set cell, it it hasn't been overriden
        /*
        let set_cell_fn_name = unsafe { self.hook_set_name.assume_init_ref() };
        let fn_target_abi = quote! { extern "C" };
        let fn_set_og_tk = quote! {
            #[no_mangle]
            #[doc(hidden)]
            pub unsafe #fn_target_abi fn #set_cell_fn_name(cb: #fn_ty) {
                let _ = #ptr_fn_name.set(cb);
            }
        };
        */
        for stmt in f.block.stmts.iter_mut() {
            match stmt {
                syn::Stmt::Local(l) => {
                    match &mut l.init {
                        Some(i) => {
                            if let Some(v) = Self::check_expression_statement(&mut i.expr, &ptr_fn_name) {
                                *i.expr = v?;
                            }
                            if let Some(e) = &mut i.diverge {
                                if let Some(v) = Self::check_expression_statement(&mut e.1, &ptr_fn_name) {
                                    *e.1 = v?;
                                }
                            }
                        },
                        None => continue
                    }
                },
                syn::Stmt::Expr(e, t) => {
                    match Self::check_expression_statement(e, &ptr_fn_name) {
                        Some(v) => *e = v?,
                        None => continue
                    } 
                },
                _ => continue,
            }
        }
        let fk = f.to_token_stream();
        Ok(quote! {
            #fn_og_tk // ItemStatic
            #fn_set_og_tk // ItemFn
            #[no_mangle]
            #fk // ItemFn
        })
    }
    fn codegen_rust_static(&mut self, s: &mut StaticVarHook) -> syn::Result<TokenStream2> {
        // wrap the target type in a OnceLock that Reloaded's hook callback can set 
        let name_ident = &s.name;
        let type_ident = Self::get_type_name(&s.ty)?;
        let global_static = quote! { pub static #name_ident: ::std::sync::OnceLock<#type_ident> = ::std::sync::OnceLock::new(); };
        // expose function to Reloaded-II to set pointer
        let set_static_fn = self.create_set_function(&type_ident, name_ident.to_string().as_str(), name_ident);
        Ok(quote! {
            #global_static
            #set_static_fn
        })
    }
    fn codegen_rust_class(&mut self, c: &mut syn::ItemStruct) -> syn::Result<TokenStream2> {
        let class_name = c.ident.to_string();
        let class_name_ident = syn::Ident::new(&class_name, c.span());
        let class_name_upper = class_name.to_ascii_uppercase();
        // create new item for struct to store vtable OnceLock (it won't change after init, so this
        // is fine)
        let class_vtbl_str = Self::make_vtbl_ptr_string(&class_name_upper);
        let class_vtbl_name = syn::Ident::new(&class_vtbl_str, c.span());
        let vtable_ty = syn::Ident::new("usize", c.span());
        let class_vtbl_store = quote! { pub static #class_vtbl_name: ::std::sync::OnceLock<#vtable_ty> = ::std::sync::OnceLock::new(); };
        // expose function to Reloaded-II to set vtable pointer
        let class_vtbl_fn = self.create_set_function(&vtable_ty.to_token_stream(), &class_vtbl_str, &class_vtbl_name);
        // add cpp pointer
        match &mut c.fields {
            syn::Fields::Named(n) => {
                n.named.insert(0, syn::Field::parse_named
                    .parse2(quote! { #[doc(hidden)] __cpp_vtbl: #vtable_ty })?)
            },
            _ => return Err(syn::Error::new(c.span(), "Only supported fields are supported"))
        };
        let c_out = c.to_token_stream();
        Ok(quote! {
            #c_out
            #class_vtbl_store
            #class_vtbl_fn
        }) 
        // check functions inside of struct to check if they are associated with a vtable entry
        // hook each associated function, like in function codegen
    } 
}

#[doc(hidden)]
struct Reloaded2RustHooks;
// impl HookFramework for Reloaded2RustHooks { }

#[doc(hidden)]
struct Reloaded3;
// impl HookFramework for Reloaded3 { }


pub(crate) struct CppClassMethodGenerator;
impl CppClassMethodGenerator {
    pub fn codegen_rust(&self, im: &syn::ItemImpl, arg: &CppClassMethods) {

    }
    pub fn codegen_external(&self) {

    }
}
