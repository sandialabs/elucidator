use super::utils::{get_last_type_path_segment, syn_err};
use std::fmt;

/// Intermediate representation of an a type in Rust for the Interpret
/// procedural macro. The internal enum consists of easy to use variants
///  for validation and processing purposes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrimitiveType {
    /// A marker indicating an unsigned 8-bit integer.
    Byte,
    /// A marker indicating an unsigned 16-bit integer.
    UnsignedInteger16,
    /// A marker indicating an unsigned 32-bit integer.
    UnsignedInteger32,
    /// A marker indicating an unsigned 64-bit integer.
    UnsignedInteger64,
    /// A marking indicating a signed 8-bit integer.
    SignedInteger8,
    /// A marking indicating a signed 16-bit integer.
    SignedInteger16,
    /// A marking indicating a signed 32-bit integer.
    SignedInteger32,
    /// A marking indicating a signed 64-bit integer.
    SignedInteger64,
    /// A marking indicating a 32-bit floating point value (specifically, the
    /// “binary32” type defined in IEEE 754-2008).
    Float32,
    /// A marking indicating a 64-bit floating-point value (specifically, the
    /// “binary64” type defined in IEEE 754-2008).
    Float64,
    /// A marking indicating a String value.
    Str,
}

impl TryFrom<&syn::Type> for PrimitiveType {
    type Error = syn::Error;

    fn try_from(value: &syn::Type) -> Result<Self, Self::Error> {
        match value {
            syn::Type::Path(type_path) => { Self::try_from(type_path) }
            _ => todo!("Raise error - unexpected type - type must be expressed as a type path for Interpret"),
        }
    }
}

impl TryFrom<String> for PrimitiveType {
    type Error = syn::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let last_segment = value.split("::").last().unwrap_or(value.as_str());
        match last_segment {
            "u8" => syn::Result::Ok(PrimitiveType::Byte),
            "u16" => syn::Result::Ok(PrimitiveType::UnsignedInteger16),
            "u32" => syn::Result::Ok(PrimitiveType::UnsignedInteger32),
            "u64" => syn::Result::Ok(PrimitiveType::UnsignedInteger64),
            "i8" => syn::Result::Ok(PrimitiveType::SignedInteger8),
            "i16" => syn::Result::Ok(PrimitiveType::SignedInteger16),
            "i32" => syn::Result::Ok(PrimitiveType::SignedInteger32),
            "i64" => syn::Result::Ok(PrimitiveType::SignedInteger64),
            "f32" => syn::Result::Ok(PrimitiveType::Float32),
            "f64" => syn::Result::Ok(PrimitiveType::Float64),
            "String" => syn::Result::Ok(PrimitiveType::Str),
            unexpected => syn_err(
                value.clone(),
                format!("'{}' is not an Elucidator primitive type", unexpected),
            ),
        }
    }
}

impl TryFrom<&syn::TypePath> for PrimitiveType {
    type Error = syn::Error;

