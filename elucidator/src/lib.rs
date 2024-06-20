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

fn validated_trimmed_or_err(s: &str) -> Result<&str, ElucidatorError> {
    let trimmed = ascii_trimmed_or_err(s)?;
    for c in trimmed.chars() {
        if !c.is_alphanumeric() && c != '_' {
            Err(ElucidatorError::ParsingError{
                offender: s.to_string(),
                reason: Invalidity::IllegalCharacter(c),
            })?;
        }
    }
    Ok(trimmed)
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
        let ss = ascii_trimmed_or_err(s)?;
        todo!("");
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
    IllegalCharacter(char),
    IllegalDataType,
    MissingIdSpecDelimiter,
    UnexpectedEndOfExpression,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DTYPES: [&str; 10] = [
        "u8",
        "u16",
        "u32",
        "u64",
        "i16",
        "i32",
        "i64",
        "f32",
        "f64",
        "string",
    ];

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
    fn dtype_all_ok() {
        for dt in DTYPES {
            assert!(Dtype::from(dt).is_ok());
        }
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
        let invalid_ident = "e ver";
        assert_eq!(
            MemberSpecification::from(&format!("{invalid_ident}: u32")),
            Err(
                ElucidatorError::ParsingError {
                    offender: invalid_ident.to_string(),
                    reason: Invalidity::IllegalCharacter(' '),
                }
            )
        );
    }
}
