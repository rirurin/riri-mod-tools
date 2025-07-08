use chrono::Utc;
use proc_macro2::TokenStream;
use quote::quote;

pub fn date_time_impl(input: TokenStream) -> TokenStream {
    let static_name = match syn::parse2::<syn::Ident>(input) {
        Ok(name) => name,
        Err(e) => return TokenStream::from(e.to_compile_error())
    };
    let current_date = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    quote! {
        pub static #static_name: &'static str = #current_date;
    }
}