    fn try_from(value: &syn::TypePath) -> Result<Self, Self::Error> {
        let segment = get_last_type_path_segment(value)?;
        let segment_type = segment.ident.to_string();

        let syn::PathArguments::None = segment.arguments else {
            syn_err(
                value,
                format!(
                    "'{}' is not an Elucidator primitive type",
                    segment_type.as_str()
                ),
            )?
        };

        PrimitiveType::try_from(segment_type)
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            PrimitiveType::Byte => "u8",
            PrimitiveType::UnsignedInteger16 => "u16",
            PrimitiveType::UnsignedInteger32 => "u32",
            PrimitiveType::UnsignedInteger64 => "u64",
            PrimitiveType::SignedInteger8 => "i8",
            PrimitiveType::SignedInteger16 => "i16",
            PrimitiveType::SignedInteger32 => "i32",
            PrimitiveType::SignedInteger64 => "i64",
            PrimitiveType::Float32 => "f32",
            PrimitiveType::Float64 => "f64",
            PrimitiveType::Str => "String",
        };
        write!(f, "{}", type_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod try_from {
        use super::*;

        mod syn_type {
            use super::*;

            #[test]
            fn u8_ok() {
                let syn_type: syn::Type = syn::parse_quote!(u8);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Byte);
            }

            #[test]
            fn qualified_u8_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::u8);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Byte);
            }

            #[test]
            fn u16_ok() {
                let syn_type: syn::Type = syn::parse_quote!(u16);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger16);
            }

            #[test]
            fn qualified_u16_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::u16);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger16);
            }

            #[test]
            fn u32_ok() {
                let syn_type: syn::Type = syn::parse_quote!(u32);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger32);
            }

            #[test]
            fn qualified_u32_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::u32);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger32);
            }

            #[test]
            fn u64_ok() {
                let syn_type: syn::Type = syn::parse_quote!(u64);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger64);
            }

            #[test]
            fn qualified_u64_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::u64);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger64);
            }

            #[test]
            fn i8_ok() {
                let syn_type: syn::Type = syn::parse_quote!(i8);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger8);
            }

            #[test]
            fn qualified_i8_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::i8);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger8);
            }

            #[test]
            fn i16_ok() {
                let syn_type: syn::Type = syn::parse_quote!(i16);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger16);
            }

            #[test]
            fn qualified_i16_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::i16);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger16);
            }

            #[test]
            fn i32_ok() {
                let syn_type: syn::Type = syn::parse_quote!(i32);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger32);
            }

            #[test]
            fn qualified_i32_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::i32);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger32);
            }

            #[test]
            fn i64_ok() {
                let syn_type: syn::Type = syn::parse_quote!(i64);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger64);
            }

            #[test]
            fn qualified_i64_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::i64);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger64);
            }

            #[test]
            fn f32_ok() {
                let syn_type: syn::Type = syn::parse_quote!(f32);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float32);
            }

            #[test]
            fn qualified_f32_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::f32);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float32);
            }

            #[test]
            fn f64_ok() {
                let syn_type: syn::Type = syn::parse_quote!(f64);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float64);
            }

            #[test]
            fn qualified_f64_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::primitive::f64);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float64);
            }

            #[test]
            fn string_ok() {
                let syn_type: syn::Type = syn::parse_quote!(String);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Str);
            }

            #[test]
            fn qualified_string_ok() {
                let syn_type: syn::Type = syn::parse_quote!(std::string::String);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Str);
            }

            #[test]
            fn unexpected_fails() {
                let syn_type: syn::Type = syn::parse_quote!(bool);
                let result = PrimitiveType::try_from(&syn_type);

                assert!(result.is_err());
            }
        }

        mod syn_type_path {
            use super::*;

            #[test]
            fn u8_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(u8);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Byte);
            }

            #[test]
            fn qualified_u8_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::u8);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Byte);
            }

            #[test]
            fn u16_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(u16);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger16);
            }

            #[test]
            fn qualified_u16_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::u16);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger16);
            }

            #[test]
            fn u32_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(u32);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger32);
            }

            #[test]
            fn qualified_u32_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::u32);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger32);
            }

            #[test]
            fn u64_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(u64);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger64);
            }

            #[test]
            fn qualified_u64_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::u64);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger64);
            }

            #[test]
            fn i8_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(i8);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger8);
            }

            #[test]
            fn qualified_i8_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::i8);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger8);
            }

            #[test]
            fn i16_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(i16);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger16);
            }

            #[test]
            fn qualified_i16_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::i16);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger16);
            }

            #[test]
            fn i32_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(i32);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger32);
            }

            #[test]
            fn qualified_i32_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::i32);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger32);
            }

            #[test]
            fn i64_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(i64);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger64);
            }

            #[test]
            fn qualified_i64_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::i64);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger64);
            }

            #[test]
            fn f32_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(f32);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float32);
            }

            #[test]
            fn qualified_f32_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::f32);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float32);
            }

            #[test]
            fn f64_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(f64);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float64);
            }

            #[test]
            fn qualified_f64_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::primitive::f64);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float64);
            }

            #[test]
            fn string_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(String);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Str);
            }

            #[test]
            fn qualified_string_ok() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(std::string::String);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Str);
            }

            #[test]
            fn unexpected_fails() {
                let syn_type_path: syn::TypePath = syn::parse_quote!(bool);
                let result = PrimitiveType::try_from(&syn_type_path);

                assert!(result.is_err());
            }
        }

        mod string {
            use super::*;

            #[test]
            fn u8_ok() {
                let value = "u8".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Byte);
            }

            #[test]
            fn qualified_u8_ok() {
                let value = "std::primitive::u8".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Byte);
            }

            #[test]
            fn u16_ok() {
                let value = "u16".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger16);
            }

            #[test]
            fn qualified_u16_ok() {
                let value = "std::primitive::u16".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger16);
            }

            #[test]
            fn u32_ok() {
                let value = "u32".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger32);
            }

            #[test]
            fn qualified_u32_ok() {
                let value = "std::primitive::u32".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger32);
            }

            #[test]
            fn u64_ok() {
                let value = "u64".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger64);
            }

            #[test]
            fn qualified_u64_ok() {
                let value = "std::primitive::u64".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::UnsignedInteger64);
            }

            #[test]
            fn i8_ok() {
                let value = "i8".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger8);
            }

            #[test]
            fn qualified_i8_ok() {
                let value = "std::primitive::i8".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger8);
            }

            #[test]
            fn i16_ok() {
                let value = "i16".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger16);
            }

            #[test]
            fn qualified_i16_ok() {
                let value = "std::primitive::i16".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger16);
            }

            #[test]
            fn i32_ok() {
                let value = "i32".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger32);
            }

            #[test]
            fn qualified_i32_ok() {
                let value = "std::primitive::i32".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger32);
            }

            #[test]
            fn i64_ok() {
                let value = "i64".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger64);
            }

            #[test]
            fn qualified_i64_ok() {
                let value = "std::primitive::i64".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::SignedInteger64);
            }

            #[test]
            fn f32_ok() {
                let value = "f32".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float32);
            }

            #[test]
            fn qualified_f32_ok() {
                let value = "std::primitive::f32".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float32);
            }

            #[test]
            fn f64_ok() {
                let value = "f64".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float64);
            }

            #[test]
            fn qualified_f64_ok() {
                let value = "std::primitive::f64".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Float64);
            }

            #[test]
            fn string_ok() {
                let value = "String".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Str);
            }

            #[test]
            fn qualified_string_ok() {
                let value = "std::string::String".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_ok());
                let primitive_type = result.unwrap();
                assert_eq!(primitive_type, PrimitiveType::Str);
            }

            #[test]
            fn unexpected_fails() {
                let value = "bool".to_string();
                let result = PrimitiveType::try_from(value);

                assert!(result.is_err());
            }
        }
    }

    mod display {
        use super::*;

        #[test]
        fn u8_ok() {
            assert_eq!(PrimitiveType::Byte.to_string(), "u8");
        }

        #[test]
        fn u16_ok() {
            assert_eq!(PrimitiveType::UnsignedInteger16.to_string(), "u16");
        }

        #[test]
        fn u32_ok() {
            assert_eq!(PrimitiveType::UnsignedInteger32.to_string(), "u32");
        }

        #[test]
        fn u64_ok() {
            assert_eq!(PrimitiveType::UnsignedInteger64.to_string(), "u64");
        }

        #[test]
        fn i8_ok() {
            assert_eq!(PrimitiveType::SignedInteger8.to_string(), "i8");
        }

        #[test]
        fn i16_ok() {
            assert_eq!(PrimitiveType::SignedInteger16.to_string(), "i16");
        }

        #[test]
        fn i32_ok() {
            assert_eq!(PrimitiveType::SignedInteger32.to_string(), "i32");
        }

        #[test]
        fn i64_ok() {
            assert_eq!(PrimitiveType::SignedInteger64.to_string(), "i64");
        }

        #[test]
        fn f32_ok() {
            assert_eq!(PrimitiveType::Float32.to_string(), "f32");
        }

        #[test]
        fn f64_ok() {
            assert_eq!(PrimitiveType::Float64.to_string(), "f64");
        }

        #[test]
        fn string_ok() {
            assert_eq!(PrimitiveType::Str.to_string(), "String");
        }
    }
}
