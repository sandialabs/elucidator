use std::fmt;

use crate::interpret::utils::syn_err;

use super::utils::get_last_type_path_segment;

/// Intermediate representation of how many elements a field in Rust has for
/// the Interpret procedural macro. The internal fields of the struct
/// consists of easy to use enum variants for validation and processing
/// purposes.
#[derive(Clone, Debug, PartialEq)]
pub enum Sizing {
    /// Represents the sizing of a field of data not in a sequence.
    Singleton,
    /// Represents the sizing of a field that holds a sequence of fixed length.
    Fixed(usize),
    /// Represents the sizing of a field that holds a sequence of variable size.
    Dynamic,
}

impl TryFrom<&syn::Type> for Sizing {
    type Error = syn::Error;

    fn try_from(value: &syn::Type) -> Result<Self, Self::Error> {
        match value {
            syn::Type::Array(syn::TypeArray {len, ..}) => {
                match len {
                    syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(litint), .. }) => {
                        let n = litint.base10_parse()?;
                        Ok(Sizing::Fixed(n))
                    },
                    _ => syn_err(len, "array size must be a non-negative integer literal")
                }
            },
            syn::Type::Path(type_path) => {
                let last_segment = get_last_type_path_segment(type_path)?;
                match last_segment.arguments {
                    syn::PathArguments::None => syn::Result::Ok(Sizing::Singleton),
                    syn::PathArguments::AngleBracketed(_) => syn::Result::Ok(Sizing::Dynamic),
                    _ => syn_err(
                        type_path,
                        "Interpret requires primitives, a vector, or an array with a non-negative integer literal to determine sizing"
                    )
                }
            }
            _ => syn_err(
                value,
                "Interpret requires primitives, a vector, or an array with a non-negative integer literal to determine sizing"
            )
        }
    }
}

impl fmt::Display for Sizing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sizing_str = match self {
            Sizing::Singleton => "".to_string(),
            Sizing::Dynamic => "[]".to_string(),
            Sizing::Fixed(n) => format!("[{}]", n),
        };
        write!(f, "{}", sizing_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton_real_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!(u8);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Singleton);
    }

    #[test]
    fn test_singleton_fake_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!(fake_type);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Singleton);
    }

    #[test]
    fn test_singleton_qualified_real_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!(std::primitive::u8);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Singleton);
    }

    #[test]
    fn test_singleton_qualified_fake_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!(std::foo::fake_type);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Singleton);
    }

    #[test]
    fn test_fixed_real_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!([u8; 8]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Fixed(8));
    }

    #[test]
    fn test_fixed_fake_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!([fake_type; 8]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Fixed(8));
    }

    #[test]
    fn test_fixed_qualified_real_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!([std::primitive::u8; 8]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Fixed(8));
    }

    #[test]
    fn test_fixed_qualified_fake_type_ok() {
        let syn_type: syn::Type = syn::parse_quote!([std::foo::fake_type; 8]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Fixed(8));
    }

    #[test]
    fn test_fixed_float_fails() {
        let syn_type: syn::Type = syn::parse_quote!([u8; 8.0]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error.to_string(),
            "array size must be a non-negative integer literal"
        );
    }

    #[test]
    fn test_fixed_negative_fails() {
        let syn_type: syn::Type = syn::parse_quote!([u8; -8]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error.to_string(),
            "array size must be a non-negative integer literal"
        );
    }

    #[test]
    fn test_fixed_expression_fails() {
        let syn_type: syn::Type = syn::parse_quote!([u8; foobar]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error.to_string(),
            "array size must be a non-negative integer literal"
        );
    }

    #[test]
    fn test_fixed_expression_2_fails() {
        let syn_type: syn::Type = syn::parse_quote!([u8; 4 * 4]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error.to_string(),
            "array size must be a non-negative integer literal"
        );
    }

    #[test]
    fn test_fixed_zero_ok() {
        let syn_type: syn::Type = syn::parse_quote!([u8; 0]);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Fixed(0));
    }

    #[test]
    fn test_fixed_dynamic_ok() {
        let syn_type: syn::Type = syn::parse_quote!(Vec<u8>);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Dynamic);
    }

    #[test]
    fn test_dynamic_ok() {
        let syn_type: syn::Type = syn::parse_quote!(Vec<u8>);
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_ok());
        let sizing = result.unwrap();
        assert_eq!(sizing, Sizing::Dynamic);
    }

    #[test]
    fn test_unknown_type_fails() {
        let syn_type: syn::Type = syn::parse_quote!((u8, u8));
        let result = Sizing::try_from(&syn_type);

        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(
            error.to_string(),
            "Interpret requires primitives, a vector, or an array with a non-negative integer literal to determine sizing"
        );
    }
}
