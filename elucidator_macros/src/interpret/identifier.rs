use crate::interpret::utils::{syn_err, combine_result_errors};

/// Intermediate representation of an identifier in Rust for the Interpret
/// procedural macro. The internal fields of the struct consists of easy to use
/// data types for validation and processing purposes.
#[derive(Clone)]
pub struct Identifier {
    /// String representation of an identifier
    ident: String
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.ident)
    }
}

impl TryFrom<&syn::Ident> for Identifier {
    type Error = syn::Error;

    fn try_from(value: &syn::Ident) -> Result<Self, Self::Error> {
        let ident = value.to_string();
        let first_alphabetic = if ident.chars().next().unwrap().is_alphabetic() {
            Ok(())
        } else {
            syn_err(value, "Identifier must start with an alphabetic character.")
        };
        let all_alphanumeric = if ident.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Ok(())
        } else {
            syn_err(
                value, 
                "Identifier must only contain alphanumeric or underscore characters."
            )
        };
        match combine_result_errors(&[first_alphabetic, all_alphanumeric]) {
            Some(e) => syn::Result::Err(e)?,
            None => syn::Result::Ok(Self { ident }),
        }
    }
}

impl quote::ToTokens for Identifier {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = proc_macro2::Ident::new(&self.to_string(), proc_macro2::Span::call_site());
        ident.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alphabetic_ok() {
        let ident: syn::Ident = syn::parse_quote!(valid_name);
        let result = Identifier::try_from(&ident);
        
        assert!(result.is_ok());
        let identifier = result.unwrap();
        assert_eq!(identifier.ident, "valid_name");
    }

    #[test]
    fn test_alphanumeric_ok() {
        let ident: syn::Ident = syn::parse_quote!(valid_name_123);
        let result = Identifier::try_from(&ident);
        
        assert!(result.is_ok());
        let identifier = result.unwrap();
        assert_eq!(identifier.ident, "valid_name_123");
    }

   
    #[test]
    fn test_underscore_first_fails() {
        let ident: syn::Ident = syn::parse_quote!(_valid_name_123);
        let result = Identifier::try_from(&ident);
        
        assert!(result.is_err(), "Identifier '{}' should have failed, but did unexpectedly succeeded.", ident.to_string());
        let error = result.err().unwrap();
        assert_eq!("Identifier must start with an alphabetic character.", error.to_string());
    }
  
    // Note:
    //
    // Testing failure behavior for when identifier starts with number is not necessary
    // That naming strucuture is not allowed and is caught by rustc before the proc macro is tested
}