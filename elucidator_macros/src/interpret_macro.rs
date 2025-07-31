use std::fmt;

macro_rules! combine_single_errors {
    ( $( $result:expr ),+ $(,)? ) => {{
        vec![$( $result.clone().err() ),+]
            .into_iter()
            .flatten()
            .reduce(|mut acc, err| {
                acc.combine(err);
                acc
            })
    }};
}

macro_rules! combine_iter_errors {
    ( $iter:expr ) => {{
        $iter
            .into_iter()
            .filter_map(|result| result.clone().err())
            .reduce(|mut acc, err| {
                acc.combine(err);
                acc
            })
    }};
}

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
                match combine_iter_errors!(&fields) {
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
        match combine_single_errors!(ident, fields, check_no_generics, check_no_attributes) {
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

#[derive(Clone)]
struct Field {
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
        match combine_single_errors!(ident, field_type, field_sizing) {
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

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let member_string = format!(
            "{}: {}{}",
            self.ident,
            self.field_type,
            self.field_sizing,
       );
        write!(f, "{}", member_string)
    }
}

#[derive(Clone)]
struct Identifier {
    ident: String
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        write!(f, "{}", self.ident)
    }
}

impl TryFrom<&syn::Ident> for Identifier {
    type Error = syn::Error;

    fn try_from(value: &syn::Ident) -> Result<Self, Self::Error> {
        let ident = value.to_string();
        // This error is an instant return
        // If there is no length, we can't check anything else
        if ident.len() == 0 {
            syn_err(value, "Identifier must have a non-zero length.")?;
        }
        // These errors can be combined
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
        match combine_single_errors!(first_alphabetic, all_alphanumeric) {
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

#[derive(Clone)]
enum Sizing {
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

#[derive(Clone)]
enum PrimitiveType {
    Byte,
    UnsignedInteger16,
    UnsignedInteger32,
    UnsignedInteger64,
    SignedInteger8,
    SignedInteger16,
    SignedInteger32,
    SignedInteger64,
    Float32,
    Float64,
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

impl TryFrom<&syn::TypePath> for PrimitiveType {
    type Error = syn::Error;

    fn try_from(value: &syn::TypePath) -> Result<Self, Self::Error> {
        let segment = get_last_type_path_segment(value)?;
        let segment_type = segment.ident.to_string();

        let syn::PathArguments::None = segment.arguments else {
            todo!("Raise error - not primitive")
        };
        
        match segment_type.as_str() {
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
                value, 
                format!("'{}' is not an Elucidator primitive type", unexpected)
            ),
        }
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

// Helper functions

fn syn_err<T, U, V>(
    tokens: T,
    message: U,
) -> syn::Result<V>
where
    T: quote::ToTokens,
    U: std::fmt::Display,
{
    syn::Result::Err(
        syn::Error::new_spanned(tokens, message)
    )
}

fn get_last_type_path_segment(
    type_path: &syn::TypePath,
) -> syn::Result<&syn::PathSegment> {
    match type_path.path.segments.last() {
        Some(segment) => syn::Result::Ok(segment),
        None => todo!("Raise error - no segments?"),
    }
}

fn get_type_path_from_vec_args(
    path_segment: &syn::PathSegment,
) -> syn::Result<&syn::TypePath> {
    if path_segment.ident.to_string() != "Vec" {
        syn_err(
            path_segment,
            format!("Expected a Vec, got '{}'", path_segment.ident),
        )?
    }
    match &path_segment.arguments {
        syn::PathArguments::AngleBracketed(angled_args) => {
            if angled_args.args.first().is_none() {
                syn_err(angled_args, "No arguments supplied to Vec.")?
            }
            match &angled_args.args.first().unwrap() {
                syn::GenericArgument::Type(type_) => {
                    match type_ {
                        syn::Type::Path(type_path) => syn::Result::Ok(type_path),
                        _ => syn_err(
                            type_,
                            "Unexpected type format, expected a type path"
                        )
                    }
                },
                _ => syn_err(
                    angled_args,
                    "Unexpected arguments to Vec"
                )
            }
        },
        _ => syn_err(
            &path_segment.arguments,
            "Path segment must have arguments in angled brackets"
        ),
    }
}

pub fn derive_interpret(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use proc_macro2::TokenStream;
//     use quote::quote;

//     #[test]
//     fn test_my_macro_logic() {
//         let input = quote! {
//             struct MyStruct {
//                 field: i32,
//             }
//         };
        
//         let expected = quote! {
//             struct MyStruct {
//                 field: i32,
//             }
            
//             impl MyStruct {
//                 fn new() -> Self {
//                     Self { field: 0 }
//                 }
//             }
//         };
        
//         let result = my_macro_impl(input);
//         assert_eq!(result.to_string(), expected.to_string());
//     }
// }