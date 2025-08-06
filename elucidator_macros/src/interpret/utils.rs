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
        None => unreachable!("Raise error - no segments?"),
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
                syn_err(angled_args, "No arguments supplied to Vec")?
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
            "Vec requires arguments"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syn_err_ok() {
        let tokens: syn::Ident = syn::parse_quote!(valid_name);
        let result: syn::Result<()> = syn_err(tokens, "foobar");
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), "foobar");
    }

   
    #[test]
    fn test_syn_combine_option_all_err_ok() {
        let tokens_foo: syn::Ident = syn::parse_quote!(valid_name);
        let result_foo: syn::Result<()> = syn_err(tokens_foo, "foo");
        let tokens_bar: syn::Ident = syn::parse_quote!(valid_name);
        let result_bar: syn::Result<()> = syn_err(tokens_bar, "bar");

        let result = combine_option_errors([
            result_foo.err(),
            result_bar.err(),
        ]);
        
        assert!(result.is_some());
        let error = result.unwrap().to_compile_error().to_string();
        assert!(error.contains("foo"));
        assert!(error.contains("bar"));
    }

    #[test]
    fn test_syn_combine_option_mixed_err_ok() {
        let tokens_foo: syn::Ident = syn::parse_quote!(valid_name);
        let result_foo: syn::Result<()> = syn_err(tokens_foo, "foo");
        let result_bar = syn::Result::Ok(());
        let tokens_baz: syn::Ident = syn::parse_quote!(valid_name);
        let result_baz: syn::Result<()> = syn_err(tokens_baz, "baz");

        let result = combine_option_errors([
            result_foo.err(),
            result_bar.err(),
            result_baz.err(),
        ]);
        
        assert!(result.is_some());
        let error = result.unwrap().to_compile_error().to_string();
        assert!(error.contains("foo"));
        assert!(error.contains("baz"));
    }

    #[test]
    fn test_syn_combine_result_all_err_ok() {
        let tokens_foo: syn::Ident = syn::parse_quote!(valid_name);
        let result_foo: syn::Result<()> = syn_err(tokens_foo, "foo");
        let tokens_bar: syn::Ident = syn::parse_quote!(valid_name);
        let result_bar: syn::Result<()> = syn_err(tokens_bar, "bar");

        let result = combine_result_errors(&[
            result_foo,
            result_bar,
        ]);
        
        assert!(result.is_some());
        let error = result.unwrap().to_compile_error().to_string();
        assert!(error.contains("foo"));
        assert!(error.contains("bar"));
    }
    
    #[test]
    fn test_syn_combine_result_mixed_err_ok() {
        let tokens_foo: syn::Ident = syn::parse_quote!(valid_name);
        let result_foo: syn::Result<()> = syn_err(tokens_foo, "foo");
        let result_bar = syn::Result::Ok(());
        let tokens_baz: syn::Ident = syn::parse_quote!(valid_name);
        let result_baz: syn::Result<()> = syn_err(tokens_baz, "baz");

        let result = combine_result_errors(&[
            result_foo,
            result_bar,
            result_baz,
        ]);
        
        assert!(result.is_some());
        let error = result.unwrap().to_compile_error().to_string();
        assert!(error.contains("foo"));
        assert!(error.contains("baz"));
    }

    #[test]
    fn test_get_last_type_path_ok() {
        let type_path: syn::TypePath = syn::parse_quote!(std::primitive::u8);
        let last_segment = get_last_type_path_segment(&type_path);

        assert!(last_segment.is_ok());
        let last_segment = last_segment.unwrap();
        assert_eq!(last_segment.ident.to_string(), "u8");
        assert!(last_segment.arguments.is_none());
    }

    #[test]
    fn test_get_type_path_from_vec_args_ok() {
        let path_segment: syn::PathSegment = syn::parse_quote!(Vec<std::primitive::u8>);
        let gt: syn::TypePath = syn::parse_quote!(std::primitive::u8);
        let type_path = get_type_path_from_vec_args(&path_segment);

        assert!(type_path.is_ok());
        let type_path = type_path.unwrap();
        assert_eq!(type_path, &gt);
    }

    #[test]
    fn test_get_type_path_from_vec_args_not_vec_fails() {
        let path_segment: syn::PathSegment = syn::parse_quote!(VecDeque<std::primitive::u8>);
        let type_path = get_type_path_from_vec_args(&path_segment);

        assert!(type_path.is_err());
        let error = type_path.err().unwrap();
        assert_eq!(error.to_string(), "Expected a Vec, got 'VecDeque'");
    }

    #[test]
    fn test_get_type_path_from_vec_args_no_vec_args_fails() {
        let path_segment: syn::PathSegment = syn::parse_quote!(Vec<>);
        let type_path = get_type_path_from_vec_args(&path_segment);

        assert!(type_path.is_err());
        let error = type_path.err().unwrap();
        assert_eq!(error.to_string(), "No arguments supplied to Vec");
    }

    #[test]
    fn test_get_type_path_from_vec_args_not_type_path_fails() {
        let path_segment: syn::PathSegment = syn::parse_quote!(Vec<(u8, u8)>);
        let type_path = get_type_path_from_vec_args(&path_segment);

        assert!(type_path.is_err());
        let error = type_path.err().unwrap();
        assert_eq!(error.to_string(), "Unexpected type format, expected a type path");
    }

    #[test]
    fn test_get_type_path_from_vec_args_not_angled_brackets_fails() {
        let path_segment: syn::PathSegment = syn::parse_quote!(Vec);
        let type_path = get_type_path_from_vec_args(&path_segment);

        assert!(type_path.is_err());
        let error = type_path.err().unwrap();
        assert_eq!(error.to_string(), "Vec requires arguments");
    }
}