#![cfg(test)]
use std::{
    error::Error,
    fmt::Display
};
use riri_mod_tools_impl::hook_codegen::Reloaded2CSharpHook;

type ReturnType = Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct WrongItemType(&'static str);
impl Error for WrongItemType {}
impl Display for WrongItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parsed wrong item type, should be {}", self.0)
    }
}

#[derive(Debug)]
struct WrongTypeFormat(&'static str);
impl Error for WrongTypeFormat {}
impl Display for WrongTypeFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parsed wrong type format, should be {}", self.0)
    }
}

fn ensure_functions_are_identical(first: &syn::ItemFn, other: &syn::ItemFn) -> ReturnType {
    assert!(first.block.as_ref().stmts.len() == other.block.as_ref().stmts.len(), 
    "Number of blocks should be equal since the function should be unchanged");
    check_function_signature_types(first, other)?;
    for i in 0..first.block.as_ref().stmts.len() {
        assert!(first.block.as_ref().stmts[i] == other.block.as_ref().stmts[i],
        "Block {} does not match, the function should be unchanged", i);
    }
    Ok(())
}

fn check_function_signature_types(first: &syn::ItemFn, other: &syn::ItemFn) -> ReturnType {
    assert!(first.sig.inputs.len() == other.sig.inputs.len(), "Input parameter count should match");
    for i in 0..first.sig.inputs.len() {
        let (first_curr, other_curr) = (&first.sig.inputs[i], &other.sig.inputs[i]);
        let (first_ty, other_ty) = (
            match first_curr {
                syn::FnArg::Typed(t) => t.ty.as_ref(),
                _ => return Err(Box::new(WrongTypeFormat("Type")))
            },
            match other_curr {
                syn::FnArg::Typed(t) => t.ty.as_ref(),
                _ => return Err(Box::new(WrongTypeFormat("Type")))
            }
        );
        assert!(first_ty == other_ty, "Input parameter {} should match", i);
    }
    assert!(&first.sig.output == &other.sig.output, "Return parameter should match");
    Ok(())
}

fn check_function_signature_types_bare_type(first: &syn::ItemFn, other: &syn::TypeBareFn) -> ReturnType {
    assert!(other.abi != None, "Function pointer should be defined using an ABI");
    assert!(other.abi.as_ref().unwrap().name.as_ref().unwrap().value() == "C", 
    "Function pointer should be defined using C ABI");
    assert!(first.sig.inputs.len() == other.inputs.len(), "Input parameter count should match");
    for i in 0..first.sig.inputs.len() {
        let (first_curr, other_curr) = (&first.sig.inputs[i], &other.inputs[i]);
        let (first_ty, other_ty) = (
            match first_curr {
                syn::FnArg::Typed(t) => t.ty.as_ref(),
                _ => return Err(Box::new(WrongTypeFormat("Type")))
            },
            &other_curr.ty
        );
        assert!(first_ty == other_ty, "Input parameter {} should match", i);
    }
    assert!(&first.sig.output == &other.output, "Return parameter should match");
    Ok(())
}

fn check_original_function_pointer(hook_fn: &syn::ItemFn, gen_ptr: &syn::Item) -> ReturnType {
    let correct_path_segment_oncelock = ["std", "sync", "OnceLock"];
    let original_function_ptr = match &gen_ptr {
        syn::Item::Static(s) => s,
        _ => return Err(Box::new(WrongItemType("static")))
    };
    let pointer_name = Reloaded2CSharpHook::make_hook_og_fn_string(&hook_fn.sig.ident.to_string().to_ascii_uppercase());
    assert!(&original_function_ptr.ident.to_string() == &pointer_name,
    "Wrong name was generated for original function pointer");
    let segments = match original_function_ptr.ty.as_ref() {
        syn::Type::Path(p) => {
            assert!(p.path.leading_colon != None, "std type should have leading colon in macros");
            &p.path.segments  
        },
        _ => return Err(Box::new(WrongTypeFormat("path")))
    };
    for (i, segment) in segments.iter().enumerate() {
        assert!(&segment.ident.to_string() == correct_path_segment_oncelock[i], "Incorrect segment");
    }
    // Now check that the last segment stores the function pointer for the hook function
    let oncelock_ty = segments.last().unwrap();
    let inner_type = match &oncelock_ty.arguments {
        syn::PathArguments::AngleBracketed(a) => {
            assert!(a.args.len() == 1, "Only one generic argument should exist for function pointer");
            match &a.args[0] {
                syn::GenericArgument::Type(t) => t,
                _ => return Err(Box::new(WrongTypeFormat("Type")))
            }
        },
        _ => return Err(Box::new(WrongTypeFormat("AngleBracketed")))
    };
    let inner_type = match inner_type {
        syn::Type::BareFn(f) => f,
        _ => return Err(Box::new(WrongTypeFormat("BareFn")))
    };
    check_function_signature_types_bare_type(hook_fn, &inner_type)?;
    Ok(())
}

