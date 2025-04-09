#![allow(unused_variables)]

use proc_macro::TokenStream;
use riri_mod_tools_impl::{ 
    ensure_layout,
    interleave,
    riri_hook,
    riri_init
};

/// Defines a function that can hook into another function and replace or extend it's
/// logic by invoking the original_function! macro. 
#[proc_macro_attribute]
pub fn riri_hook_fn(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    riri_hook::riri_hook_fn_impl(input.into(), annotated_item.into()).into()
}

/// When inside a hooked function, this call is replaced with either a raw function call if they
/// share the same calling convention or to Reloaded's OriginalFunction if they don't
#[proc_macro]
pub fn original_function(input: TokenStream) -> TokenStream {
    riri_hook::original_function_impl(input.into()).into()
}

/// Defines a global variable in the target program.
#[proc_macro_attribute]
pub fn riri_hook_static(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    riri_hook::riri_hook_static_impl(input.into(), annotated_item.into()).into()
}

/// Definition of the macro itself. Requires a name for the generated global and a typename valid
/// inside the given module. This is additionally annotated with riri_hook_static to allow the mod
/// framework to bind an address to it on initialization.
#[proc_macro]
pub fn riri_static(input: TokenStream) -> TokenStream {
    riri_hook::riri_static(input.into()).into()
}

/// Definition of a C++ class, where the first field contains the struct's vtable. Works similarly
/// to riri_static, but provides additional functionality for implementing methods to annotate
/// themselves as overriding a vtable entry, hooking them without having to run a separate sigscan
/// for it.
#[proc_macro_attribute]
pub fn cpp_class(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    riri_hook::cpp_class_impl(input.into(), annotated_item.into()).into()
}

/// Applied to struct implementations that may contain hookable methods. Contains the optional
/// fields "path" to prefix the generated function name to avoid naming conflicts, and "auto_drop"
/// to automatically create a hooked function that calls Drop.
#[proc_macro_attribute]
pub fn cpp_class_methods(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    riri_hook::cpp_class_methods_impl(input.into(), annotated_item.into()).into()
}

/// Virtual method entry for a C++ class. Requires being inside of a cpp_class_methods, but allows
/// for the hook framework to resolve without requiring another sigscan.
#[proc_macro_attribute]
pub fn vtable_method(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    riri_hook::vtable_method_impl(input.into(), annotated_item.into()).into()
}

/// `#[ensure_layout(size = 0x180, align = 0x10)]`
/// Provides a method of enforcing explicit field offsets for a particular struct definition. Every
/// field in the struct it's defined in must be tagged with `#[field_offset]` except for the first
/// item, which is implicitly treated as 0x0 (this allows for `#[cpp_class]` to insert a vtable field
/// without being aware of `ensure_layout`, although this means defining it before `ensure_layout`). A
/// size and alignment value that doesn't match Rust's alignment rules will cause a compiler error.
/// (See [the Rust reference's type layout page](https://doc.rust-lang.org/stable/reference/type-layout.html#size-and-alignment) 
/// for more info)
///
///
/// # Example
/// Given the following struct:
///
/// ``` 
/// # use riri_mod_tools_proc::ensure_layout;
/// pub struct DatUnitId(u32);
///
/// #[ensure_layout(size = 0x5c)]
/// pub struct DatUnitArchetype {
///     #[field_offset(0x0)] flag: u16,
///     #[field_offset(0x4)] id: DatUnitId,
///     #[field_offset(0x8)] level: u32,
///     #[field_offset(0x10)] skills: [u32; 12],
///     #[field_offset(0x48)] extra_skills: [u32; 4]
/// }
/// ```
///
/// The generated struct will be
///
/// ```
/// # pub struct DatUnitId(u32);
/// #[repr(C, align(4))]
/// pub struct DatUnitArchetype {
///     flag: u16,
///     #[doc(hidden)]
///     __riri_ensure_layout_pad0: [u8; 4usize - ::core::mem::size_of::<u16>()],
///     id: DatUnitId,
///     #[doc(hidden)]
///     __riri_ensure_layout_pad1: [u8; 4usize - ::core::mem::size_of::<DatUnitId>()],
///     level: u32,
///     #[doc(hidden)]
///     __riri_ensure_layout_pad2: [u8; 8usize - ::core::mem::size_of::<u32>()],
///     skills: [u32; 12],
///     #[doc(hidden)]
///     __riri_ensure_layout_pad3: [u8; 56usize - 12 * (::core::mem::size_of::<u32>())],
///     extra_skills: [u32; 4],
///     #[doc(hidden)]
///     __riri_ensure_layout_pad4: [u8; 20usize - 4 * (::core::mem::size_of::<u32>())],
/// }
/// ```
///
#[proc_macro_attribute]
pub fn ensure_layout(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    ensure_layout::ensure_layout_impl(input.into(), annotated_item.into()).into()
}

/// An attribute that provides no functionality on it's own, and is instead used to annotate each
/// field of a struct with an appropriate offset so `#[ensure_layout]` can add padding as required.
/// The offset for each entry must be a value equal to or larger than the previous field, not
/// overlap the previous field and meet the alignment requirements for the type. Failing to meet
/// those requirements will cause a compiler error.
#[proc_macro_attribute]
pub fn field_offset(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    ensure_layout::field_offset_impl(input.into(), annotated_item.into()).into()
}

/// Provide a formatted name to print out for an enum when debugging it. This will be embedded
/// into the program on debug builds and be a no-op on release builds.
#[proc_macro_derive(FriendlyName, attributes(friendly_name))]
pub fn friendly_name(input: TokenStream) -> TokenStream {
    input
}

/// Like `riri_hook_fn`, but defines a mid-function hook instead.
/// 
/// `callee_saved_registers`: A list of registers to save information for. By default, Reloaded-II
/// will preserve registers rbx, rdi, rs9, r12, r13, r14 and r15.
#[proc_macro_attribute]
pub fn riri_hook_inline_fn(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    riri_hook::riri_hook_inline_fn_impl(input.into(), annotated_item.into()).into()
}

/// Interleave an array or slice 
#[proc_macro_derive(Interleave, attributes(ignore))]
pub fn interleave(input: TokenStream) -> TokenStream {
    interleave::interleave_impl(input.into()).into()
}

#[proc_macro]
pub fn interleave_auto(input: TokenStream) -> TokenStream {
    interleave::interleave_auto_impl(input.into()).into()
}

/// Defines a function that can hook into another function and replace or extend it's
/// logic by invoking the original_function! macro. 
#[proc_macro_attribute]
pub fn riri_init_fn(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    riri_init::riri_init_fn_impl(input.into(), annotated_item.into()).into()
}

/// When inside a hooked function, this call is replaced with either a raw function call if they
/// share the same calling convention or to Reloaded's OriginalFunction if they don't
#[proc_macro]
pub fn create_hook(input: TokenStream) -> TokenStream {
    riri_hook::create_hook_impl(input.into()).into()
}