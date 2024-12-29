#![allow(dead_code, unused_imports, unused_mut, unused_variables)]
use proc_macro2::{
    Span as Span2,
    TokenStream as TokenStream2
};
use quote::ToTokens;
use std::{
    borrow::{ Borrow, BorrowMut },
    mem::MaybeUninit
};
use syn::{
    parse::{ Parse, Parser, ParseStream },
    punctuated::Punctuated,
    spanned::Spanned,
    Token
};

use crate::riri_hook::HookInfo;

pub(crate) trait HookInfoParam {
    fn get_param(e: &syn::ExprAssign) -> syn::Result<Self> where Self: Sized;
}

pub(crate) struct HookParseTools;
impl HookParseTools {
    pub fn get_parameter_name(e: &syn::ExprAssign) -> syn::Result<&syn::Path> {
        match e.left.borrow() {
            syn::Expr::Path(p) => Ok(&p.path),
            _ => Err(syn::Error::new(e.span(), "LHS of assignment should be a valid field name"))
        }
    }
    pub fn get_single_param<T>(expr: &syn::ExprAssign, parm: &mut Option<T>, name: &syn::Path) 
        -> syn::Result<()> 
        where T: PartialEq + HookInfoParam
    {
        if *parm != None {
            return Err(syn::Error::new(name.span(), format!("{} was already defined", name.get_ident().unwrap())));
        }
        *parm = Some(T::get_param(expr)?);
        Ok(())
    }
}

// Hook static/dynamic parameter parsing

#[derive(Debug, Clone, PartialEq)]
struct DataSignature(String);

impl HookInfoParam for DataSignature {
    fn get_param(e: &syn::ExprAssign) -> syn::Result<Self> where Self: Sized {
        if let syn::Expr::Lit(l) = e.right.borrow() {
            if let syn::Lit::Str(s) = &l.lit {
                return Ok(Self(s.value()));
            }
        }
        Err(syn::Error::new(e.span(), "Invalid value for signature - should be a String"))       
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ResolveType(String);

impl HookInfoParam for ResolveType {
    fn get_param(e: &syn::ExprAssign) -> syn::Result<Self> where Self: Sized {
        // to work with Reloaded-II, it's required that the function is a path with a single
        // segment, either pointing to a built in resolver or to a custom function defined inside
        // of the same module, annotated with extern "C" + #[no_mangle]
        // (i'm sorry that this is so cursed)
        if let syn::Expr::Path(p) = e.right.borrow() {
            let seg = &p.path.segments;
            if seg.len() > 1 {
                return Err(syn::Error::new(seg.span(), "Only single segment paths are supported"));
            }
            return Ok(Self(seg[0].ident.to_string()));
        }
        Err(syn::Error::new(e.span(), "Invalid value for resolve_type - should be a String"))       
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CallingConvention {
    Microsoft,
    Unknown
}

impl TryFrom<&str> for CallingConvention {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "microsoft" => Ok(CallingConvention::Microsoft),
            "unknown" => Ok(CallingConvention::Unknown),
            _ => Err(())
        }
    }
}

impl HookInfoParam for CallingConvention {
    fn get_param(e: &syn::ExprAssign) -> syn::Result<Self> where Self: Sized {
        if let syn::Expr::Lit(l) = e.right.borrow() {
            if let syn::Lit::Str(s) = &l.lit {
                return match CallingConvention::try_from(s.value().as_str()) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(syn::Error::new(e.span(), "Unimplemented calling convention was provided"))
                }
            }
        }
        Err(syn::Error::new(e.span(), "Invalid value for calling convention - should be a String"))       
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RyoTuneSharedScan {
    Produce,
    Consume
}

impl TryFrom<&str> for RyoTuneSharedScan {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "producer" => Ok(RyoTuneSharedScan::Produce),
            "consumer" => Ok(RyoTuneSharedScan::Consume),
            _ => Err(())
        }
    }
}

impl HookInfoParam for RyoTuneSharedScan {
    fn get_param(e: &syn::ExprAssign) -> syn::Result<Self> where Self: Sized {
        if let syn::Expr::Lit(l) = e.right.borrow() {
            if let syn::Lit::Str(s) = &l.lit {
                return match RyoTuneSharedScan::try_from(s.value().as_str()) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(syn::Error::new(e.span(), "Unimplemented shared scan was provided"))
                }
            }
        }
        Err(syn::Error::new(e.span(), "Invalid value for shared scan - should be a String"))
    }
}

