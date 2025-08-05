use super::interpret::InterpretStruct;

pub fn derive_interpret2(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let context: syn::DeriveInput = match syn::parse2(item) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let interpret = match InterpretStruct::new(context) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };

    quote::quote! { #interpret }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    
    #[test]
    fn test_my_macro_logic_str() {
        let input = quote! {
            struct MyStruct {
                field: i32,
            }
        };
        
        let result = derive_interpret2(input);
        let generated = result.to_string();
        assert!(generated.contains("impl Interpret for MyStruct"));
    }

    // #[test]
    // fn test_valid_identifier() {
    //     let ident: syn::Ident = syn::parse_quote!(valid_name);
    //     let result = Identifier::try_from(&ident);
        
    //     assert!(result.is_ok());
    //     let identifier = result.unwrap();
    //     assert_eq!(identifier.ident, "valid_name");
    // }
}