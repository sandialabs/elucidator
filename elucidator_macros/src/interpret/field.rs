use super::utils::{syn_err, combine_option_errors, get_last_type_path_segment, get_type_path_from_vec_args};
use super::identifier::Identifier;
use super::primitive_type::PrimitiveType;
use super::sizing::Sizing;

/// Intermediate representation of a field in Rust for the Interpret
/// procedural macro. The internal fields of the struct consists of easy to use
/// data types for validation and processing purposes.
#[derive(Clone)]
pub struct Field {
    /// The identifier associated with a field
    ident: Identifier,
    /// The type type associated with the field
    field_type: PrimitiveType,
    /// The sizing information associated with the field
    field_sizing: Sizing,
}

impl TryFrom<&syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(context: &syn::Field) -> Result<Self, Self::Error> {
        let ident = match &context.ident {
            Some(ident) => Identifier::try_from(ident),
            None => syn_err(
                &context.ident, 
                "identifier required for on fields when deriving `Interpret`"
            )
        };
        // This might seem like it could be replaced with PrimitiveType::TryFrom
        // but it *can not*! This is needs to handle the fact that the primitive
        // type could be a primitive type, nested inside an array, or an arg to
        // a Vec
        let field_type = match &context.ty {
            syn::Type::Array(syn::TypeArray { elem, .. }) => {
                PrimitiveType::try_from(elem.as_ref())
            },
            syn::Type::Path(type_path) => {
                let last_segment = get_last_type_path_segment(type_path)?;
                if last_segment.ident.to_string() == "Vec" {
                    let vec_inner_type = get_type_path_from_vec_args(last_segment)?;
                    PrimitiveType::try_from(vec_inner_type)
                } else {
                    PrimitiveType::try_from(type_path)
                }
            }
            _ => {
                syn_err(
                    &context.ty, 
                    "unsupported field type, expected `Vec`, array, or primitive `elucidator` type",
                )
            },
        };
        let field_sizing = Sizing::try_from(&context.ty);
        match combine_option_errors([ident.clone().err(), field_type.clone().err(), field_sizing.clone().err()]) {
            Some(e) => syn::Result::Err(e),
            None => {
                syn::Result::Ok(Self {
                    ident: ident.unwrap(),
                    field_type: field_type.unwrap(),
                    field_sizing: field_sizing.unwrap(),
                })
            }
        }
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let member_string = format!(
            "{}: {}{}",
            self.ident,
            self.field_type,
            self.field_sizing,
       );
        write!(f, "{}", member_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_named_ident_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: u8);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.ident.to_string(), "foo");
    }

    #[test]
    fn test_primitive_type_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: i16);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.field_type, PrimitiveType::SignedInteger16);
    } 

    #[test]
    fn test_array_type_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: [i16; 8]);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.field_type, PrimitiveType::SignedInteger16);
    } 

    #[test]
    fn test_vec_type_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: Vec<i16>);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.field_type, PrimitiveType::SignedInteger16);
    } 

    #[test]
    fn test_primitive_sizing_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: i16);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.field_sizing, Sizing::Singleton);
    } 

    #[test]
    fn test_array_sizing_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: [i16; 8]);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.field_sizing, Sizing::Fixed(8));
    } 

    #[test]
    fn test_vec_sizing_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: Vec<i16>);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.field_sizing, Sizing::Dynamic);
    } 

    #[test]
    fn test_unnamed_ident_fails() {
        let syn_field: syn::Field = syn::parse_quote!(u8);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), "identifier required for on fields when deriving `Interpret`");
    }

    #[test]
    fn test_non_primitive_singlton_fails() {
        let syn_field: syn::Field = syn::parse_quote!(foo: u128);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), "'u128' is not an Elucidator primitive type");
    }

    #[test]
    fn test_non_primitive_dynamic_fails() {
        let syn_field: syn::Field = syn::parse_quote!(foo: Vec<u128>);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), "'u128' is not an Elucidator primitive type");
    }

    #[test]
    fn test_non_primitive_fixed_fails() {
        let syn_field: syn::Field = syn::parse_quote!(foo: [u128; 8]);
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), "'u128' is not an Elucidator primitive type");
    }

    #[test]
    fn test_unhandled_type_fails() {
        let syn_field: syn::Field = syn::parse_quote!(foo: (u8, u8));
        let result = Field::try_from(&syn_field);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), "unsupported field type, expected `Vec`, array, or primitive `elucidator` type");
    }

    #[test]
    fn test_display_singleton_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: u8);
        let result = Field::try_from(&syn_field).unwrap(); 
        assert_eq!(result.to_string(), "foo: u8");
    }
    
    #[test]
    fn test_display_fixed_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: [u64; 100]);
        let result = Field::try_from(&syn_field).unwrap(); 
        assert_eq!(result.to_string(), "foo: u64[100]");
    }

    #[test]
    fn test_display_dynamic_ok() {
        let syn_field: syn::Field = syn::parse_quote!(foo: Vec<i32>);
        let result = Field::try_from(&syn_field).unwrap(); 
        assert_eq!(result.to_string(), "foo: i32[]");
    }
}