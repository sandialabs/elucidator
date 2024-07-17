use crate::error::*;
use crate::helper;

pub mod dtype;
pub use dtype::{Dtype, Sizing};

#[derive(Debug, PartialEq)]
pub struct MemberSpecification {
    identifier: String,
    typespec: TypeSpecification,
}

impl MemberSpecification {
    pub fn from(s: &str) -> Result<MemberSpecification, ElucidatorError> {
        let ss = helper::ascii_trimmed_or_err(s)?;
        if let Some((ident, typespec)) = ss.split_once(':') {
            helper::validate_identifier(ident)?;
            let ts = TypeSpecification::from(typespec)?;
            Ok(
                Self {
                    identifier: ident.to_string(),
                    typespec: ts,
                }
            )
        } else {
            Err(
                ElucidatorError::Parsing{
                    offender: s.to_string(),
                    reason: ParsingFailure::MissingIdSpecDelimiter,
                }
            )
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TypeSpecification {
    dtype: Dtype,
    sizing: Sizing,
}

impl TypeSpecification {
    pub fn from(s: &str) -> Result<TypeSpecification, ElucidatorError> {
        let type_spec = helper::validated_trimmed_or_err(s)?;
        match type_spec.find('[') {
            Some(lbracket_index) => {
                if type_spec.ends_with(']') {
                    let end_index = type_spec.len() - 1;
                    let inside = &type_spec[(lbracket_index + 1)..end_index];
                    if inside.parse::<u64>().is_ok() {
                        Ok ( Self {
                            dtype: Dtype::from(&type_spec[0..lbracket_index])?,
                            sizing: Sizing::Fixed(inside.parse::<u64>().unwrap()),
                        })
                    } else if inside.chars().all(|c| c.is_whitespace()) {
                        Ok( Self {
                            dtype: Dtype::from(&type_spec[0..lbracket_index])?,
                            sizing: Sizing::Dynamic
                        })
                    }
                    else {
                        Err(ElucidatorError::Parsing {
                            offender: s.to_string(),
                            reason: ParsingFailure::IllegalArraySizing
                        })
                    }
                } else {
                    Err(ElucidatorError::Parsing{
                        offender: s.to_string(),
                        reason: ParsingFailure::UnexpectedEndOfExpression
                    })
                }
            },
            None => {
                Ok( Self { dtype: Dtype::from(type_spec)?, sizing: Sizing::Singleton } )
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typespec_non_ascii_str() {
        let crab_emoji = String::from('\u{1F980}');
        assert_eq!(
            TypeSpecification::from(&crab_emoji),
            Err(
                ElucidatorError::Parsing {
                    offender: crab_emoji,
                    reason: ParsingFailure::NonAsciiEncoding,
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
                ElucidatorError::Parsing {
                    offender: ts.to_string(),
                    reason: ParsingFailure::UnexpectedEndOfExpression,
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
                sizing: Sizing::Fixed(10_u64),
            })
        );
    }
    #[test]
    fn typespec_gets_dynamic_empty() {
        let ts = "u32[]";
        assert_eq!(
            TypeSpecification::from(ts),
            Ok( TypeSpecification {
                dtype: Dtype::UnsignedInteger32,
                sizing: Sizing::Dynamic,
            })
        );
    }
    #[test]
    fn typespec_gets_dynamic_whitespace() {
        let ts = "u32[     ]";
        assert_eq!(
            TypeSpecification::from(ts),
            Ok( TypeSpecification {
                dtype: Dtype::UnsignedInteger32,
                sizing: Sizing::Dynamic,
            })
        );
    }
    #[test]
    fn memberspec_non_ascii_str() {
        let crab_emoji = String::from('\u{1F980}');
        assert_eq!(
            MemberSpecification::from(&crab_emoji),
            Err(
                ElucidatorError::Parsing {
                    offender: crab_emoji,
                    reason: ParsingFailure::NonAsciiEncoding,
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
                ElucidatorError::Parsing {
                    offender: invalid_ident.to_string(),
                    reason: ParsingFailure::IdentifierStartsNonAlphabetical,
                }
            )
        );
    }
    #[test]
    fn memberspec_invalid_identifier_char() {
        let mut illegal_chars = vec!['{', '}', '?'];
        illegal_chars.sort();
        let invalid_ident: String = illegal_chars.iter().collect();
        assert_eq!(
            MemberSpecification::from(&format!("{invalid_ident}: u32")),
            Err(
                ElucidatorError::Parsing {
                    offender: invalid_ident.to_string(),
                    reason: ParsingFailure::IllegalCharacters(illegal_chars),
                }
            )
        );
    }
}
