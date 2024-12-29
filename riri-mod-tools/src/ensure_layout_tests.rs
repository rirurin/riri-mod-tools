#![cfg(test)]
#[test]
fn correct_primitives() -> Result<(), Box<dyn std::error::Error>> {
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
    let attributes = quote! {
        size = 0x10
    };

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
#[ignore]
fn correct_pointers() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[test]
#[ignore]
fn correct_arrays() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[test]
#[ignore]
fn correct_start_padding() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[test]
#[ignore]
fn correct_end_padding() -> Result<(), Box<dyn std::error::Error>> {
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
