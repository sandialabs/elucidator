use crate::error::*;
use crate::{helper, member::TypeSpecification};

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


#[cfg(test)]
mod tests {
    use super::*;

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
