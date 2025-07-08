use proc_macro2::TokenStream;
use std::{
    path::Path,
    process::Command
};
use quote::{ ToTokens, quote };

pub trait CheckStaticItemEquality {
    fn equal_to_static_item(&self, s: &syn::ItemStatic) -> bool;
}

impl CheckStaticItemEquality for usize {
    fn equal_to_static_item(&self, s: &syn::ItemStatic) -> bool {
        if let syn::Expr::Lit(l) = s.expr.as_ref() {
            if let syn::Lit::Int(i) = &l.lit {
                return i.base10_parse::<usize>().unwrap() == *self
            }
        }
        false       
    }
}

impl CheckStaticItemEquality for &str {
    fn equal_to_static_item(&self, s: &syn::ItemStatic) -> bool {
        if let syn::Expr::Lit(l) = s.expr.as_ref() {
            if let syn::Lit::Str(s) = &l.lit {
                return &s.value() == self
            }
        }
        false
    }
}

pub fn create_version_file<P>(base: P, r2_ver: &str) 
where P: AsRef<Path> {
    let commit_out = base.as_ref().join("src/version.rs");
    if let Some(commit_file_out) = create_version_file_object(&commit_out.as_path(), r2_ver) {
        std::fs::write(commit_out, commit_file_out.to_token_stream().to_string()).unwrap();
    }
}

pub fn create_version_file_object(commit_out: &Path, r2_ver: &str) -> Option<TokenStream> {
    let commit_ver = Command::new("git").args(["log", "--pretty=format:%H", "-n 1"])
        .output().unwrap();
    
    let commit_hash = unsafe { std::slice::from_raw_parts(commit_ver.stdout.as_ptr(), commit_ver.stdout.len()) };
    let commit_hash = unsafe { std::str::from_utf8_unchecked(commit_hash).trim() };

    let commit_list_cnt = Command::new("git").args(["rev-list", "--count", "--all"]).output().unwrap();
    let commit_count = unsafe { std::slice::from_raw_parts(commit_list_cnt.stdout.as_ptr(), commit_list_cnt.stdout.len()) };
    let commit_count = u32::from_str_radix(unsafe { std::str::from_utf8_unchecked(commit_count).trim() }, 10).unwrap() as usize;

    if std::fs::exists(commit_out).unwrap() {
        let version_old = std::fs::read_to_string(commit_out).unwrap();
        let version_syntax = syn::parse_file(&version_old).unwrap();
        let mut commit_count_is_same = false;
        let mut commit_hash_is_same = false;
        let mut reloaded_version_is_same = false;
        for item in &version_syntax.items {
            if let syn::Item::Static(s) = item {
                match s.ident.to_string().as_ref() {
                    "COMMIT_COUNT" => if commit_count.equal_to_static_item(s) { commit_count_is_same = true },
                    "COMMIT_HASH" => if commit_hash.equal_to_static_item(s) { commit_hash_is_same = true },
                    "RELOADED_VERSION" => if r2_ver.equal_to_static_item(s) { reloaded_version_is_same = true },
                    _ => ()
                }
            }
        }
        if commit_count_is_same && commit_hash_is_same && reloaded_version_is_same { return None; }
    }
    Some(quote! {

        use riri_mod_tools_proc::date_time;

        pub static COMMIT_COUNT: usize = #commit_count;
        pub static COMMIT_HASH: &'static str = #commit_hash;
        pub static RELOADED_VERSION: &'static str = #r2_ver;

        date_time!(COMPILE_DATE);
    })
}