fn check_function_pointer_setter(hook_fn: &syn::ItemFn, set_fn: &syn::Item) -> ReturnType {
    let set_fn = match set_fn {
        syn::Item::Fn(f) => f,
        _ => return Err(Box::new(WrongItemType("function")))
    };
    let pointer_name = Reloaded2CSharpHook::make_hook_set_string(&hook_fn.sig.ident.to_string().to_ascii_uppercase());
    assert!(&set_fn.sig.ident.to_string() == &pointer_name,
    "Wrong name was generated for pointer setter");
    // Check input parameter type for cb
    assert!(set_fn.sig.inputs.len() == 1, "Only one type parameter should exist for pointer setter");
    let cb_type = match set_fn.sig.inputs.first().unwrap() {
        syn::FnArg::Typed(t) => t.ty.as_ref(),
        _ => return Err(Box::new(WrongTypeFormat("Type")))
    };
    let cb_type = match cb_type {
        syn::Type::BareFn(f) => f,
        _ => return Err(Box::new(WrongTypeFormat("BareFn")))
    };
    check_function_signature_types_bare_type(hook_fn, cb_type)?;
    Ok(())
}

#[test]
fn correct_function_hook_static_offset() -> ReturnType {
    use riri_mod_tools_impl::riri_hook::riri_hook_fn_impl;
    use quote::quote;
    let input_function = quote! {
        pub unsafe extern "C" fn test_function_static_offset(a1: u32, a2: *mut u8) -> u32 {
            a1 * 2
        }
    };
    let attributes = quote! { static_offset(0x10) };
    let result = riri_hook_fn_impl(attributes, input_function.clone());
    let transformed: syn::File = syn::parse2(result)?; 
    let input_function_ast: syn::ItemFn = syn::parse2(input_function)?;
    assert!(transformed.items.len() == 3, "Incorrect number of items generated");
    // First item to check is the pointer to the original function
    check_original_function_pointer(&input_function_ast, &transformed.items[0])?;
    // Second test is setting the pointer
    check_function_pointer_setter(&input_function_ast, &transformed.items[1])?;
    // Third test is the hooked function we've defined
    let hooked_function = match &transformed.items[2] {
        syn::Item::Fn(f) => f,
        _ => return Err(Box::new(WrongItemType("function")))
    };
    ensure_functions_are_identical(hooked_function, &input_function_ast)?;
    Ok(())
}

// This should generate the same AST as static offset.
#[test]
fn correct_function_hook_dynamic_offset() -> ReturnType {
    use riri_mod_tools_impl::riri_hook::riri_hook_fn_impl;
    use quote::quote;
    let input_function = quote! {
        pub unsafe extern "C" fn test_function_dynamic_offset(a1: u32, a2: *mut u8) -> u32 {
            a1 * 2
        }
    };
    let attributes = quote! { dynamic_offset(
        signature = "F7 05 ?? ?? ?? ?? 00 00 00 02", // Metaphor: Refantazio gfdGlobal
        resolve_type = set_gfd_global,
        calling_convention = "microsoft"
    ) };
    let result = riri_hook_fn_impl(attributes, input_function.clone());
    let transformed: syn::File = syn::parse2(result)?; 
    let input_function_ast: syn::ItemFn = syn::parse2(input_function)?;
    assert!(transformed.items.len() == 3, "Incorrect number of items generated");
    // First item to check is the pointer to the original function
    check_original_function_pointer(&input_function_ast, &transformed.items[0])?;
    // Second test is setting the pointer
    check_function_pointer_setter(&input_function_ast, &transformed.items[1])?;
    // Third test is the hooked function we've defined
    let hooked_function = match &transformed.items[2] {
        syn::Item::Fn(f) => f,
        _ => return Err(Box::new(WrongItemType("function")))
    };
    ensure_functions_are_identical(hooked_function, &input_function_ast)?;
    Ok(())
}

#[test]
fn hook_original_function_in_main_block() -> ReturnType {
    use riri_mod_tools_impl::riri_hook::riri_hook_fn_impl;
    use quote::quote;
    let input_function = quote! {
        pub unsafe extern "C" fn test_function_main (a1: u32) -> u32 {
            let local_0 = original_function!(a1);
            let local_1 = original_function!(a1) * 2;
            let local_2 = { original_function!(a1) };
            let local_3 = unsafe { original_function!(a1) };
            original_function!(a1)
        }
    };
    let attributes = quote! { static_offset(0) };
    let result = riri_hook_fn_impl(attributes, input_function.clone());
    // let transformed: syn::File = syn::parse2(result)?;
    println!("{}", result.to_string());
    Ok(())
}
/*
// TODO
// Add original_function! into condition, then branch and else branch
#[test]
fn hook_original_function_in_if_statement() -> ReturnType {
    Ok(())
}

#[test]
fn hook_original_function_in_match() -> ReturnType {
    Ok(())
}

#[test]
fn hook_original_function_in_for_loop() -> ReturnType {
    Ok(())
}

#[test]
fn hook_original_function_in_method_call() -> ReturnType {
    Ok(())
}

#[test]
fn hook_original_function_in_while_loop() -> ReturnType {
    Ok(())
}

#[test]
fn hook_function_in_loop() -> ReturnType {
    Ok(())
}
*/
