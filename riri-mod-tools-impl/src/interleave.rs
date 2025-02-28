#![allow(unused_imports)]

use proc_macro2::{ Span, TokenStream };
use syn::{
    parse::{ Parse, Parser },
    spanned::Spanned,
    Token
};
use std::{
    error::Error,
    fmt::Display
};
use quote::{quote, ToTokens,format_ident};

pub fn interleave_impl(item: TokenStream) -> TokenStream {
    let target: syn::ItemStruct = match syn::parse2(item) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.to_compile_error())
    };
    match &target.fields {
        syn::Fields::Named(_) => (),
        syn::Fields::Unnamed(_) => (),
        _ => return syn::Error::new(target.span(), "Named structs and tuple structs are supported").to_compile_error()
    };
    match implement_interleave_trait_for_struct(target) {
        Ok(t) => t,
        Err(e) => e.to_compile_error()
    }
}

#[derive(Debug)]
pub struct InterleavedBannedFieldTypeError;
impl Display for InterleavedBannedFieldTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InterleavedBannedFieldTypeError")
    }
}
impl Error for InterleavedBannedFieldTypeError {}

enum InterleavedStructFieldType<'a> {
    Array(&'a syn::TypeArray),
    Path(&'a syn::TypePath)
}

enum InterleavedFieldNameType {
    Named(String),
    Unnamed(usize)
}

struct InterleavedStructField<'a> {
    // ty: &'a syn::Type,
    ty: InterleavedStructFieldType<'a>,
    // field_name: String,
    field_name: InterleavedFieldNameType,
    ident_name: syn::Ident
}
impl<'a> InterleavedStructField<'a> {
    fn from_field(f: &'a syn::Field, i: usize) -> Result<Self, InterleavedBannedFieldTypeError> {
        let ty = match Self::get_legal_field_type(&f.ty) {
            Ok(v) => v,
            Err(e) => return Err(e)
        };
        let (field_name, ident_name) = match f.ident.as_ref() {
            Some(n) => (InterleavedFieldNameType::Named(n.to_string()), n.clone()),
            None => (InterleavedFieldNameType::Unnamed(i),
                syn::Ident::new(&format!("field_{}", i), Span::call_site()),
            )
        };
        Ok(Self { ty, field_name, ident_name })
    }
    fn get_legal_field_type(ty: &'a syn::Type) 
        -> Result<InterleavedStructFieldType<'a>, InterleavedBannedFieldTypeError> {
        match ty {
            syn::Type::Array(t) => Ok(InterleavedStructFieldType::Array(t)),
            syn::Type::Path(t) => Ok(InterleavedStructFieldType::Path(t)),
            _ => Err(InterleavedBannedFieldTypeError)
        }
    }
    fn get_method_call_fmt(&self) -> TokenStream {
        match self.ty {
            InterleavedStructFieldType::Path(t) => {
                t.to_token_stream()
            },
            InterleavedStructFieldType::Array(t) => {
                let ty_in = t.to_token_stream();
                quote! {
                    <#ty_in as riri_mod_tools_rt::interleave::Interleave>
                }
            }
        }
    }
    fn get_type_fmt(&self) -> TokenStream {
        match self.ty {
            InterleavedStructFieldType::Array(t) => t.to_token_stream(),
            InterleavedStructFieldType::Path(t) => t.to_token_stream()
        }
    }
}

