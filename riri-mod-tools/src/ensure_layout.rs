use syn::{
    self,
    parse::{ Parse, Parser },
    spanned::Spanned
};
use riri_mod_tools_impl::ensure_layout::EnsureLayout;

pub fn get_field_offset(attr: &syn::Attribute) -> syn::Result<usize> {
    EnsureLayout::get_field_offset(attr)
}

pub fn get_struct_size(attr: &syn::Attribute) -> syn::Result<Option<usize>> {
    if let syn::Meta::List(l) = &attr.meta {
        let params = EnsureLayout::parse.parse2(l.tokens.clone())?;
        Ok(params.get_size())
    } else {
        Err(syn::Error::new(attr.span(), "ensure_layout format is invalid"))
    }
}
