use super::identifier::Identifier;
use super::field::Field;
use super::utils::{syn_err, combine_result_errors, combine_option_errors};

pub struct InterpretStruct {
    ident: Identifier,
    fields: Vec<Field>
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
                    None => syn::Result::Ok(fields.into_iter().map(syn::Result::unwrap).collect::<Vec<Field>>()),
                }
            }
            _ => {
                syn_err(
                    &context,
                    "Interpret can only be derived on structs."
                )
            }
        };
        let check_no_generics = if context.generics.params.is_empty() {
            syn::Result::Ok(())
        } else {
            syn_err(
                &context,
                "Deriving Interpret is not supported with generic parameters."
            )
        };
        // syn_err(&context, "Hello world!")?;
        let check_no_attributes = if context.attrs.is_empty() {
            syn::Result::Ok(())
        } else {
            syn_err(
                &context,
                "Deriving Interpret does not support additional attributes."
            )
        };

        match combine_option_errors([
            ident.clone().err(),
            fields.clone().err(),
            check_no_generics.clone().err(),
            check_no_attributes.clone().err(),
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