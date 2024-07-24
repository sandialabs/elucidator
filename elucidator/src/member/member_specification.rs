use crate::error::*;
use crate::{parsing, member::TypeSpecification};

#[derive(Debug, PartialEq)]
pub struct MemberSpecification {
    identifier: String,
    typespec: TypeSpecification,
}

impl MemberSpecification {
    pub fn from(s: &str) -> Result<MemberSpecification, ElucidatorError> {
        let ss = parsing::ascii_trimmed_or_err(s)?;
        if s.is_empty() {
            Err(
                ElucidatorError::Parsing{
                    offender: "".to_string(),
                    reason: ParsingFailure::UnexpectedEndOfExpression,
                }
            )?
        }
        if let Some((ident, typespec)) = ss.split_once(':') {
            parsing::validate_identifier(ident)?;
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_ascii_str_err() {
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
    fn invalid_identifier_start_err() {
        let invalid_ident = "5ever";
        assert_eq!(
            MemberSpecification::from(&format!("{invalid_ident}: u32")),
            Err(
                ElucidatorError::IllegalSpecification {
                    offender: invalid_ident.to_string(),
                    reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                }
            )
        );
    }
    #[test]
    fn invalid_identifier_char_err() {
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
    #[test]
    fn empty_err() {
        let spec = "";
        assert_eq!(
            MemberSpecification::from(""),
            Err(
                ElucidatorError::Parsing {
                    offender: spec.to_string(),
                    reason: ParsingFailure::UnexpectedEndOfExpression,
                }
            )
        );
    }
    #[test]
    fn ok() {
        let spec = "foo: i32";
        assert_eq!(
            MemberSpecification::from(spec),
            Ok(
                MemberSpecification {
                    identifier: "foo".to_string(),
                    typespec: TypeSpecification::from("i32").unwrap(),
                }
            )
        );
    }
}
