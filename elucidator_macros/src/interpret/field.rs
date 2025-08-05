use super::utils::{syn_err, combine_option_errors, get_last_type_path_segment, get_type_path_from_vec_args};
use super::identifier::Identifier;
use super::primitive_type::PrimitiveType;
use super::sizing::Sizing;

#[derive(Clone)]
pub struct Field {
    ident: Identifier,
    field_type: PrimitiveType,
    field_sizing: Sizing,
}

impl TryFrom<&syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(context: &syn::Field) -> Result<Self, Self::Error> {
        let ident = match &context.ident {
            Some(ident) => Identifier::try_from(ident),
            None => syn_err(&context.ident, "The field has no identifier.")
        };
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
                syn_err(&context.ty, "Unsupported field type, expected array or type path.")
            },
            // todo!("Raise error - Can't get field type - Match path arm")
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