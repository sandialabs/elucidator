extern crate proc_macro;
use proc_macro::TokenStream;
use std::fmt;

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
    } else if (source.is_signed() && !target.is_signed())
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

// Only usable for primitives!! Specifically, u, i, f types. NO chars or bools.
fn attempt_convert_vec(source: &str, target: &str) -> String {
    let source = Primitive::from(source);
    let target = Primitive::from(target);
    let narrow =
        format!("crate::ElucidatorError::new_narrowing(\"{source} array\", \"{target} array\")");
    let ok = format!(
        "Ok(self.iter().map(|x| *x as {}).collect())",
        target.as_string()
    );

    let return_value = if source == target {
        "Ok(self.to_vec())".to_string()
    } else if (source.is_signed() && !target.is_signed())
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
    format!("fn as_vec_{}(&self) -> std::result::Result<std::vec::Vec<std::primitive::{}>, crate::ElucidatorError> {{ {return_value} }}", target, target)
}

#[proc_macro]
pub fn representable_primitive_impl(item: TokenStream) -> TokenStream {
    let t: syn::Type = syn::parse(item).unwrap();
    let in_path = match &t {
        syn::Type::Path(tp) => tp,
        _ => {
            panic!("make_representable_impl must be a valid path")
        }
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
        _ => {
            todo!("Need to add get_dtype_return for {}", string_repr)
        }
    }
    .to_token_stream();

    let buffer_conversion = quote! {
        self.to_le_bytes().iter().map(|x| *x).collect()
    }
    .to_token_stream();

    // Logic for conversions
    let target_types = [
        "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64",
    ];
    let conversion_text = target_types
        .iter()
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

#[proc_macro]
pub fn representable_vec_impl(item: TokenStream) -> TokenStream {
    let t: syn::Type = syn::parse(item).unwrap();
    let in_path = match &t {
        syn::Type::Path(tp) => tp,
        _ => {
            panic!("make_representable_impl must be a valid path")
        }
    };
    let last_ident = &in_path.path.segments.iter().last().unwrap().ident;
    // println!("{last_ident:#?}");
    let string_repr = format!("{last_ident}");
    let string_repr_arr = format!("{last_ident} array");

    let this_primitive = Primitive::from(string_repr.as_str());

    let is_numeric = true;
    let is_array = true;
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
        _ => {
            todo!("Need to add get_dtype_return for {}", string_repr)
        }
    }
    .to_token_stream();

    let buffer_conversion = quote! {
        let length = self.len() * std::mem::size_of::<#last_ident>();
        let mut buffer: std::vec::Vec<u8> = std::vec::Vec::with_capacity(length);
        for item in self {
            let mut item_buffer = item.as_buffer();
            buffer.append(&mut item_buffer);
        }
        buffer
    }
    .to_token_stream();

    // Logic for conversions
    let target_types = [
        "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64",
    ];
    let conversion_text = target_types
        .iter()
        .map(|x| {
            format!(
        "fn as_{x}(&self) -> std::result::Result<std::primitive::{x}, crate::ElucidatorError> {{
           crate::ElucidatorError::new_conversion(\"{string_repr} array\", \"{x}\")
        }}\n"
    )
        })
        .collect::<Vec<String>>()
        .join("\n");
    let conversion_functions: proc_macro2::TokenStream = conversion_text.parse().unwrap();
    let vec_conversion_text = target_types
        .iter()
        .map(|x| attempt_convert_vec(this_primitive.as_string().as_str(), x))
        .collect::<Vec<String>>()
        .join("\n");
    let vec_conversion_functions: proc_macro2::TokenStream = vec_conversion_text.parse().unwrap();

    let gen = quote! {
        impl Representable for std::vec::Vec<#last_ident> {
            fn is_numeric(&self) -> std::primitive::bool { #is_numeric }
            fn is_array(&self) -> std::primitive::bool { #is_array }
            fn is_signed(&self) -> std::primitive::bool { #is_signed }
            fn is_integer(&self) -> std::primitive::bool { #is_integer }
            fn is_floating(&self) -> std::primitive::bool { #is_floating }
            fn get_dtype(&self) -> Dtype { #get_dtype_return }
            fn as_buffer(&self) -> std::vec::Vec<u8> { #buffer_conversion }
            #conversion_functions
            fn as_string(&self) -> std::result::Result<std::string::String, crate::ElucidatorError> {
                crate::ElucidatorError::new_conversion(#string_repr_arr, "string")
            }
            #vec_conversion_functions
        }
    };
    gen.into()
}

#[proc_macro]
pub fn make_dtype_interpreter(item: TokenStream) -> TokenStream {
    let t: syn::Type = syn::parse(item).unwrap();
    let in_path = match &t {
        syn::Type::Path(tp) => tp,
        _ => {
            panic!("make_representable_impl must be a valid path")
        }
    };
    let last_ident = &in_path.path.segments.iter().last().unwrap().ident;
    let signature: proc_macro2::TokenStream = format!(
        "fn interpret_{last_ident}(
            cursor: &mut Cursor<&[u8]>,
            items_to_read: usize,
            sizing: &Sizing,
        ) -> Result<Box<dyn Representable>>
        "
    )
    .parse()
    .unwrap();

    let buffer_conversion = quote! {
        #signature {
            let item_width = std::mem::size_of::<#last_ident>();
            let bytes_to_read = items_to_read * item_width;
            let mut result_buffer: std::vec::Vec<u8> = std::vec::Vec::with_capacity(bytes_to_read);
            get_n_bytes_from_buff(cursor, &mut result_buffer, bytes_to_read)?;
            let mut item_buff: std::vec::Vec<std::primitive::u8> = std::vec::Vec::with_capacity(std::mem::size_of::<#last_ident>());
            let mut item_cursor = std::io::Cursor::new(result_buffer.as_slice());
            let mut result: std::vec::Vec<#last_ident> = Vec::with_capacity(items_to_read);
            for _ in 0..items_to_read {
                item_buff.clear();
                get_n_bytes_from_buff(&mut item_cursor, &mut item_buff, item_width)?;
                result.push(#last_ident::from_le_bytes(item_buff[0..item_width].try_into().unwrap()));
            }
            if sizing == &Sizing::Singleton {
                return Ok(std::boxed::Box::new(result[0]));
            }
            Ok(std::boxed::Box::new(result))
        }
    }.to_token_stream();
    buffer_conversion.into()
}

/// Derived macro logic
/// vvvvvvvvvvvvvvvvvvv

#[proc_macro_derive(Interpret)]
pub fn derive_interpret(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let fields = match extract_fields(&input) {
        Ok(fields) => fields,
        Err(error) => return error.to_compile_error().into(),
    };

    let members = extract_members(&fields);

    let member_string = match members {
        Ok(strings) => strings.join(", "),
        Err(error) => return error.to_compile_error().into(),
    };

    // return syn::Error::new_spanned(fields[0], member_string).to_compile_error().into();

    let impl_block = quote! {
        impl Interpret for #name {
            fn get_spec() -> DesignationSpecification {
                // todo!("Derived implementation of get_spec not implemented.")
                DesignationSpecification::from_text(#member_string).unwrap()
            }
            fn from_eluciblob() -> Result<Self, ElucidatorError> { todo!("Derived implementation of from_eluciblob not implemented.") }
            fn to_eluciblob() -> Result<Self, ElucidatorError> { todo!("Derived implementation of to_eluciblob not implemented.") }
        }
    }.to_token_stream();
    impl_block.into()
}

fn extract_fields(input: &syn::DeriveInput) -> syn::Result<Vec<&syn::Field>> {
    // Extract fields from the struct
    match &input.data {
        syn::Data::Struct(data_struct) => {
            match &data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    // For structs with named fields like: struct Foo { field1: Type1, field2: Type2 }
                    Ok(fields_named.named.iter().collect::<Vec<_>>())
                }
                syn::Fields::Unnamed(_fields_unnamed) => Err(syn::Error::new_spanned(
                    &input,
                    "Interpret can only be derived for structs with named fields",
                )),
                syn::Fields::Unit => {
                    // For unit structs like: struct Foo;
                    Ok(Vec::new())
                }
            }
        }
        _ => {
            // Not a struct - return a compile error
            Err(syn::Error::new_spanned(
                &input,
                "Interpret can only be derived for structs with named fields.",
            ))
        }
    }
}

fn extract_members(fields: &Vec<&syn::Field>) -> Result<Vec<String>> {
    let mut combined_error: Option<syn::Error> = None;
    let mut member_strings: Vec<String> = Vec::new();

    for f in fields {
        let mut current_error: Option<syn::Error> = None;
        let member_ident: Option<String>;
        let member_dtype: Option<Type>;
        let member_sizing: Option<Sizing>;

        member_ident = accumulate_syn_result(
&mut current_error, 
            get_member_ident(f)
        );
        member_sizing = accumulate_syn_result(
            &mut current_error, 
            get_member_sizing(f)
        );
        member_dtype = accumulate_syn_result(
            &mut current_error, 
            get_member_type(f)
        );
        
        match current_error {
            None => {
               let member_string = format!(
                    "{}: {}{}",
                    member_ident.unwrap(),
                    member_dtype.unwrap(),
                    member_sizing.unwrap(),
               );
               member_strings.push(member_string); 
            },
            Some(err) => {
                accumulate_errors(
                    &mut combined_error, 
                    Some(err)
                );
            },
        }
    }

    match combined_error {
        Some(err) => syn::Result::Err(err),
        None => syn::Result::Ok(member_strings)
    }
}

enum Sizing {
    Singleton,
    Fixed(usize),
    Dynamic,
}

enum Type {
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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            Type::Byte => "u8",
            Type::UnsignedInteger16 => "u16",
            Type::UnsignedInteger32 => "u32",
            Type::UnsignedInteger64 => "u64",
            Type::SignedInteger8 => "i8",
            Type::SignedInteger16 => "i16",
            Type::SignedInteger32 => "i32",
            Type::SignedInteger64 => "i64",
            Type::Float32 => "f32",
            Type::Float64 => "f64",
            Type::Str => "String",
        };
        write!(f, "{}", type_str)
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

impl From<&syn::Type> for Type {
    // Function to match syn type to elucidator type, error if unknown.
    // impl from
    fn from(syn_type: &syn::Type) -> Self {
        match syn_type {
            syn::Type::Path(type_path) => {
                let segment = type_path
                    .path
                    .segments
                    .last()
                    .expect("Type path should have at least one segment");

                match segment.ident.to_string().as_str() {
                    "u8" => Type::Byte,
                    "u16" => Type::UnsignedInteger16,
                    "u32" => Type::UnsignedInteger32,
                    "u64" => Type::UnsignedInteger64,
                    "i8" => Type::SignedInteger8,
                    "i16" => Type::SignedInteger16,
                    "i32" => Type::SignedInteger32,
                    "i64" => Type::SignedInteger64,
                    "f32" => Type::Float32,
                    "f64" => Type::Float64,
                    "String" => Type::Str,
                    "Vec" => panic!("Here!"),
                    _ => panic!("Unexpected type encountered when deriving interpret."),
                }
            }
            syn::Type::Array(_syn_arr) => {
                unimplemented!("Types from array not implemented")
            }
            syn::Type::Reference(_syn_ref) => {
                unimplemented!("Types from reference not implemented")
            }
            _ => panic!("Unsupported syn::Type variant."),
        }
    }
}

fn get_member_ident(field: &syn::Field) -> syn::Result<String> {
    match field.ident.as_ref() {
        Some(ident) => {
            if !ident.to_string().chars().next().unwrap().is_alphabetic() {
                syn::Result::Err(syn::Error::new_spanned(
                    field,
                    format!(
                        "The identifier '{}' is invalid. Identifiers must start with an alphabetical character.",
                        ident,
                    )
                )) 
            }
            else {
                syn::Result::Ok(ident.to_string())
            }
        },
        None => syn::Result::Err(syn::Error::new_spanned(
            field,
            "Field does not have an identifier.",
        )),
    }
}

fn get_member_sizing(field: &syn::Field) -> syn::Result<Sizing> {
    match &field.ty {
        syn::Type::Array(arr) => { 
            match &arr.len {
                syn::Expr::Lit(lit_expr) => {
                    match &lit_expr.lit {
                        syn::Lit::Int(lit_int) => {
                            match lit_int.base10_parse::<usize>() {
                                Ok(n) => syn::Result::Ok(Sizing::Fixed(n)),
                                Err(_) => {
                                    syn::Result::Err(
                                        syn::Error::new_spanned(
                                            &arr.len,
                                            "Could not interpret expression as a literal integer.",
                                        )
                                    )
                                }
                            }
                        },
                        _ => {
                            syn::Result::Err(
                                syn::Error::new_spanned(
                                    &arr.len,
                                    "Unexpected expression, Interpret can only be derive fixed arrays with literal integer expressions.",
                                )
                            ) 
                        }
                    }
                },
                _ => { 
                    syn::Result::Err(
                        syn::Error::new_spanned(
                            &arr.len,
                            "Unexpected expression, Interpret can only be derive fixed arrays with literal integer expressions.",
                        )
                    )
                }
            }
        },
        // syn::Type::Array(arr) => {
        //     let len: Result<usize> = match &arr.len {
        //         syn::Expr::Lit(lit_expr) => match &lit_expr.lit {
        //             syn::Lit::Int(int_lit) => {
        //                 match int_lit.base10_parse::<usize>() {
        //                     Ok(parsed_int) => { syn::Result::Ok(Sizing::Fixed(parsed_int)) },
        //                     Err(error) => { unimplemented!("TODO") },
        //                 }
        //                 todo!();
        //                 // .expect("Failed to parse array length")
        //             },
        //             _ => panic!("Array length must be an integer literal"),
        //         },
        //         _ => {
        //             syn::Result::Err(
        //                 syn::Error::new_spanned(
        //                     &field.ty, 
        //                     "Must use literal expression in array length for derived interpretter.",
        //                 )
        //             )
        //         }
        //     }
        //     // syn::Result::Ok(Sizing::Fixed(len))
        // }
        syn::Type::Path(_type_path) => {
            // This probably needs to look for vectors.
            // unimplemented!("Implement regular types");
            syn::Result::Ok(Sizing::Singleton)
        },
        _ => {
            panic!("Unexpected sizing when deriving interpret");
        }
    }
}

fn get_member_type(field: &syn::Field) -> syn::Result<Type> {
    match &field.ty {
        syn::Type::Path(type_path) => {
            extract_type_from_type_path(&field.ty, type_path)
        }
        syn::Type::Array(arr) => {
            match &*arr.elem {
                syn::Type::Path(type_path) => {
                    extract_type_from_type_path(&field.ty, type_path)
                },
                _ => {
                    syn::Result::Err(
                    syn::Error::new_spanned(
                        arr,
                        "Type associated with field is not included in the list of allowable primitive types.")) 
                },
            }
        },
        // syn::Type::Reference(type_ref) => {
        //     // Handle &str specifically
        //     unimplemented!("References for &str are not implemented.")
        // }
        _ => { panic!("Panic! At the field type") },
    }
}

fn extract_type_from_type_path(
    context: &syn::Type,
    type_path: &syn::TypePath,
) -> syn::Result<Type> {
    let segment = type_path.path.segments.last().unwrap();
    let segment_type = match segment.ident.to_string().as_str() {
        "u8" => syn::Result::Ok(Type::Byte),
        "u16" => syn::Result::Ok(Type::UnsignedInteger16),
        "u32" => syn::Result::Ok(Type::UnsignedInteger32),
        "u64" => syn::Result::Ok(Type::UnsignedInteger64),
        "i8" => syn::Result::Ok(Type::SignedInteger8),
        "i16" => syn::Result::Ok(Type::SignedInteger16),
        "i32" => syn::Result::Ok(Type::SignedInteger32),
        "i64" => syn::Result::Ok(Type::SignedInteger64),
        "f32" => syn::Result::Ok(Type::Float32),
        "f64" => syn::Result::Ok(Type::Float64),
        "String" => syn::Result::Ok(Type::Str),
        "Vec" => {
            match &segment.arguments {
                syn::PathArguments::None => panic!("No args?"),
                syn::PathArguments::Parenthesized(_) => panic!("Not sure how we got there"),
                syn::PathArguments::AngleBracketed(angled) => {
                    match angled.args.first() {
                        None => panic!("There should be a type!"),
                        Some(gen_arg) => {
                            match gen_arg {
                                syn::GenericArgument::Type(ty) => {
                                    match ty {
                                        syn::Type::Path(type_path) => todo!("Implement path"),
                                        _ => unreachable!("This isn't valid syntax"),
                                    }
                                },
                                _ => unreachable!("This isn't a type, how did you do this?"),
                            }
                        }
                    }
                    panic!("This is what we expect!")
                }
            }
            syn::Result::Ok(Type::Str)
        },
        unexpected => syn::Result::Err(syn::Error::new_spanned(
            context,
            format!("Unexpected field type: '{}'", unexpected)
        )),
    };
    segment_type
}

fn extract_vec_type_arg(
    context: &syn::Type,
    type_path: &syn::TypePath,
) -> syn::Result<Type> {
    let segment = type_path.path.segments.last().unwrap();
    let segment_type = match segment.ident.to_string().as_str() { 
        "Vec" => {
            match &segment.arguments {
                syn::PathArguments::AngleBracketed(angled) => {
                    match angled.args.first() {
                        None => panic!("There should be a type!"),
                        Some(gen_arg) => {
                            match gen_arg {
                                syn::GenericArgument::Type(ty) => {
                                    match ty {
                                        syn::Type::Path(type_path) => {
                                            extract_primitive_type_from_type_path(context, type_path)
                                        },
                                        _ => unreachable!("This isn't valid syntax"),
                                    }
                                },
                                _ => unreachable!("This isn't a type, how did you do this?"),
                            }
                        }
                    }
                },
                _ => todo!("Raise syn error for non angled args"),
            }
        },
        _ => unreachable!("How did I get here???"),
    };
    segment_type
}

fn extract_primitive_type_from_type_path(
    context: &syn::Type,
    type_path: &syn::TypePath,
) -> syn::Result<Type> {
    let segment = type_path.path.segments.last().unwrap();
    // match type_path.path.segments.last() {
    //     None
    // }
    match segment.ident.to_string().as_str() {
        "u8" => syn::Result::Ok(Type::Byte),
        "u16" => syn::Result::Ok(Type::UnsignedInteger16),
        "u32" => syn::Result::Ok(Type::UnsignedInteger32),
        "u64" => syn::Result::Ok(Type::UnsignedInteger64),
        "i8" => syn::Result::Ok(Type::SignedInteger8),
        "i16" => syn::Result::Ok(Type::SignedInteger16),
        "i32" => syn::Result::Ok(Type::SignedInteger32),
        "i64" => syn::Result::Ok(Type::SignedInteger64),
        "f32" => syn::Result::Ok(Type::Float32),
        "f64" => syn::Result::Ok(Type::Float64),
        "String" => syn::Result::Ok(Type::Str),
        unexpected => create_syn_err_result(
            context, 
            format!("Unexpected field type: '{}'", unexpected),
        ),
    }
}

fn create_syn_err_result<T, U, V>(
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
// // fn make_syn_err(context: )
// fn accumulate_syn_error(accumulated: mut Option<syn::Error>, newest: Option<syn::Error>) -> Option<syn::Error> {
//     match (accumulated, newest)  {
//         (acc, None) => acc.take(),

//         (_, _) => todo!("Not implemented yet."),
//         // (None, _) => accumulated,
//         // (Some(newest), None) => Some(newest),
//         // (Some(newest), Some(accumulated)) => accumulated.combine(newest),
//     }
// }

// fn accumulate_syn_error(accumulated: &Option<syn::Error>, newest: &Option<syn::Error>) -> Option<syn::Error> {
//     match (&accumulated, &newest) {
//         (Some(acc), None) => Some(acc.clone()),
//         (None, Some(new)) => Some(new.clone()),
//         (Some(acc), Some(new)) => {
//             acc.clone().combine(new.clone());
//             Some(acc.clone())
//         },
//         (None, None) => None,
//     }
// }

fn accumulate_errors(
    accumulated: &mut Option<syn::Error>,
    newest: Option<syn::Error>
) {
    match (accumulated.as_mut(), newest) {
        (None, None) => {}
        (None, Some(err)) => *accumulated = Some(err),
        (Some(_), None) => {}
        (Some(acc), Some(new)) => {
            acc.combine(new);
        }
    }
}

fn accumulate_syn_result<T>(
    accumulated: &mut Option<syn::Error>,
    result: syn::Result<T>
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(err) => {
            accumulate_errors(accumulated, Some(err));
            None
        }
    }
}