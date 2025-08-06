use super::identifier::Identifier;
use super::field::Field;
use super::utils::{syn_err, combine_result_errors, combine_option_errors};

pub struct InterpretStruct {
    ident: Identifier,
    fields: Vec<Field>,
}

impl InterpretStruct {
    pub fn new(context: syn::DeriveInput) -> syn::Result<Self> {
        let ident = Identifier::try_from(&context.ident);
        let fields: syn::Result<Vec<Field>> = match &context.data {
            syn::Data::Struct(syn::DataStruct {fields, ..} ) => {
                let fields: Vec<syn::Result<Field>> = fields
                    .iter()
                    .map(|f| Field::try_from(f))
                    .collect();
                match combine_result_errors(&fields) {
                    Some(e) => syn::Result::Err(e),
                    None => syn::Result::Ok(fields.into_iter().map(syn::Result::unwrap).collect()),
                }
            }
            _ => {
                syn_err(
                    &context,
                    "`Interpret` can only be derived on structs"
                )
            }
        };
        let check_no_generics = if context.generics.params.is_empty() {
            syn::Result::Ok(())
        } else {
            syn_err(
                &context,
                "deriving `Interpret` is not supported with generic parameters"
            )
        };

        match combine_option_errors([
            ident.clone().err(),
            fields.clone().err(),
            check_no_generics.clone().err(),
        ]) {
            Some(e) => syn::Result::Err(e),
            None => {
                syn::Result::Ok(Self {
                    ident: ident.unwrap(),
                    fields: fields.unwrap(),
                })
            }
        }
    }

    fn get_designation_spec(&self) -> String {
        self.fields
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl quote::ToTokens for InterpretStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.ident;
        let designation_spec = self.get_designation_spec();

        let impl_block = quote::quote! {
            impl Interpret for #name {
                fn get_spec() -> DesignationSpecification {
                    DesignationSpecification::from_text(#designation_spec).unwrap()
                }
                fn from_eluciblob() -> Result<Self, ElucidatorError> { todo!("Derived implementation of from_eluciblob not implemented.") }
                fn to_eluciblob() -> Result<Self, ElucidatorError> { todo!("Derived implementation of to_eluciblob not implemented.") }
            }
        };
        impl_block.to_tokens(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::sizing::Sizing;

    #[test]
    fn test_new_ok() {
        let context: syn::DeriveInput = syn::parse_quote!(
            struct MyFoo {
                /// Foo field
                foo: u8,
                /// Bar field
                bar: [u16; 4],
                /// Baz field
                baz: Vec<i32>,
                /// Qualified foo field
                qfoo: std::primitive::u8,
                /// Qualified bar field
                qbar: [std::primitive::u16; 4],
                /// Qualified baz field
                qbaz: std::vec::Vec<std::primitive::i32>,
            }
        );
        let interpret = InterpretStruct::new(context);
        assert!(interpret.is_ok());
    }

    #[test]
    fn test_ident_ok() {
        let context: syn::DeriveInput = syn::parse_quote!(
            struct MyFoo { }
        );
        let result = InterpretStruct::new(context);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.ident.to_string(), "MyFoo");
    }

    #[test]
    fn test_field_names_ok() {
        let context: syn::DeriveInput = syn::parse_quote!(
            struct MyFoo {
                /// Foo field
                foo: u8,
                /// Bar field
                bar: [u16; 4],
                /// Baz field
                baz: Vec<i32>,
                /// Qualified foo field
                qfoo: std::primitive::u8,
                /// Qualified bar field
                qbar: [std::primitive::u16; 4],
                /// Qualified baz field
                qbaz: std::vec::Vec<std::primitive::i32>,
            }
        );
        let interpret = InterpretStruct::new(context).unwrap();
        let field_names: Vec<String> = interpret.fields
            .iter()
            .map(|f| f.get_ident().to_string())
            .collect();
        assert_eq!(
            field_names,
            vec!["foo", "bar", "baz", "qfoo", "qbar", "qbaz"]
        );
    }

    #[test]
    fn test_field_types_ok() {
        let context: syn::DeriveInput = syn::parse_quote!(
            struct MyFoo {
                /// Foo field
                foo: u8,
                /// Bar field
                bar: [u16; 4],
                /// Baz field
                baz: Vec<i32>,
                /// Qualified foo field
                qfoo: std::primitive::u8,
                /// Qualified bar field
                qbar: [std::primitive::u16; 4],
                /// Qualified baz field
                qbaz: std::vec::Vec<std::primitive::i32>,
            }
        );
        let interpret = InterpretStruct::new(context).unwrap();
        let field_types: Vec<String> = interpret.fields
            .iter()
            .map(|f| f.get_field_type().to_string())
            .collect();
        assert_eq!(
            field_types,
            vec!["u8", "u16", "i32", "u8", "u16", "i32"]
        );
    }

    #[test]
    fn test_field_sizes_ok() {
        let context: syn::DeriveInput = syn::parse_quote!(
            struct MyFoo {
                /// Foo field
                foo: u8,
                /// Bar field
                bar: [u16; 4],
                /// Baz field
                baz: Vec<i32>,
                /// Qualified foo field
                qfoo: std::primitive::u8,
                /// Qualified bar field
                qbar: [std::primitive::u16; 4],
                /// Qualified baz field
                qbaz: std::vec::Vec<std::primitive::i32>,
            }
        );
        let interpret = InterpretStruct::new(context).unwrap();
        let field_sizes: Vec<Sizing> = interpret.fields
            .iter()
            .map(|f| f.get_field_sizing().clone())
            .collect();
        assert_eq!(
            field_sizes,
            vec![
                Sizing::Singleton, Sizing::Fixed(4), Sizing::Dynamic, 
                Sizing::Singleton, Sizing::Fixed(4), Sizing::Dynamic, 
            ]
        );
    }

    #[test]
    fn test_field_designation_spec_ok() {
        let context: syn::DeriveInput = syn::parse_quote!(
            struct MyFoo {
                /// Foo field
                foo: u8,
                /// Bar field
                bar: [u16; 4],
                /// Baz field
                baz: Vec<i32>,
                /// Qualified foo field
                qfoo: std::primitive::u8,
                /// Qualified bar field
                qbar: [std::primitive::u16; 4],
                /// Qualified baz field
                qbaz: std::vec::Vec<std::primitive::i32>,
            }
        );
        let interpret = InterpretStruct::new(context).unwrap();
        let designation_spec = interpret.get_designation_spec();
        let gt = "foo: u8, bar: u16[4], baz: i32[], qfoo: u8, qbar: u16[4], qbaz: i32[]";
        assert_eq!(designation_spec, gt);
    }

    #[test]
    fn test_generic_args_fails() {
        let context: syn::DeriveInput = syn::parse_quote!(
            struct MyFoo<'a> {
                /// Foo field
                foo: u8,
            }
        );
        let interpret = InterpretStruct::new(context);
        assert!(interpret.is_err());
        let error = interpret.err().unwrap();
        assert_eq!(
            error.to_string(),
            "deriving `Interpret` is not supported with generic parameters"
        );
    }

    #[test]
    fn test_nonstruct_fails() {
        let context: syn::DeriveInput = syn::parse_quote!(
            enum MyFoo {
                /// Foo field
                FOo
            }
        );
        let interpret = InterpretStruct::new(context);
        assert!(interpret.is_err());
        let error = interpret.err().unwrap();
        assert_eq!(
            error.to_string(),
            "`Interpret` can only be derived on structs"
        );
    }

     
}