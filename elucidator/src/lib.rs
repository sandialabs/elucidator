//! Main elucidator library.
//!

/// Represent array sizing for a Member.
/// Generally not useful except when constructing Members for users, though it is used in this
/// library.
/// ```
/// use elucidator::Sizing;
///
/// // Fixed Sizing of 10
/// let fixed_size = Sizing::Fixed(10 as u64);
/// // Dynamic Sizing based on the identifier "len"
/// let dynamic_size = Sizing::Dynamic("len".to_string());
/// assert_eq!(fixed_size, Sizing::Fixed(10));
/// assert_eq!(dynamic_size, Sizing::Dynamic("len".to_string()));
/// ```
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Sizing {
    Singleton,
    Fixed(u64),
    Dynamic(String),
}

/// Possible Data Types allowed in The Elucidation Metadata Standard, most composable as arrays.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Dtype {
    Byte,
    UnsignedInteger16,
    UnsignedInteger32,
    UnsignedInteger64,
    SignedInteger16,
    SignedInteger32,
    SignedInteger64,
    Float32,
    Float64,
    Str,
}

impl Dtype {
    fn from(s: &str) -> Result<Dtype, ElucidatorError> {
        let dt = match ascii_trimmed_or_err(s)? {
            "u8" => Self::Byte,
            "u16" => Self::UnsignedInteger16,
            "u32" => Self::UnsignedInteger32,
            "u64" => Self::UnsignedInteger64,
            "i16" => Self::SignedInteger16,
            "i32" => Self::SignedInteger32,
            "i64" => Self::SignedInteger64,
            "f32" => Self::Float32,
            "f64" => Self::Float64,
            "string" => Self::Str,
            ss => {
                Err(
                    ElucidatorError::ParsingError{
                        offender: ss.to_string(),
                        reason: Invalidity::IllegalDataType,
                    }
                )?
            },
        };
        Ok(dt)
    }
}


fn ascii_or_err(s: &str) -> Result<(), ElucidatorError> {
    if !s.is_ascii() { 
            Err(
                ElucidatorError::ParsingError{
                    offender: s.to_string(),
                    reason: Invalidity::NonAsciiEncoding
                }
            )
    } else {
        Ok(())
    }

}

fn ascii_trimmed_or_err(s: &str) -> Result<&str, ElucidatorError> {
    ascii_or_err(s)?;
    Ok(s.trim())
}

fn is_valid_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '[' || c == ']'
}

fn validated_trimmed_or_err(s: &str) -> Result<&str, ElucidatorError> {
    let trimmed = ascii_trimmed_or_err(s)?;
    let illegal_chars = trimmed
        .chars()
        .filter(|c| !is_valid_char(*c))
        .collect::<Vec<char>>();
    if illegal_chars.len() == 0 {
        Ok(trimmed)
    } else {
        Err(ElucidatorError::ParsingError{
            offender: s.to_string(),
            reason: Invalidity::IllegalCharacters(illegal_chars),
        })
    }
}

fn validate_identifier(s: &str) -> Result<&str, ElucidatorError> {
    let ss = validated_trimmed_or_err(s)?;
    match ss.chars().nth(0) {
        Some(c) => {
            if !c.is_alphabetic() {
                Err(
                    ElucidatorError::ParsingError{
                        offender: s.to_string(),
                        reason: Invalidity::IdentifierStartsNonAlphabetical,
                    }
                )?;
            }
        },
        None => {
            Err(ElucidatorError::ParsingError{offender: s.to_string(), reason: Invalidity::UnexpectedEndOfExpression})?;
        },
    }
    Ok(ss)
}

/// Full specification for any type
#[derive(Debug, PartialEq)]
pub struct TypeSpecification {
    dtype: Dtype,
    sizing: Sizing,
}

