#![cfg(test)]
type ReturnType = Result<(), Box<dyn std::error::Error>>;
#[test]
fn correct_primitives() -> ReturnType {
    use riri_mod_tools_impl::ensure_layout::ensure_layout_impl;
    //use syn::parse2;
    use quote::quote;
    let input_struct = quote! {
        pub struct DatWeaponEntry {
            #[field_offset(0x0)] flags: u16,
            #[field_offset(0x4)] power: u32,
            #[field_offset(0xc)] equip_users: u32
        }
    };
    let attributes = quote! { size = 0x10 };

    let result = ensure_layout_impl(attributes, input_struct);
    // validate the layout
    let transformed: syn::ItemStruct = syn::parse2(result)?;
    assert!(transformed.attrs.len() == 1, "Incorrect number of attributes were generated");
    // check #[repr(C)] (TODO: Add align(N) support)
    let correct_repr = {
        let is_repr = transformed.attrs[0].path().is_ident("repr");
        is_repr
    };
    assert!(correct_repr, "Didn't generate the correct repr attribute");
    // check offsets + padding for each field

    Ok(())
}

#[test]
fn correct_pointers() -> ReturnType {
    use riri_mod_tools_impl::ensure_layout::ensure_layout_impl;
    //use syn::parse2;
    use quote::quote;
    let input_struct = quote! {
        pub struct LinkedList {
            #[field_offset(0x0)] prev: *mut LinkedList,
            #[field_offset(0x8)] next: *mut LinkedList,
            #[field_offset(0x10)] data: u32
        }
    };
    let attributes = quote! { size = 0x18 };

    let result = ensure_layout_impl(attributes, input_struct);
    // validate the layout
    let transformed: syn::ItemStruct = syn::parse2(result)?;
    assert!(transformed.attrs.len() == 1, "Incorrect number of attributes were generated");
    // check #[repr(C)] (TODO: Add align(N) support)
    let correct_repr = {
        let is_repr = transformed.attrs[0].path().is_ident("repr");
        is_repr
    };
    assert!(correct_repr, "Didn't generate the correct repr attribute");
    // check offsets + padding for each field
    Ok(())
}

#[test]
fn zero_field_type() -> ReturnType {
    use riri_mod_tools_impl::ensure_layout::ensure_layout_impl;
    //use syn::parse2;
    use quote::quote;
    let input_struct = quote! {
        pub struct Blank {}
    };
    let attributes = quote! { size = 0x20 };

    let result = ensure_layout_impl(attributes, input_struct);
    // validate the layout
    let _transformed: syn::ItemStruct = syn::parse2(result)?;
    Ok(())
}

#[test]
fn correct_generic_types() -> ReturnType {
    use riri_mod_tools_impl::ensure_layout::ensure_layout_impl;
    //use syn::parse2;
    use quote::quote;
    let input_struct = quote! {
        pub struct IntegerMaybe {
            #[field_offset(0x0)] int: Option<u32>,
        }
    };
    let attributes = quote! { size = 0x10 };

    let result = ensure_layout_impl(attributes, input_struct);
    // validate the layout
    let transformed: syn::ItemStruct = syn::parse2(result)?;
    Ok(())
}

#[test]
#[ignore]
fn correct_arrays() -> ReturnType {
    Ok(())
}

#[test]
#[ignore]
fn correct_start_padding() -> ReturnType {
    Ok(())
}

#[test]
#[ignore]
fn correct_end_padding() -> ReturnType {
    Ok(())
}

#[test]
#[should_panic]
#[ignore]
fn incorrect_item_type() {
}

#[test]
#[should_panic]
#[ignore]
fn incorrect_missing_field_label() {
}

#[test]
#[should_panic]
#[ignore]
fn incorrect_duplicate_field_label() {
}

#[test]
#[should_panic]
#[ignore]
fn incorrect_overlapping_fields() {
}

#[test]
#[should_panic]
#[ignore]
fn incorrect_dst_field() {
}