trait OffsetBuilder {
    fn from_expr_call(entry: &syn::ExprCall) -> syn::Result<Self> where Self : Sized;
}
/// Represents a static point of interest within the program module. It's not recommended to use
/// this as will very likely break between versions, and the compiler will give a warning on debug
/// builds and error on release builds when using this.
/// (Update 12/11/2024: you can't actually do this :naosmiley: There's an RFC proposing user
/// generated diagnostics https://github.com/rust-lang/rust/issues/54140 , but this feature is only
/// available at nightly *and in proc_macro*.)
#[derive(Clone, Copy, Debug)]
pub struct StaticOffset(usize);

impl OffsetBuilder for StaticOffset {
    fn from_expr_call(entry: &syn::ExprCall) -> syn::Result<Self> where Self : Sized {
        if entry.args.len() != 1 {
            return Err(syn::Error::new(entry.span(), "Incorrect argument count for static offset"));
        }
        let offset = match &entry.args[0] {
            syn::Expr::Lit(l) => match &l.lit {
                syn::Lit::Int(v) => v.base10_parse::<usize>().unwrap(),
                _ => return Err(syn::Error::new(entry.span(), "Argument should be a constant number"))
            },
            _ => return Err(syn::Error::new(entry.span(), "Argument must be a literal"))
        };
        Ok(StaticOffset(offset))
    }
}

#[derive(Debug)]
pub struct DynamicOffset {
    pub sig: String,
    pub resolve_type: Option<String>,
    pub call_conv: CallingConvention,
    pub shared_scan: Option<RyoTuneSharedScan>
}

impl OffsetBuilder for DynamicOffset {
    fn from_expr_call(entry: &syn::ExprCall) -> syn::Result<Self> where Self: Sized {
        // treat as dynamic for now
        let mut ret_sig: Option<DataSignature> = None; // required
        let mut resolve_type: Option<ResolveType> = None;
        let mut call_conv: Option<CallingConvention> = None; // optional, default to microsoft
        let mut shared_scan: Option<RyoTuneSharedScan> = None; // optional (None = not shared)
        for arg in &entry.args {
            if let syn::Expr::Assign(v) = arg {
                let carg = HookParseTools::get_parameter_name(v)?;
                // signature = "48 83 ??"
                if carg.is_ident("signature") {
                    HookParseTools::get_single_param(v, &mut ret_sig, carg)?;
                // resolve_type = "get_address_may_thunk"
                } else if carg.is_ident("resolve_type") {
                    HookParseTools::get_single_param(v, &mut resolve_type, carg)?;
                // calling_convention = "microsoft"
                } else if carg.is_ident("calling_convention") {
                    HookParseTools::get_single_param(v, &mut call_conv, carg)?;
                // shared_scan = "producer"
                } else if carg.is_ident("shared_scan") {
                    HookParseTools::get_single_param(v, &mut shared_scan, carg)?;
                } else {
                    return Err(syn::Error::new(arg.span(), "Unsupported argument name"))
                }
            }
        }
        // get default arguments
        if ret_sig == None {
            return Err(syn::Error::new(entry.span(), "Signature field is required"));
        }
        if call_conv == None {
            // Atlus defaults to using Visual C++ compiler, so default to that
            call_conv = Some(CallingConvention::Microsoft);
        }
        Ok(DynamicOffset {
            sig: ret_sig.unwrap().0,
            resolve_type: if resolve_type.is_some() { Some(resolve_type.unwrap().0) } else { None },
            call_conv: call_conv.unwrap(),
            shared_scan
        })
    }
}

#[derive(Debug)]
pub enum HookEntry {
    Static(StaticOffset),
    Dyn(DynamicOffset)
}

impl HookEntry {
    pub fn new(entry: &syn::ExprCall) -> syn::Result<Self> {
        match entry.func.borrow() {
            syn::Expr::Path(v) => {
                match v.path.require_ident()?.to_string().as_str() {
                    "dynamic_offset" => Ok(HookEntry::Dyn(DynamicOffset::from_expr_call(&entry)?)),
                    "static_offset" => Ok(HookEntry::Static(StaticOffset::from_expr_call(&entry)?)),
                    _ => return Err(syn::Error::new(entry.span(), "Unknown entry name (should be static_offset or dynamic_offset)"))
                }
            },
            _ => return Err(syn::Error::new(entry.span(), "Should be a name"))
        }
    }
}

