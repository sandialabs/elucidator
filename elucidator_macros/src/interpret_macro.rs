use std::fmt;

enum Sizing {
    Singleton,
    Fixed(usize),
    Dynamic,
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


// impl From<&syn::Type> for Type {
//     // Function to match syn type to elucidator type, error if unknown.
//     // impl from
//     fn from(syn_type: &syn::Type) -> Self {
//         match syn_type {
//             syn::Type::Path(type_path) => {
//                 let segment = type_path
//                     .path
//                     .segments
//                     .last()
//                     .expect("Type path should have at least one segment");

//                 match segment.ident.to_string().as_str() {
//                     "u8" => Type::Byte,
//                     "u16" => Type::UnsignedInteger16,
//                     "u32" => Type::UnsignedInteger32,
//                     "u64" => Type::UnsignedInteger64,
//                     "i8" => Type::SignedInteger8,
//                     "i16" => Type::SignedInteger16,
//                     "i32" => Type::SignedInteger32,
//                     "i64" => Type::SignedInteger64,
//                     "f32" => Type::Float32,
//                     "f64" => Type::Float64,
//                     "String" => Type::Str,
//                     "Vec" => panic!("Here!"),
//                     _ => panic!("Unexpected type encountered when deriving interpret."),
//                 }
//             }
//             syn::Type::Array(_syn_arr) => {
//                 unimplemented!("Types from array not implemented")
//             }
//             syn::Type::Reference(_syn_ref) => {
//                 unimplemented!("Types from reference not implemented")
//             }
//             _ => panic!("Unsupported syn::Type variant."),
//         }
//     }
// }


pub fn extract_fields(input: &syn::DeriveInput) -> syn::Result<Vec<&syn::Field>> {
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

pub fn extract_members(fields: &Vec<&syn::Field>) -> syn::Result<Vec<String>> {
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