extern crate proc_macro;
use std::fmt;
use proc_macro::{TokenStream};

use quote::{quote, ToTokens};
use syn::*;

#[derive(PartialEq)]
struct Primitive {
    first_char: char,
    size: u8,
}

impl Primitive {
    fn from(identifier: &str) -> Primitive {
        let first_char = identifier.chars().nth(0).unwrap();
        let size = identifier[1..].parse::<u8>().unwrap();
        Primitive { first_char, size }
    }
    fn is_signed(&self) -> bool {
        self.first_char == 'i' || self.first_char == 'f'
    }
    fn is_integer(&self) -> bool {
        self.first_char != 'f'
    }
    fn is_float(&self) -> bool {
        self.first_char == 'f'
    }
    fn as_string(&self) -> String {
        format!("{}{}", self.first_char, self.size)
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

// Only usable for primitives!! Specifically, u, i, f types. NO chars or bools.
fn attempt_convert(source: &str, target: &str) -> String {
    let source = Primitive::from(source);
    let target = Primitive::from(target);
    let narrow = format!("crate::ElucidatorError::new_narrowing(\"{source}\", \"{target}\")");
    let ok = format!("Ok( *self as {})", target.as_string());

    let return_value = if source == target {
        "Ok(*self)".to_string()
    } else if
        (source.is_signed() && !target.is_signed())
        || (source.is_float() && !target.is_float())
        || (source.size > target.size)
    {
        narrow
    } else if source.is_integer() && target.is_float() {
        if source.size <= target.size / 2 {
            ok
        } else {
            narrow
        }
    } else if source.is_float() && target.is_float() {
        if source.size < target.size {
            ok
        } else {
            narrow
        }
    } else if source.is_integer() && target.is_integer() {
        if !source.is_signed() && target.is_signed() {
            if source.size < target.size {
                ok
            } else {
                narrow
            }
        } else {
            // Source is smaller, signs match
            ok
        }
    } else {
        panic!("else drop: {}, {}", source, target);
    };
    format!("fn as_{}(&self) -> std::result::Result<std::primitive::{}, crate::ElucidatorError> {{ {return_value} }}", target, target)
}

#[proc_macro]
pub fn representable_primitive_impl(item: TokenStream) -> TokenStream {
    let t: Type = syn::parse(item).unwrap();
    let in_path = match &t {
        Type::Path(tp) => tp,
        _ => {panic!("make_representable_impl must be a valid path")}
    };
    let last_ident = &in_path.path.segments.iter().last().unwrap().ident;
    // println!("{last_ident:#?}");
    let string_repr = format!("{last_ident}");
    let this_primitive = Primitive::from(string_repr.as_str());

    let is_numeric = true;
    let is_array = false;
    let is_signed = this_primitive.is_signed();
    let is_integer = this_primitive.is_integer();
    let is_floating = this_primitive.is_float();
    let get_dtype_return = match string_repr.as_str() {
        "u8" => quote! { Dtype::Byte},
        "u16" => quote! { Dtype::UnsignedInteger16 },
        "u32" => quote! { Dtype::UnsignedInteger32 },
        "u64" => quote! { Dtype::UnsignedInteger64 },
        "i8" => quote! { Dtype::SignedInteger8},
        "i16" => quote! { Dtype::SignedInteger16 },
        "i32" => quote! { Dtype::SignedInteger32 },
        "i64" => quote! { Dtype::SignedInteger64 },
        "f32" => quote! { Dtype::Float32 },
        "f64" => quote! { Dtype::Float64 },
        _ => { todo!("Need to add get_dtype_return for {}", string_repr)}
    }.to_token_stream();

    let buffer_conversion = quote! {
        self.to_le_bytes().iter().map(|x| *x).collect()
    }.to_token_stream();

    // Logic for conversions
    let target_types = ["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64"];
    let conversion_text = target_types.iter()
        .map(|x| attempt_convert(this_primitive.as_string().as_str(), x))
        .collect::<Vec<String>>()
        .join("\n");
    let conversion_functions: proc_macro2::TokenStream = conversion_text.parse().unwrap();
    let vec_conversion_text = target_types.iter()
        .map(|x| format!(
            "fn as_vec_{x}(&self) -> std::result::Result<std::vec::Vec<std::primitive::{x}>, crate::ElucidatorError> {{
               crate::ElucidatorError::new_conversion(\"{string_repr}\", \"{x} array\")
            }}\n"
        ))
        .collect::<Vec<String>>()
        .join("\n");
    let vec_conversion_functions: proc_macro2::TokenStream = vec_conversion_text.parse().unwrap();

    let gen = quote! {
        impl Representable for #last_ident {
            fn is_numeric(&self) -> std::primitive::bool { #is_numeric }
            fn is_array(&self) -> std::primitive::bool { #is_array }
            fn is_signed(&self) -> std::primitive::bool { #is_signed }
            fn is_integer(&self) -> std::primitive::bool { #is_integer }
            fn is_floating(&self) -> std::primitive::bool { #is_floating }
            fn get_dtype(&self) -> Dtype { #get_dtype_return }
            fn as_buffer(&self) -> std::vec::Vec<u8> { #buffer_conversion }
            #conversion_functions
            fn as_string(&self) -> std::result::Result<std::string::String, crate::ElucidatorError> {
                crate::ElucidatorError::new_conversion(#string_repr, "string")
            }
            #vec_conversion_functions
        }
    };
    gen.into()
}
