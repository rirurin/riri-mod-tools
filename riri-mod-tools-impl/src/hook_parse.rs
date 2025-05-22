#![allow(dead_code, unused_imports, unused_mut, unused_variables)]
use proc_macro2::{
    Span as Span2,
    TokenStream as TokenStream2
};
use quote::ToTokens;
use std::{
    borrow::{ Borrow, BorrowMut },
    fmt::Display,
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
pub struct StaticOffset(pub usize);

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
        if ret_sig.is_none() {
            if let Some(s) = shared_scan.as_ref() {
                if *s != RyoTuneSharedScan::Consume {
                    return Err(syn::Error::new(entry.span(), "Signature field is required"));   
                }
            } else {
                return Err(syn::Error::new(entry.span(), "Signature field is required"));   
            }
        }
        if call_conv == None {
            // Atlus defaults to using Visual C++ compiler, so default to that
            call_conv = Some(CallingConvention::Microsoft);
        }
        let sig = match ret_sig {
            Some(v) => v.0,
            None => "".to_string()
        };
        Ok(DynamicOffset {
            sig,
            resolve_type: if resolve_type.is_some() { Some(resolve_type.unwrap().0) } else { None },
            call_conv: call_conv.unwrap(),
            shared_scan
        })
    }
}

#[derive(Debug)]
pub enum HookEntry {
    Static(StaticOffset),
    Dyn(DynamicOffset),
    Delayed
}

impl HookEntry {
    pub fn new(entry: &syn::ExprCall) -> syn::Result<Self> {
        match entry.func.borrow() {
            syn::Expr::Path(v) => {
                match v.path.require_ident()?.to_string().as_str() {
                    "dynamic_offset" => Ok(HookEntry::Dyn(DynamicOffset::from_expr_call(&entry)?)),
                    "static_offset" => Ok(HookEntry::Static(StaticOffset::from_expr_call(&entry)?)),
                    "user_defined" => Ok(HookEntry::Delayed),
                    _ => return Err(syn::Error::new(entry.span(), "Unknown entry name (should be static_offset, dynamic_offset or user_defined)"))
                }
            },
            _ => return Err(syn::Error::new(entry.span(), "Should be a name"))
        }
    }
}

#[derive(Debug)]
pub enum HookConditional {
    // single entry
    None,
    // multiple entries
    HashNamed(String),
    HashNum(u64),
    Default
}

impl Display for HookConditional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let disp = match self {
            Self::None | Self::Default => "".to_owned(),
            Self::HashNamed(name) => name.clone(),
            Self::HashNum(v) => format!("{}", v)
        };
        write!(f, "{}", disp)
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

// #[derive(Debug)]
pub struct HookInfoMultipleCaseCollector {
    arms: Vec<syn::Arm>,
    has_default_arm: bool,
}

impl HookInfoMultipleCaseCollector {
    pub fn new(entries: syn::parse::ParseBuffer<'_>) -> syn::Result<Self> {
        let mut arms: Vec<syn::Arm> = vec![];
        while !entries.is_empty() {
            arms.push(entries.parse()?);
        }
        Ok(Self { arms, has_default_arm: false })
    }

    pub fn collect_patterns(&mut self) -> syn::Result<Vec<(HookConditional, HookEntry)>> {
        let mut patterns: Vec<(HookConditional, HookEntry)> = vec![];
        for arm in &self.arms {
            if self.has_default_arm {
                return Err(syn::Error::new(arm.span(), "Default arm must be the last arm"));
            }
            let entry = match arm.body.borrow() {
                syn::Expr::Call(c) => c,
                _ => return Err(syn::Error::new(arm.span(), "Invalid hook entry format"))
            };
            let mut has_default_arm = self.has_default_arm;
            patterns.extend(self.collect_patterns_inner(&arm.pat, entry, &mut has_default_arm)?);
            if has_default_arm { self.has_default_arm = has_default_arm }
        }
        Ok(patterns)
    }

    fn collect_patterns_inner(&self, pat: &syn::Pat, entry: &syn::ExprCall, default_arm: &mut bool) -> syn::Result<Vec<(HookConditional, HookEntry)>> {
        if *default_arm {
            return Err(syn::Error::new(Span2::call_site(), "Default arm must be the last arm"));
        }
        match pat {
            syn::Pat::Ident(v) => Ok(vec![(HookConditional::HashNamed(v.ident.to_string()), HookEntry::new(entry)?)]),
            syn::Pat::Lit(l) => {
                if let syn::Lit::Int(i) = &l.lit {
                    Ok(vec![(HookConditional::HashNum(i.base10_parse::<u64>()?), HookEntry::new(entry)?)])
                } else {
                    Err(syn::Error::new(Span2::call_site(), "Only integer literals are allowed"))
                }
            },
            syn::Pat::Or(o) => {
                let mut vars = vec![];
                for c in &o.cases { 
                    vars.extend(self.collect_patterns_inner(c, entry, default_arm)?);
                }
                // println!("{:?}", vars);
                // panic!("OR WAS CALLED!");
                Ok(vars)
            },
            syn::Pat::Wild(w) => {
                *default_arm = true;
                Ok(vec![(HookConditional::Default, HookEntry::new(entry)?)])
            },
            _ => Err(syn::Error::new(Span2::call_site(), "Unimplemented match pattern"))
        }
    }
}

