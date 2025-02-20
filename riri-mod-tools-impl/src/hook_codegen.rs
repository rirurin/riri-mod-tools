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
        Self::replace_original_function_unchecked_inner(&expr.mac, fn_name)
    }
    fn replace_original_function_unchecked_inner(mac: &syn::Macro, fn_name: &syn::Ident) -> Option<syn::Result<syn::Expr>> {
        if mac.path.is_ident("original_function") {
            let arg_tokens = &mac.tokens;
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

    fn traverse_expression(expr: &mut syn::Expr, fn_name: &syn::Ident) {
        match expr {
            // Direct invocation
            syn::Expr::Macro(m) => {
                match Self::replace_original_function_unchecked(&m, fn_name) {
                    Some(r) => match r {
                        Ok(v) => *expr = v,
                        Err(_) => ()
                    },
                    None => ()
                }
            },
            // blocked scope
            syn::Expr::Block(b) => Self::traverse_statements(&mut b.block.stmts, fn_name),
            // unsafe blocked scope
            syn::Expr::Unsafe(u) => Self::traverse_statements(&mut u.block.stmts, fn_name),
            // binary operations (e.g arithmetic)
            syn::Expr::Binary(b) => {
                Self::traverse_expression(&mut b.left, fn_name);
                Self::traverse_expression(&mut b.right, fn_name);
            },
            // Cast operation
            syn::Expr::Cast(c) => Self::traverse_expression(&mut c.expr, fn_name),
            // Parenthesized expressions
            syn::Expr::Paren(p) => Self::traverse_expression(&mut p.expr, fn_name),
            syn::Expr::Tuple(t) => {
                for elem in &mut t.elems {
                    Self::traverse_expression(elem, fn_name);
                }
            },
            // Unary operation (e.g dereferencing)
            syn::Expr::Unary(u) => Self::traverse_expression(&mut u.expr, fn_name),
            // Address-of
            syn::Expr::RawAddr(r) => Self::traverse_expression(&mut r.expr, fn_name),
            // Reference
            syn::Expr::Reference(r) => Self::traverse_expression(&mut r.expr, fn_name),
            // If-then-else
            syn::Expr::If(i) => {
                Self::traverse_expression(&mut i.cond, fn_name);
                Self::traverse_statements(&mut i.then_branch.stmts, fn_name);
                if i.else_branch.is_some() {
                    Self::traverse_expression(i.else_branch.as_mut().unwrap().1.as_mut(), fn_name);
                }
            },
            // For loop
            syn::Expr::ForLoop(f) => {
                Self::traverse_expression(&mut f.expr, fn_name);
                Self::traverse_statements(&mut f.body.stmts, fn_name);
            },
            // Conditinless loop
            syn::Expr::Loop(l) => {
                Self::traverse_statements(&mut l.body.stmts, fn_name);
            },
            // While loop
            syn::Expr::While(w) => {
                Self::traverse_expression(&mut w.cond, fn_name);
                Self::traverse_statements(&mut w.body.stmts, fn_name);
            },
            // Function call
            syn::Expr::Call(c) => {
                for arg in &mut c.args {
                    Self::traverse_expression(arg, fn_name);
                }
            },
            // Method call
            syn::Expr::MethodCall(m) => {
                for arg in &mut m.args {
                    Self::traverse_expression(arg, fn_name);
                }
            },
            // Match
            syn::Expr::Match(m) => {
                Self::traverse_expression(&mut m.expr, fn_name);
                for arm in &mut m.arms {
                    if arm.guard.is_some() {
                        Self::traverse_expression(arm.guard.as_mut().unwrap().1.as_mut(), fn_name);
                    }
                    Self::traverse_expression(arm.body.as_mut(), fn_name);
                }
            }
            _ => ()
        }
    }

    // Search statement list for invocations of original_function! and replace it with our hooked
    // pointer
    fn traverse_statements(stmts: &mut Vec<syn::Stmt>, fn_name: &syn::Ident) {
        for stmt in stmts {
            match stmt {
                syn::Stmt::Local(l) => {
                    match &mut l.init {
                        Some(v) => {
                            Self::traverse_expression(&mut v.expr, fn_name);
                            if v.diverge.is_some() {
                                Self::traverse_expression(v.diverge.as_mut().unwrap().1.as_mut(), fn_name);
                            }
                        },
                        None => continue
                    }
                },
                syn::Stmt::Expr(e, _) => Self::traverse_expression(e, fn_name),
                _ => continue
            }
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
        if f.sig.abi.is_none() || &f.sig.abi.as_ref().unwrap().name.as_ref().unwrap().value() != "C" {
            return Err(syn::Error::new(f.span(), "Hookable function must be defined with C ABI: extern \"C\""))
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
        Self::traverse_statements(&mut f.block.stmts, &ptr_fn_name);
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
