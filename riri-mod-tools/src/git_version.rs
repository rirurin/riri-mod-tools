use chrono::Utc;
use proc_macro2::TokenStream;
use std::{
    path::Path,
    process::Command
};
use quote::{ ToTokens, quote };

pub fn create_version_file<P>(base: P, r2_ver: &str) 
where P: AsRef<Path> {
    let commit_out = base.as_ref().join("src/version.rs");
    let commit_file_out = create_version_file_object(r2_ver);
    std::fs::write(commit_out, commit_file_out.to_token_stream().to_string()).unwrap();
}

pub fn create_version_file_object(r2_ver: &str) -> TokenStream {
    let commit_ver = Command::new("git").args(["log", "--pretty=format:%H", "-n 1"])
        .output().unwrap();
    
    let commit_hash = unsafe { std::slice::from_raw_parts(commit_ver.stdout.as_ptr(), commit_ver.stdout.len()) };
    let commit_hash = unsafe { std::str::from_utf8_unchecked(commit_hash).trim() };

    let commit_list_cnt = Command::new("git").args(["rev-list", "--count", "--all"]).output().unwrap();
    let commit_count = unsafe { std::slice::from_raw_parts(commit_list_cnt.stdout.as_ptr(), commit_list_cnt.stdout.len()) };
    let commit_count = u32::from_str_radix(unsafe { std::str::from_utf8_unchecked(commit_count).trim() }, 10).unwrap() as usize;

    let compile_date = Utc::now().format("%Y-%m-%d %H:%M").to_string();

    quote! {
        pub static COMMIT_COUNT: usize = #commit_count;
        pub static COMMIT_HASH: &'static str = #commit_hash;
        pub static RELOADED_VERSION: &'static str = #r2_ver;
        pub static COMPILE_DATE: &'static str = #compile_date;
    }
}