impl Parse for HookInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // #[riri_hook_fn(static_offset(...))]
        // or
        // #[riri_hook_fn({
        //    XRD759_STEAM_102 => static_offset(...),
        //    XRD759_STEAM_103 => static_offset(...),
        //    _ => dynamic_offset(...)
        // })]
        //
        let mut hook_var: Vec<(HookConditional, HookEntry)> = vec![];
        if input.peek(syn::Ident) && input.peek2(syn::token::Paren) { // one entry only
            let entry = syn::ExprCall::parse(input)?;
            hook_var.push((HookConditional::None, HookEntry::new(&entry)?));
        } else if input.peek(syn::token::Brace) { // multiple entries
            let entries;
            syn::braced!(entries in input);
            let mut collector = HookInfoMultipleCaseCollector::new(entries)?;
            hook_var.extend(collector.collect_patterns()?);
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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AsmHookExecuteBehavior {
    ///  Executes your assembly code before the original.
    ExecuteFirst,
    /// Executes your assembly code after the original.
    ExecuteAfter,
    /// Do not execute original replaced code. (Dangerous!)
    DoNotExecuteOriginal
}

impl TryFrom<String> for AsmHookExecuteBehavior {
    type Error = syn::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "ExecuteFirst" => Ok(Self::ExecuteFirst),
            "ExecuteAfter" => Ok(Self::ExecuteAfter),
            "DoNotExecuteOriginal" => Ok(Self::DoNotExecuteOriginal),
            _ => Err(syn::Error::new(
                Span2::call_site(),
                &format!("Unknown assembly execute method {}", &value)
            ))
        }
    }
}

impl From<AsmHookExecuteBehavior> for &'static str {
    fn from(value: AsmHookExecuteBehavior) -> Self {
        match value {
            AsmHookExecuteBehavior::ExecuteFirst => "ExecuteFirst",
            AsmHookExecuteBehavior::ExecuteAfter => "ExecuteAfter",
            AsmHookExecuteBehavior::DoNotExecuteOriginal => "DoNotExecuteOriginal",
        }
    }
}

impl Display for AsmHookExecuteBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: &str = (*self).into();
        write!(f, "{}", v)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum RegistersX86 {
    rax, rbx, rcx, rdx, 
    rsi, rdi, rbp, rsp, 
    r8, r9, r10, r11,
    r12, r13, r14, r15
}

impl TryFrom<String> for RegistersX86 {
    type Error = syn::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "rax" => Ok(Self::rax),
            "rbx" => Ok(Self::rbx),
            "rcx" => Ok(Self::rcx),
            "rdx" => Ok(Self::rdx),
            "rsi" => Ok(Self::rsi),
            "rdi" => Ok(Self::rdi),
            "rbp" => Ok(Self::rbp),
            "rsp" => Ok(Self::rsp),
            "r8" => Ok(Self::r8),
            "r9" => Ok(Self::r9),
            "r10" => Ok(Self::r10),
            "r11" => Ok(Self::r11),
            "r12" => Ok(Self::r12),
            "r13" => Ok(Self::r13),
            "r14" => Ok(Self::r14),
            "r15" => Ok(Self::r15),
            _ => Err(syn::Error::new(
                Span2::call_site(),
                &format!("Unknown register {}", &value)
            ))
        }
    }
}

impl From<RegistersX86> for &'static str {
    fn from(value: RegistersX86) -> Self {
        match value {
            RegistersX86::rax => "rax",
            RegistersX86::rbx => "rbx",
            RegistersX86::rcx => "rcx",
            RegistersX86::rdx => "rdx",
            RegistersX86::rsi => "rsi",
            RegistersX86::rdi => "rdi",
            RegistersX86::rbp => "rbp",
            RegistersX86::rsp => "rsp",
            RegistersX86::r8 => "r8",
            RegistersX86::r9 => "r9",
            RegistersX86::r10 => "r10",
            RegistersX86::r11 => "r11",
            RegistersX86::r12 => "r12",
            RegistersX86::r13 => "r13",
            RegistersX86::r14 => "r14",
            RegistersX86::r15 => "r15",
        }
    }
}

impl Display for RegistersX86 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: &str = (*self).into();
        write!(f, "{}", v)
    }
}

//
// #[riri_hook_inline_fn(
//      static_offset(0xa150), // static/dynamic offset
//      count must match up with signature count
//      [
//          {
//              ExecuteFirst, // hook execution order
//              [r8, r9, r15], rax, // parameter registers + return register
//              false, // allocate shadow space
//              [], // callee saved registers
//              "sub rsp, 0x40", // insert custom assembly before
//              "add rsp, 0x40" // insert custom assembly after
//          },
//          {
//              ...
//          }
//      ]
// )]
//
//
#[derive(Debug)]
pub struct AssemblyFunctionHook {
    pub hook_info: HookInfo,
    pub data: Vec<AssemblyFunctionHookData>
}