pub(crate) struct StaticVarHook {
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl Parse for StaticVarHook {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_token: syn::Ident = input.parse()?;
        let _comma_token: Token![,] = input.parse()?;
        let type_token: syn::Type = input.parse()?;
        if input.peek(Token![,]) {
        }
        Ok(StaticVarHook {
            name: name_token,
            ty: type_token
        })
    }
}

impl ToTokens for StaticVarHook {
    fn to_tokens(&self, tokens: &mut TokenStream2) {}
}

impl Parse for HookInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // #[riri_hook_fn(static_offset(...))]
        // or
        // #[riri_hook_fn(XRD759_STEAM_102 => static_offset())]
        // or
        // #[riri_hook_fn(1 => static_offset(), _ => static_offset())]
        //
        let mut hook_var: Vec<HookEntry> = vec![];
        if input.peek(syn::Ident) { 
            if input.peek2(syn::token::Paren) { // one entry only
                let entry = syn::ExprCall::parse(input)?;
                hook_var.push(HookEntry::new(&entry)?);
            } else if input.peek2(Token![=>]) { // multiple entries
                let arms = Punctuated::<syn::Arm, Token![,]>::parse_terminated(input)?;
                let mut default_arm = false;
                for arm in &arms {
                    let entry = match arm.body.borrow() {
                        syn::Expr::Call(c) => c,
                        _ => return Err(syn::Error::new(arm.span(), "Invalid hook entry format"))
                    };
                    match &arm.pat {
                        syn::Pat::Ident(v) => hook_var.push(HookEntry::new(&entry)?),
                        _ => return Err(syn::Error::new(arm.span(), "Unimplemented match pattern"))
                    }
                }
            }
        } else {
            return Err(syn::Error::new(input.span(), "Invalid macro structure (should be either like a function call or match arms)"))
        }
        Ok( HookInfo::new(hook_var) )
    }
}

// For implement block for cpp class

#[derive(Debug, Clone, PartialEq)]
struct CppClassPath(String);
impl HookInfoParam for CppClassPath {
    fn get_param(e: &syn::ExprAssign) -> syn::Result<Self> where Self: Sized {
        if let syn::Expr::Lit(l) = e.right.borrow() {
            if let syn::Lit::Str(s) = &l.lit {
                return Ok(Self(s.value()))
            }
        }
        Err(syn::Error::new(e.span(), "Invalid assignment for path"))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CppClassDropIndex(usize);
impl HookInfoParam for CppClassDropIndex {
    fn get_param(e: &syn::ExprAssign) -> syn::Result<Self> where Self: Sized {
        if let syn::Expr::Lit(l) = e.right.borrow() {
            if let syn::Lit::Int(i) = &l.lit {
                return Ok(Self(i.base10_parse::<usize>()?))
            }
        }
        Err(syn::Error::new(e.span(), "Invalid assignment for auto_drop"))       
    }
}

pub(crate) struct CppClassMethods {
    path: Option<CppClassPath>,
    auto_drop: Option<CppClassDropIndex>
}

impl Parse for CppClassMethods {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let call = syn::ExprCall::parse(input)?;
        let mut path: Option<CppClassPath> = None;
        let mut auto_drop: Option<CppClassDropIndex> = None;
        for arg in &call.args {
            match arg {
                syn::Expr::Assign(v) => {
                    if let syn::Expr::Path(p) = v.left.borrow() {
                        if p.path.is_ident("path") {
                            HookParseTools::get_single_param(v, &mut path, &p.path)?;
                        } else if p.path.is_ident("auto_drop") {
                            HookParseTools::get_single_param(v, &mut auto_drop, &p.path)?;
                        } else {
                            return Err(syn::Error::new(arg.span(), "Unimplemented argument"));
                        }
                    }
                    continue;
                },
                _ => {
                    return Err(syn::Error::new(arg.span(), "Parameters should be assignments only"));
                }
            }
        }
        Ok(CppClassMethods{ path, auto_drop })
    }
}
