// Admittedly, this is very similar to the memory-layout crate:
// https://crates.io/crates/memory-layout
// Though I mainly took this as an oppotunity to learn how to use procedural macros (I was
// certainly going to need it to make riri_hook :adachi:)

#![allow(dead_code)]

// use const_hex;
use proc_macro2::TokenStream as TokenStream2;
use std::borrow::Borrow;
use syn::{ 
    self,
    parse::{ Parse, Parser },
    punctuated::Punctuated,
    spanned::Spanned,
    Token,
};
use quote::{
    format_ident,
    ToTokens,
    quote
};

#[derive(Clone)]
pub struct EnsureLayout {
    size: Option<usize>,
    align: Option<usize>
}

impl EnsureLayout {
    pub fn get_size(&self) -> Option<usize> { self.size }
    pub fn get_align(&self) -> Option<usize> { self.align }

    #[inline(always)]
    fn fmt_invalid_identifer(count: &str, ident_name: &str) -> String {
        format!("Name of {} argument should be the identifier {}", count, ident_name)
    }
    #[inline(always)]
    fn fmt_compile_constant(ident_name: &str) -> String {
        format!("Identifier {} must be a value known at compile time", ident_name)
    }
    fn parse_parameter(arg: &syn::ExprAssign, label: &str, count: &str) -> syn::Result<usize> {
        if let syn::Expr::Path(i) = arg.left.borrow() {
            let ident_label = i.path.require_ident()?;
            /*let ident_label = match i.path.get_ident() {
                Some(n) => n,
                None => return Err(syn::Error::new(arg.span(), EnsureLayout::fmt_invalid_identifer(count, label)))
            };
            */
            if ident_label.to_string().as_str() != label {
                return Err(syn::Error::new(i.span(), EnsureLayout::fmt_invalid_identifer(count, label)));
            }
            if let syn::Expr::Lit(n) = arg.right.borrow() {
                if let syn::Lit::Int(v) = &n.lit {
                    let ret = v.base10_parse::<usize>()?;
                    return Ok(ret);
                }
            }
        }
        Err(syn::Error::new(arg.span(), EnsureLayout::fmt_compile_constant(label)))
    }
    fn build_type_fragment(ty: &syn::Type) -> TokenStream2 {
        match ty {
            syn::Type::Path(p) => {
                let p_i = p.path.to_token_stream();
                quote! { ::core::mem::size_of::<#p_i>() }
            },
            syn::Type::Ptr(_p) => {
                // this will always be usize
                quote! { ::core::mem::size_of::<usize>() }
                /*
                let p_p = if let Some(v) = &p.const_token {
                    quote! { *#v }
                } else if let Some(v) = &p.mutability {
                    quote! { *#v }
                } else {
                    syn::Error::new(p.span(), "Pointer type isn't const or mut (what?)").to_compile_error()
                };
                let p_ty = EnsureLayout::build_type_fragment(&p.elem);
                quote! { ::core::mem::size_of::<#p_p #p_ty>() }
                */
            },
            syn::Type::Array(p) => {
                let p_l = p.len.to_token_stream();
                let p_ty = EnsureLayout::build_type_fragment(&p.elem);
                quote! { #p_l * (#p_ty) }
            }
            syn::Type::Slice(_) | 
            syn::Type::TraitObject(_) => syn::Error::new(ty.span(), "Dynamically sized types are not supported").to_compile_error(),
            _ => syn::Error::new(ty.span(), "Field type is not supported").to_compile_error()
        }
    }
    pub fn get_field_offset(attr: &syn::Attribute) -> syn::Result<usize> {
        if let syn::Meta::List(l_ofs) = &attr.meta {
            let t_ofs = syn::parse2::<syn::LitInt>(l_ofs.tokens.clone())?;
            t_ofs.base10_parse::<usize>()
        } else {
            return Err(syn::Error::new(attr.span(), "field_offset is formatted incorrectly"))
        }
    }

    fn get_pad_tokens(tk: &Option<TokenStream2>, id: usize, d_ofs: usize, ofs: usize) -> Option<TokenStream2> {
        let field_name = format_ident!("__riri_ensure_layout_pad{}", id);
        if let Some(t_ty) = tk {
            Some(quote! { #[doc(hidden)] #field_name: [u8; #d_ofs - #t_ty] })
        } else if ofs > 0 {
            Some(quote! { #[doc(hidden)] #field_name: [u8; #d_ofs] })
        } else {
            None
        }
    }

    fn create_padding_fields(&self, fields_named: &mut syn::FieldsNamed) -> syn::Result<()> {
        let mut ofs: usize = 0;
        let mut pads: Vec<TokenStream2> = vec![];
        let mut type_tokens: Option<TokenStream2> = None;
        for (i, field) in (&mut fields_named.named).into_iter().enumerate() {
            // Check that field_offset for the target field is formatted properly
            let attrs: Vec<&syn::Attribute> = field.attrs.iter()
                .filter(|attr| attr.path().is_ident("field_offset"))
                .collect(); 
            if attrs.len() == 0 && i > 0 {
                return Err(syn::Error::new(field.span(), "field_offset must be defined for each field"));
            } else if attrs.len() > 1 {
                return Err(syn::Error::new(field.span(), "Only one field_offset should be added for each field"));
            }
            // Only the first field can omit the field_offset (this is used in the #[cpp_class]
            // macro to implicitly add a vtable field)
            // There's no mechanism to be able to switch between explicit and implicit offsets
            // after this since we don't know the size of any types (there is no type reflection)
            if attrs.len() == 1 {
                let f_ofs = Self::get_field_offset(&attrs[0])?;
                // println!("{}: 0x{:x}", field.ident.as_ref().unwrap().to_string(), f_ofs);
                /*
                let f_ofs = if let syn::Meta::List(l_ofs) = &attrs[0].meta {
                    let t_ofs = syn::parse2::<syn::LitInt>(l_ofs.tokens.clone())?;
                    t_ofs.base10_parse::<usize>()?
                } else { return Err(syn::Error::new(field.span(), "field_offset is formatted incorrectly"))};
                */
                // check that this field_offset makes sense
                if ofs > f_ofs {
                    return Err(syn::Error::new(field.span(), "field_offset should be larger than the field above"));
                }
                // add padding
                if let Some(p) = Self::get_pad_tokens(&type_tokens, pads.len(), f_ofs - ofs, f_ofs) {
                    pads.push(p);
                }
                // set prev field state
                ofs = f_ofs; 
                // TODO: enumerate() first loop so we don't iterate twice.
                field.attrs.retain(|attr| !attr.path().is_ident("field_offset"));
            }
            type_tokens = Some(EnsureLayout::build_type_fragment(&field.ty));
        }
        // insert padding between fields
        let start_len = fields_named.named.len();
        let end_pad_id = pads.len();
        if pads.len() > 0 {
            let mut pad = pads.pop();
            let mut p_i = 0;
            while let Some(pt) = pad {
                fields_named.named.insert(start_len - (p_i + 1), syn::Field::parse_named.parse2(pt)?);
                pad = pads.pop();
                p_i += 1;
            }
        } 
        // add padding at the end if we aren't equal to size
        if ofs < self.size.unwrap() {
            fields_named.named.push(
                syn::Field::parse_named.parse2(
                    Self::get_pad_tokens(&type_tokens, end_pad_id, self.size.unwrap() - ofs, ofs).unwrap()
                )?
            );
        }
        Ok(())
    }

    fn create_zero_field_struct(&self, i: &mut syn::FieldsNamed) -> syn::Result<()> {
        let pad_token = Self::get_pad_tokens(&None, 0, self.size.unwrap(), self.size.unwrap()).unwrap();
        i.named.push(syn::Field::parse_named.parse2(pad_token)?);
        Ok(())
    }

    // We are somewhat limited in how flexible this attribute can be since Rust doesn't have type
    // reflection. The first field is allowed to not include a field_offset defintion since we know
    // we're starting at 0, but after that, there's no mechanism to get type information such as
    // size.
    //
    fn visit_annotated_struct(&self, i: &mut syn::ItemStruct) -> syn::Result<TokenStream2> {
        let fields_named = match &mut i.fields {
            syn::Fields::Named(n) => n,
            syn::Fields::Unnamed(_) => return Err(syn::Error::new(i.span(), "Structs with unnamed fields are not currently supported")),
            syn::Fields::Unit => return Err(syn::Error::new(i.span(), "Unit structs are not supported"))
        };
        
        if fields_named.named.len() > 0 {
            self.create_padding_fields(fields_named)?;
        } else {
            self.create_zero_field_struct(fields_named)?;
        }

        // TODO: Add explicit align into repr
        let c_rep = syn::Attribute::parse_outer.parse2(quote!{ #[repr(C)] })?;
        i.attrs.push(c_rep[0].clone());
        Ok(i.to_token_stream())
    }
}

impl Parse for EnsureLayout {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<syn::ExprAssign, Token![,]>::parse_terminated(input)?;
        if args.len() == 0 {
            return Ok( EnsureLayout {
                size: None,
                align: None
            });
        } else if args.len() > 2 {
            return Err(input.error("Too many arguments were specified. Only arguments for size and alignment can be specified."));
        }
        let vargs: Vec<syn::ExprAssign> = args.into_iter().collect();
        let v_size = EnsureLayout::parse_parameter(&vargs[0], "size", "first")?;
        let v_align = match vargs.len() {
            2 => Some(EnsureLayout::parse_parameter(&vargs[1], "align", "second")?),
            _ => None
        };
        // Size has to be a multiple of the defined alignment
        if v_align != None && (v_size % v_align.unwrap() != 0) { 
            return Err(input.error("Defined struct size is not a multiple of the alignment"));
        }
        // First argument should be the size
        Ok( EnsureLayout {
            size: Some(v_size),
            align: v_align
        })
    }
}

#[derive(Clone)]
struct FieldOffset(usize);

pub fn ensure_layout_impl(input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    let mut target_struct: syn::ItemStruct = match syn::parse2(annotated_item) {
        Ok(n) => n,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    let args: EnsureLayout = match syn::parse2(input) {
        Ok(n) => n, 
        Err(e) => return TokenStream2::from(e.to_compile_error())
    };
    match args.visit_annotated_struct(&mut target_struct) {
        Ok(trans) => trans,
        Err(e) => return TokenStream2::from(e.to_compile_error())
    }
}

pub fn field_offset_impl(_input: TokenStream2, annotated_item: TokenStream2) -> TokenStream2 {
    TokenStream2::from(syn::Error::new(annotated_item.span(), "field_offset attribute can only be used in structs marked ensure_layout").to_compile_error())
}