impl AssemblyFunctionHook {
    pub(crate) fn is_user_defined_init(&self) -> bool {
        self.hook_info.is_user_defined_init()
    }
}

impl Parse for AssemblyFunctionHook {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let hook_info: HookInfo = input.parse()?;
        let mut asm_param = 0;
        let _: Token![,] = input.parse()?;
        let asm_stream;
        syn::bracketed!(asm_stream in input);
        let asm_entries = asm_stream.parse_terminated(AssemblyFunctionHookData::parse, Token![,])?;
        if asm_entries.len() != hook_info.0.len() {
            return Err(syn::Error::new(input.span(), "Assembly entry data array count should match signature array length"));
        }
        let data: Vec<AssemblyFunctionHookData> = asm_entries.into_iter().collect();
        Ok(Self { hook_info, data })
    }
}
#[derive(Debug)]
pub struct AssemblyFunctionHookData {
    pub execute_mode: AsmHookExecuteBehavior,
    // last one is return register
    pub registers: Vec<RegistersX86>,
    pub callee_saved_registers: Vec<RegistersX86>,
    pub allocate_shadow_space: bool,
    pub asm_insert_before: Option<String>,
    pub asm_insert_after: Option<String>
}

impl Parse for AssemblyFunctionHookData {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut asm_data_index = 0;
        let mut execute_mode = AsmHookExecuteBehavior::ExecuteFirst;
        let mut registers = vec![];
        let mut callee_saved_registers = vec![];
        let mut allocate_shadow_space = false;
        let mut asm_insert_before = None;
        let mut asm_insert_after = None;
        let asm_fields;
        syn::braced!(asm_fields in input);
        while !asm_fields.is_empty() {
            match asm_data_index {
                0 => { // execute mode
                    let execute_ident: syn::Ident = asm_fields.parse()?;
                    execute_mode = execute_ident.to_string().try_into()?;
                },
                1 => { // parameter registers
                    let reg_stream;
                    syn::bracketed!(reg_stream in asm_fields);
                    let regs = reg_stream.parse_terminated(
                        syn::Ident::parse, Token![,])?;
                    registers = Vec::with_capacity(regs.len());
                    for reg in regs {
                        registers.push(reg.to_string().try_into()?);
                    }
                },
                2 => { // return register
                    let ret_reg: syn::Ident = asm_fields.parse()?;
                    registers.push(ret_reg.to_string().try_into()?);
                },
                3 => { // callee saved registers
                    let reg_stream;
                    syn::bracketed!(reg_stream in asm_fields);
                    let regs = reg_stream.parse_terminated(
                        syn::Ident::parse, Token![,])?;
                    callee_saved_registers = Vec::with_capacity(regs.len());
                    for reg in regs {
                        callee_saved_registers.push(reg.to_string().try_into()?);
                    }
                },
                4 => { // allocate shadow space
                    let lit = syn::Lit::parse(&asm_fields)?;
                    if let syn::Lit::Bool(b) = lit {
                        allocate_shadow_space = b.value;
                    } else {
                        return Err(syn::Error::new(asm_fields.span(), "Boolean value must be used to toggle shadow space"))
                    }
                },
                5 => { // inline assembly before
                    if asm_fields.peek(syn::Ident) {
                        let ident = syn::Ident::parse(&asm_fields)?;
                        if &ident.to_string() != "None" {
                            return Err(syn::Error::new(asm_fields.span(), "Inline assembly must be \"None\" or a string"))
                        }
                    } else {
                        let lit = syn::Lit::parse(&asm_fields)?;
                        if let syn::Lit::Str(s) = lit {
                            asm_insert_before = Some(s.value());
                        } else {
                            return Err(syn::Error::new(asm_fields.span(), "String value must be used to define inline assembly"))
                        }
                    }
                },
                6 => { // inline assembly after
                    if asm_fields.peek(syn::Ident) {
                        let ident = syn::Ident::parse(&asm_fields)?;
                        if &ident.to_string() != "None" {
                            return Err(syn::Error::new(asm_fields.span(), "Inline assembly must be \"None\" or a string"))
                        }
                    } else {
                        let lit = syn::Lit::parse(&asm_fields)?;
                        if let syn::Lit::Str(s) = lit {
                            asm_insert_after = Some(s.value());
                        } else {
                            return Err(syn::Error::new(asm_fields.span(), "String value must be used to define inline assembly"))
                        }
                    }
                },
                _ => {
                    return Err(syn::Error::new(asm_fields.span(), "Too many assembly parameters"));
                }
            };
            asm_data_index += 1;
            if asm_fields.peek(Token![,]) {
                let _: Token![,] = asm_fields.parse()?;
            } else { break; }
        }
        if asm_data_index < 3 {
            return Err(syn::Error::new(asm_fields.span(), "Missing required parameters: Execute mode and registers"));
        }
        Ok(Self {
            execute_mode,
            registers,
            callee_saved_registers,
            allocate_shadow_space,
            asm_insert_before,
            asm_insert_after,
        })
    }
}
