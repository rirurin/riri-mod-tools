use std::borrow::Borrow;
use syn::spanned::Spanned;

pub struct Utils;
impl Utils {
    // From csbindgen:
    // https://github.com/Cysharp/csbindgen/blob/main/csbindgen/src/type_meta.rs
    // Follows the Rust/C# type name conversion as shown in csbindgen's type marshalling table
    pub fn convert_type_name(ty: &str) -> &str {
        match ty {
            // rust primitives
            "i8" => "sbyte",
            "i16" => "short",
            "i32" => "int",
            "i64" => "long",
            "i128" => "Int128",                  // .NET 7
            "isize" => "nint", // C# 9.0
            "u8" => "byte",
            "u16" => "ushort",
            "u32" => "uint",
            "u64" => "ulong",
            "u128" => "UInt128", // .NET 7
            "f32" => "float",
            "f64" => "double",
            "bool" => "bool",
            "char" => "uint",
            "usize" => "nuint", // C# 9.0
            "()" => "void",
            "c_char" => "byte",
            "c_schar" => "sbyte",
            "c_uchar" => "byte",
            "c_short" => "short",
            "c_ushort" => "ushort",
            "c_int" => "int",
            "c_uint" => "uint",
            "c_long" => "CLong",   // .NET 6
            "c_ulong" => "CULong", // .NET 6
            "c_longlong" => "long",
            "c_ulonglong" => "ulong",
            "c_float" => "float",
            "c_double" => "double",
            "c_void" => "void",
            "CString" => "sbyte",
            "NonZeroI8" => "sbyte",
            "NonZeroI16" => "short",
            "NonZeroI32" => "int",
            "NonZeroI64" => "long",
            "NonZeroI128" => "Int128",
            "NonZeroIsize" => "nint",
            "NonZeroU8" => "byte",
            "NonZeroU16" => "ushort",
            "NonZeroU32" => "uint",
            "NonZeroU64" => "ulong",
            "NonZeroU128" => "UInt128",
            "NonZeroUsize" => "nuint",
            _ => ty
        }
    }
    pub fn is_primitive_type(ty: &str) -> bool {
        ty != Self::convert_type_name(ty)
    }
    // Loosely based on to_csharp_string from csbindgen
    // https://github.com/Cysharp/csbindgen/blob/main/csbindgen/src/type_meta.rs
    pub fn to_csharp_typename(ty: &syn::Type) -> syn::Result<String> {
        match ty {
            syn::Type::Array(a) => {
                let mut ty = Self::to_csharp_typename(a.elem.borrow())?;
                ty.push_str("*");
                Ok(ty)
            },
            syn::Type::Path(p) => {
                let rs_name = p.path
                    .get_ident().unwrap().to_string();
                let cs_name = Self::convert_type_name(&rs_name);
                if cs_name != &rs_name {
                    Ok(String::from(cs_name))
                } else {
                    Ok(rs_name)
                }
            },
            syn::Type::Ptr(p) => {
                let mut ty = Self::to_csharp_typename(p.elem.borrow())?;
                ty.push_str("*");
                Ok(ty)
            },
            _ => Err(syn::Error::new(ty.span(), "Unimplemented CSharp type conversion"))
        }
    }
}