pub fn implement_interleave_trait_for_struct(item: syn::ItemStruct) -> syn::Result<TokenStream> {
    let struct_name = item.ident;
    let mut interleave_array_inner = vec![];
    interleave_array_inner.push(quote! {
        let mut cursor = 0;
    });
    let mut deinterleave_array_inner = vec![];
    deinterleave_array_inner.push(quote! {
        let mut cursor = 0;
    });
    let mut interleave_slice_inner = vec![];
    let mut deinterleave_slice_inner = vec![];
    deinterleave_slice_inner.push(quote! {
        let entries = data.len() / ::std::mem::size_of::<Self>();
        let mut alloc: ::std::vec::Vec<u8> = ::std::vec::Vec::with_capacity(entries * ::std::mem::size_of::<Self>());
        let mut cursor = 0;
    });
    for (i, f) in item.fields.iter().enumerate() {
        let curr_field = match InterleavedStructField::from_field(f, i) {
            Ok(v) => v,
            Err(e) => return Err(syn::Error::new(Span::call_site(), e.to_string()))
        };
        // interleave_array_inner
        let curr_ident_name = &curr_field.ident_name;
        let field_name_arr = match &curr_field.field_name {
            InterleavedFieldNameType::Named(n) => {
                let n_ident = syn::Ident::new(n.as_ref(), Span::call_site());
                quote! { data[i].#n_ident }
            },
            
            InterleavedFieldNameType::Unnamed(u) => {
                let field = syn::parse_str::<syn::ExprField>(&format!("data[i].{}", u))?;
                quote! { #field }
            }
        };
        let curr_field_ty = curr_field.get_type_fmt();
        let curr_field_call = curr_field.get_method_call_fmt();
        let cursor_inc = if i < item.fields.len() - 1 {
            quote! { cursor += ::std::mem::size_of::<#curr_field_ty>() * N; }
        } else { TokenStream::new() };
        interleave_array_inner.push(quote! {
            let #curr_ident_name = ::std::array::from_fn::<_, N, _>(|i| #field_name_arr);
            #curr_field_call::interleave_array_inner(&mut outer[cursor..cursor + ::std::mem::size_of::<#curr_field_ty>() * N], &#curr_ident_name);
            #cursor_inc
        });
        // deinterlave_array_inner
        let cursor_inc_de = if i < item.fields.len() - 1 {
            quote! { cursor += ::std::mem::size_of::<#curr_field_ty>(); }
        } else { TokenStream::new() };
        let curr_field_name_slice = syn::Ident::new(&format!("{}_slice", curr_ident_name.to_string()), Span::call_site());
        deinterleave_array_inner.push(quote! {
            let mut #curr_ident_name: ::std::mem::MaybeUninit<[#curr_field_ty; N]> = ::std::mem::MaybeUninit::uninit();
            let #curr_field_name_slice = ::std::slice::from_raw_parts_mut(#curr_ident_name.as_mut_ptr() as *mut u8, N * ::std::mem::size_of::<#curr_field_ty>());
            let call = &data[cursor * N..(cursor * N) + ::std::mem::size_of::<#curr_field_ty>() * N];
            #curr_field_call::deinterleave_array_inner::<N>(#curr_field_name_slice, &data[cursor * N..(cursor * N) + ::std::mem::size_of::<#curr_field_ty>() * N])?;
            for i in 0..N {
                ::std::ptr::copy_nonoverlapping(
                    #curr_field_name_slice.as_ptr().add(i * ::std::mem::size_of::<#curr_field_ty>()),
                    outer.as_mut_ptr().add(i * ::std::mem::size_of::<Self>() + cursor),
                    ::std::mem::size_of::<#curr_field_ty>());
            }
            #cursor_inc_de
        });
        // interleave_slice_inner
        let field_name_slice = match &curr_field.field_name {
            InterleavedFieldNameType::Named(n) => {
                let n_ident = syn::Ident::new(n.as_ref(), Span::call_site());
                quote! { v.#n_ident }
            },
            InterleavedFieldNameType::Unnamed(u) => {
                let field = syn::parse_str::<syn::ExprField>(&format!("v.{}", u))?;
                quote! { #field }
            } 
        };
        interleave_slice_inner.push(quote! {
            let #curr_ident_name: Vec<_> = data.iter().map(|v| #field_name_slice).collect();
            #curr_field_call::interleave_slice_inner(outer, #curr_ident_name.as_slice());
        });
        // deinterleave_slice_inner
        let cursor_inc_slice = if i < item.fields.len() - 1 {
            quote! { cursor += ::std::mem::size_of::<#curr_field_ty>(); }
        } else { TokenStream::new() };
        deinterleave_slice_inner.push(quote! {
            let #curr_field_name_slice = ::std::slice::from_raw_parts_mut(alloc.as_mut_ptr() as *mut u8, entries * ::std::mem::size_of::<#curr_field_ty>());
            #curr_field_call::deinterleave_slice_inner(#curr_field_name_slice, &data[cursor * entries..(cursor * entries) + ::std::mem::size_of::<#curr_field_ty>() * entries])?;
            for i in 0..entries {
                ::std::ptr::copy_nonoverlapping(
                    #curr_field_name_slice.as_ptr().add(i * ::std::mem::size_of::<#curr_field_ty>()), 
                    outer.as_mut_ptr().add(i * ::std::mem::size_of::<Self>() + cursor),
                    ::std::mem::size_of::<#curr_field_ty>());
            }
            #cursor_inc_slice
        });
    }
    let out = quote! {
        impl riri_mod_tools_rt::interleave::Interleave for #struct_name {
            unsafe fn interleave_array_inner<const N: usize>(outer: &mut [u8], data: &[Self; N]) {
                #(#interleave_array_inner)*
            }
            unsafe fn deinterleave_array_inner<const N: usize>(outer: &mut [u8], data: &[u8]) 
                -> Result<(), riri_mod_tools_rt::interleave::InterleaveError> {
                #(#deinterleave_array_inner)*
                Ok(())
            }
            unsafe fn interleave_slice_inner(outer: &mut Vec<u8>, data: &[Self]) {
                #(#interleave_slice_inner)*
            }
            unsafe fn deinterleave_slice_inner(outer: &mut [u8], data: &[u8])
                -> Result<(), riri_mod_tools_rt::interleave::InterleaveError> {
                #(#deinterleave_slice_inner)*
                Ok(())
            }
        }
    };
    // println!("{}", out.to_string());
    Ok(out)
}

pub fn interleave_auto_impl(item: TokenStream) -> TokenStream {
    let to_impl = match syn::punctuated::Punctuated
        ::<syn::Ident, Token![,]>::parse_terminated.parse2(item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error()
    };
    let mut impls = Vec::new();
    for im in to_impl {
        impls.push(quote! {
            impl crate::interleave::Interleave for #im {
                unsafe fn interleave_array_inner<const N: usize>(outer: &mut [u8], data: &[Self; N]) {
                    for i in 0..size_of::<Self>() {
                        for j in 0..N {
                            let bytes = Self::to_le_bytes(data[j]);
                            *outer.as_mut_ptr().add(i * N + j) = bytes[i];
                        }
                    }
                }
                unsafe fn deinterleave_array_inner<const N: usize>(outer: &mut [u8], data: &[u8]) -> Result<(), crate::interleave::InterleaveError> {
                    for i in 0..N {
                        for j in 0..size_of::<Self>() {
                            *outer.as_mut_ptr().add(i * size_of::<Self>() + j) = data[j * N + i];
                        }
                    }
                    Ok(())
                }
                unsafe fn interleave_slice_inner(outer: &mut Vec<u8>, data: &[Self]) {
                    for i in 0..size_of::<Self>() {
                        for j in 0..data.len() {
                            let bytes = Self::to_le_bytes(data[j]);
                            *outer.as_mut_ptr().add(outer.len() + i * data.len() + j) = bytes[i];
                        }
                    }
                    let len = data.len() * size_of::<Self>();
                    outer.set_len(outer.len() + len);
                }
                unsafe fn deinterleave_slice_inner(outer: &mut [u8], data: &[u8]) -> Result<(), crate::interleave::InterleaveError> {
                    let len = data.len() / size_of::<Self>();
                    for i in 0..len {
                        for j in 0..size_of::<Self>() {
                            *outer.as_mut_ptr().add(i * size_of::<Self>() + j) = data[j * len + i];
                        }
                    }
                    Ok(())
                }
            }

        });
    }
    quote! { #(#impls)* }
}
