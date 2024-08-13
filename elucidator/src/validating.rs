use crate::{error::*};
use crate::parsing::{DtypeParserOutput, IdentifierParserOutput};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

fn valid_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn validate_identifier(ipo: &IdentifierParserOutput) -> Result<String> {
    let mut errors: Vec<ElucidatorError> = Vec::new();
    if !ipo.errors.is_empty() {
        Err(ElucidatorError::merge(&ipo.errors))?
    }
    if ipo.identifier.is_none() {
        unreachable!("Could not validate identifier: no errors found, but no identifier found either.");
    }
    let identifier = ipo.identifier.as_ref().unwrap().data.data;
    match &identifier.chars().nth(0) {
        None => {
            errors.push(ElucidatorError::IllegalSpecification { 
                offender: identifier.to_string(), 
                reason: SpecificationFailure::ZeroLengthIdentifier
            });
        }
        Some(c) => {
            if !c.is_alphabetic() {
                errors.push(ElucidatorError::IllegalSpecification { 
                    offender: identifier.to_string(), 
                    reason: SpecificationFailure::IdentifierStartsNonAlphabetical
                });
            }
        }
    }
    
    let mut illegal_chars: Vec<char> = identifier
        .chars()
        .filter(|c| !valid_identifier_char(*c))
        .collect();
    illegal_chars.sort();
    illegal_chars.dedup();
    if illegal_chars.len() > 0 {
        errors.push(
            ElucidatorError::IllegalSpecification { 
                offender: identifier.to_string(), 
                reason: SpecificationFailure::IllegalCharacters(illegal_chars)
            }
        );
    }
    if errors.is_empty() {
        Ok(identifier.to_string())
    } else {
        Err(ElucidatorError::merge(&errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parsing, validating};
    use pretty_assertions;

    mod identifier {
        use super::*;

        #[test]
        fn valid_ident_ok() {
            let ident_text = "foo10";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo);
            assert_eq!(ident, Ok("foo10".to_string()));
        }

        #[test]
        fn valid_ident_whitespace_ok() {
            let ident_text = "  foo10  ";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo);
            assert_eq!(ident, Ok("foo10".to_string()));
        }

        #[test]
        fn invalid_ident_err() {
            let ident_text = "5foo  ";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo);
            assert_eq!(
                ident,
                Err(ElucidatorError::IllegalSpecification {
                    offender: "5foo".to_string(),
                    reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                })
            );
        }

        #[test]
        fn invalid_whitespace_in_ident() {
            let ident_text = " foo \r\n\u{85}bar()\t";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo);
            pretty_assertions::assert_eq!(
                ident,
                Err(ElucidatorError::IllegalSpecification {
                    offender: "foo \r\n\u{85}bar()".to_string(),
                    reason: SpecificationFailure::IllegalCharacters(vec!['\n', '\r', ' ', '(', ')', '\u{85}']),
                })
            );
        }

        

    }
}