use std::fmt;

use super::utils::get_last_type_path_segment;

#[derive(Clone)]
pub enum Sizing {
    Singleton,
    Fixed(usize),
    Dynamic,
}

impl TryFrom<&syn::Type> for Sizing {
    type Error = syn::Error;

    fn try_from(value: &syn::Type) -> Result<Self, Self::Error> {
        match value {
            syn::Type::Array(syn::TypeArray {
                len: syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(litint), .. }),
                ..
            }) => {
                let n = litint.base10_parse()?;
                Ok(Sizing::Fixed(n))
            },
            syn::Type::Path(type_path) => {
                match get_last_type_path_segment(type_path)?.arguments {
                    syn::PathArguments::None => syn::Result::Ok(Sizing::Singleton),
                    syn::PathArguments::AngleBracketed(_) => syn::Result::Ok(Sizing::Dynamic),
                    _ => todo!("Raise error - Interpret requires primitives, a vector, or an array to determine sizing")
                }
            }
            _ => todo!("Raise error - Interpret requires primitives, a vector, or an array to determine sizing"),
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