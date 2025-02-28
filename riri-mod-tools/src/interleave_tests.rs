#![cfg(test)]
type ReturnType = Result<(), Box<dyn std::error::Error>>;
#[test]
#[ignore]
fn test_struct_named() -> ReturnType {
    use riri_mod_tools_impl::interleave::interleave_impl;
    use quote::quote;
    let input_struct = quote! {
        #[repr(C)]
        pub struct TestStruct {
            field0: u32,
            field1: f32
        }
    };
    let result = interleave_impl(input_struct);
    println!("{}", result.to_string());
    Ok(())
}

#[test]
fn test_struct_tuple() -> ReturnType {
    use riri_mod_tools_impl::interleave::interleave_impl;
    use quote::quote;
    // for bitflags
    let input_struct = quote! {
        #[repr(C)]
        pub struct TestStruct(u32);
    };
    let result = interleave_impl(input_struct);
    println!("{}", result.to_string());
    Ok(())
}