impl TypeSpecification {
    fn from(s: &str) -> Result<TypeSpecification, ElucidatorError> {
        let ss = validated_trimmed_or_err(s)?;
        match ss.find("[") {
            Some(lbracket_index) => {
                if ss.chars().last().unwrap() == ']' {
                    let end_index = ss.len() - 1;
                    let inside = &ss[(lbracket_index + 1)..end_index];
                    if inside.parse::<u64>().is_ok() {
                        Ok ( Self { 
                            dtype: Dtype::from(&ss[0..lbracket_index])?,
                            sizing: Sizing::Fixed(inside.parse::<u64>().unwrap()),
                        })
                    } else {
                        Ok ( Self {
                            dtype: Dtype::from(&ss[0..lbracket_index])?,
                            sizing: Sizing::Dynamic(
                                validate_identifier(inside)?.to_string()
                            ),
                        })
                    }
                } else {
                    Err(ElucidatorError::ParsingError{
                        offender: s.to_string(),
                        reason: Invalidity::UnexpectedEndOfExpression
                    })
                }
            },
            None => {
                Ok( Self { dtype: Dtype::from(ss)?, sizing: Sizing::Singleton } )
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MemberSpecification {
    identifier: String,
    typespec: TypeSpecification,
}

impl MemberSpecification {
    fn from(s: &str) -> Result<MemberSpecification, ElucidatorError> {
        let ss = ascii_trimmed_or_err(s)?;
        if let Some((ident, typespec)) = ss.split_once(":") {
            validate_identifier(ident)?;
            let ts = TypeSpecification::from(typespec)?;
            Ok(
                Self {
                    identifier: ident.to_string(),
                    typespec: ts,
                }
            )
        } else {
            Err(
                ElucidatorError::ParsingError{
                    offender: s.to_string(),
                    reason: Invalidity::MissingIdSpecDelimiter,
                }
            )
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum ElucidatorError {
    ParsingError{offender: String, reason: Invalidity}
}

#[derive(Debug, PartialEq)]
pub enum Invalidity {
    NonAsciiEncoding,
    IdentifierStartsNonAlphabetical,
    IllegalCharacters(Vec<char>),
    IllegalDataType,
    MissingIdSpecDelimiter,
    UnexpectedEndOfExpression,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn get_dtype_map() -> HashMap<&'static str, Result<Dtype, ElucidatorError>> {
        HashMap::from([
            ("u8",      Ok(Dtype::Byte)),
            ("u16",     Ok(Dtype::UnsignedInteger16)),
            ("u32",     Ok(Dtype::UnsignedInteger32)),
            ("u64",     Ok(Dtype::UnsignedInteger64)),
            ("i16",     Ok(Dtype::SignedInteger16)),
            ("i32",     Ok(Dtype::SignedInteger32)),
            ("i64",     Ok(Dtype::SignedInteger64)),
            ("f32",     Ok(Dtype::Float32)),
            ("f64",     Ok(Dtype::Float64)),
            ("string",  Ok(Dtype::Str))
        ])
    }

    #[test]
    fn dtype_non_ascii_str() {
        let crab_emoji = String::from('\u{1F980}');
        assert_eq!(
            Dtype::from(&crab_emoji),
            Err(
                ElucidatorError::ParsingError {
                    offender: crab_emoji,
                    reason: Invalidity::NonAsciiEncoding,
                }
            )
        );
    }
    #[test]
    fn dtype_illegal_dtype() {
        let invalid_dtype = "e32";
        assert_eq!(
            Dtype::from(invalid_dtype),
            Err(
                ElucidatorError::ParsingError {
                    offender: invalid_dtype.to_string(),
                    reason: Invalidity::IllegalDataType,
                }
            )
        );

    }
    #[test]
    fn dtype_all_parsed_correct() {
        let result_map = get_dtype_map()
            .keys()
            .map(|x| {(*x, Dtype::from(x))})
            .collect::<HashMap<&str, Result<Dtype, ElucidatorError>>>();
        assert_eq!(result_map, get_dtype_map());
    }
    #[test]
    fn typespec_non_ascii_str() {
        let crab_emoji = String::from('\u{1F980}');
        assert_eq!(
            TypeSpecification::from(&crab_emoji),
            Err(
                ElucidatorError::ParsingError {
                    offender: crab_emoji,
                    reason: Invalidity::NonAsciiEncoding,
                }
            )
        );
    }
    #[test]
    fn typespec_fails_nesting() {
        let ts = "u32[10][10]";
        assert!(TypeSpecification::from(ts).is_err());
    }
    #[test]
    fn typespec_fails_missing_rbracket() {
        let ts = "u32[10";
        assert!(TypeSpecification::from(ts).is_err());
    }
    #[test]
    fn typespec_fails_extra_lbracket() {
        let ts = "u32[[10]";
        assert!(TypeSpecification::from(ts).is_err());
    }
    #[test]
    fn typespec_fails_extra_rbracket() {
        let ts = "u32[10]]";
        assert!(TypeSpecification::from(ts).is_err());
    }
    #[test]
    fn typespec_fails_extra_chars() {
        let ts = "u32[10]stuff";
        assert_eq!(
            TypeSpecification::from(ts),
            Err(
                ElucidatorError::ParsingError {
                    offender: ts.to_string(),
                    reason: Invalidity::UnexpectedEndOfExpression,
                }
            )
        );
    }
    #[test]
    fn typespec_gets_singleton() {
        let ts = "u32";
        assert_eq!(
            TypeSpecification::from(ts),
            Ok( TypeSpecification {
                dtype: Dtype::UnsignedInteger32,
                sizing: Sizing::Singleton,
            })
        );
    }
    #[test]
    fn typespec_gets_fixed() {
        let ts = "u32[10]";
        assert_eq!(
            TypeSpecification::from(ts),
            Ok( TypeSpecification {
                dtype: Dtype::UnsignedInteger32,
                sizing: Sizing::Fixed(10 as u64),
            })
        );
    }
    #[test]
    fn typespec_gets_dynamic() {
        let ts = "u32[cat]";
        assert_eq!(
            TypeSpecification::from(ts),
            Ok( TypeSpecification {
                dtype: Dtype::UnsignedInteger32,
                sizing: Sizing::Dynamic("cat".to_string()),
            })
        );
    }
    #[test]
    fn memberspec_non_ascii_str() {
        let crab_emoji = String::from('\u{1F980}');
        assert_eq!(
            MemberSpecification::from(&crab_emoji),
            Err(
                ElucidatorError::ParsingError {
                    offender: crab_emoji,
                    reason: Invalidity::NonAsciiEncoding,
                }
            )
        );

    }
    #[test]
    fn memberspec_invalid_identifier_start() {
        let invalid_ident = "5ever";
        assert_eq!(
            MemberSpecification::from(&format!("{invalid_ident}: u32")),
            Err(
                ElucidatorError::ParsingError {
                    offender: invalid_ident.to_string(),
                    reason: Invalidity::IdentifierStartsNonAlphabetical,
                }
            )
        );
    }
    #[test]
    fn memberspec_invalid_identifier_char() {
        let illegal_chars = vec!['{', '}', ' ', '?'];
        let invalid_ident: String = illegal_chars.iter().collect();
        assert_eq!(
            MemberSpecification::from(&format!("{invalid_ident}: u32")),
            Err(
                ElucidatorError::ParsingError {
                    offender: invalid_ident.to_string(),
                    reason: Invalidity::IllegalCharacters(illegal_chars),
                }
            )
        );
    }
}
