pub fn syn_err<T, U, V>(
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

pub fn combine_option_errors<I>(iter: I) -> Option<syn::Error>
where
    I: IntoIterator<Item = Option<syn::Error>>
{
    iter
        .into_iter()
        .flatten()
        .reduce(|mut acc, err| {
            acc.combine(err);
            acc
        })
}

pub fn combine_result_errors<I, T>(iter: &I) -> Option<syn::Error>
where
    I: IntoIterator<Item = syn::Result<T>> + Clone,
    T: Clone,
{
    iter.clone()
        .into_iter()
        .filter_map(|result| result.clone().err())
        .reduce(|mut acc, err| {
            acc.combine(err);
            acc
        })
}

pub fn get_last_type_path_segment(
    type_path: &syn::TypePath,
) -> syn::Result<&syn::PathSegment> {
    match type_path.path.segments.last() {
        Some(segment) => syn::Result::Ok(segment),
        None => todo!("Raise error - no segments?"),
    }
}

pub fn get_type_path_from_vec_args(
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