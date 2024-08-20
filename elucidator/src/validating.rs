use std::error;

use crate::member::{Dtype, Sizing, MemberSpecification};
use crate::token::{DtypeToken, IdentifierToken, SizingToken};
use crate::{error::*, Representable};
use crate::parsing::{DtypeParserOutput, IdentifierParserOutput, MemberSpecParserOutput, SizingParserOutput, TypeSpecParserOutput};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

fn valid_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn validate_identifier(itoken: &IdentifierToken) -> Result<String> {
    let mut errors: Vec<ElucidatorError> = Vec::new();
    let identifier = itoken.data.data;
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

pub fn validate_dtype(dtoken: &DtypeToken) -> Result<Dtype> {
    Dtype::from(dtoken.data.data)
}


pub fn validate_sizing(stoken: &SizingToken) -> Result<Sizing, ElucidatorError> {
    let data = stoken.data.data;
    let trimmed_data = data.trim();
    if trimmed_data.is_empty() {
        return Ok(Sizing::Dynamic);
    }
    match data.parse::<u64>() {
        Ok(0) | Err(_) => {Err(
            ElucidatorError::IllegalSpecification { 
                offender: data.to_string(),
                reason: SpecificationFailure::IllegalArraySizing 
            }
        )},
        Ok(v) => {Ok(Sizing::Fixed(v))},
    }
}

pub fn validate_memberspec(mpo: &MemberSpecParserOutput) -> Result<MemberSpecification, ElucidatorError> {
    let mut errors: Vec<ElucidatorError> = mpo.errors.clone();

    let ident = if mpo.has_ident() {
        match validate_identifier(&mpo.identifier.clone().unwrap()) {
            Ok(o) => { Some(o) },
            Err(e) => { 
                errors.push(e);
                None
            },
        } 
    } else {
        None
    };

    let dtype = if mpo.has_dtype() {
        match validate_dtype(&mpo.typespec.clone().unwrap().dtype.unwrap()) {
            Ok(o) => { Some(o) },
            Err(e) => { 
                errors.push(e);
                None
            },
        }  
    } else {
        None
    };

    let sizing = if mpo.has_sizing() {
        let typespec = mpo.typespec.clone().unwrap();
        match &typespec.sizing {
            Some(stoken) => { 
                match validate_sizing(&stoken) {
                    Ok(o) => { Some(o) },
                    Err(e) => { 
                        errors.push(e);
                        None 
                    },
                }
            },
            None => { Some(Sizing::Singleton) },
        }  
    } else {
        None
    };

    if ident.is_some() && dtype.is_some() && sizing.is_some() {
        if !errors.is_empty() {
            unreachable!("Parsed and validated MemberSpecification, but errors were also found: {:#?}", errors);
        }
        if dtype.clone().unwrap() == Dtype::Str && sizing.clone().unwrap() != Sizing::Singleton {
            errors.push(
                ElucidatorError::IllegalSpecification {
                    offender: ident.clone().unwrap(),
                    reason: SpecificationFailure::IllegalArraySizing,
                }
            );
            Err(ElucidatorError::merge(&errors)) 
        }
        else {
            Ok(MemberSpecification::from_parts(
                &ident.unwrap(), 
                &sizing.unwrap(), 
                &dtype.unwrap())
            )
        }
    } else {
        Err(ElucidatorError::merge(&errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parsing, validating, token::TokenClone};
    use pretty_assertions;

    mod identifier {
        use super::*;

        #[test]
        fn valid_ident_ok() {
            let ident_text = "foo10";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo.identifier.unwrap());
            assert_eq!(ident, Ok("foo10".to_string()));
        }

        #[test]
        fn valid_ident_whitespace_ok() {
            let ident_text = "  foo10  ";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo.identifier.unwrap());
            assert_eq!(ident, Ok("foo10".to_string()));
        }

        #[test]
        fn invalid_ident_err() {
            let ident_text = "5foo  ";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo.identifier.unwrap());
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
            let ident = validating::validate_identifier(&ipo.identifier.unwrap());
            pretty_assertions::assert_eq!(
                ident,
                Err(ElucidatorError::IllegalSpecification {
                    offender: "foo \r\n\u{85}bar()".to_string(),
                    reason: SpecificationFailure::IllegalCharacters(vec!['\n', '\r', ' ', '(', ')', '\u{85}']),
                })
            );
        }
    }

    mod dtype {
        use super::*;

        #[test]
        fn u8_ok() {
            let text = "u8";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Byte)
            );
        }
        #[test]
        fn u16_ok() {
            let text = "u16";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::UnsignedInteger16)
            );
        }
        #[test]
        fn u32_ok() {
            let text = "u32";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::UnsignedInteger32)
            );
        }
        #[test]
        fn u64_ok() {
            let text = "u64";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::UnsignedInteger64)
            );
        }
        #[test]
        fn i8_ok() {
            let text = "i8";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger8)
            );
        }
        #[test]
        fn i16_ok() {
            let text = "i16";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger16)
            );
        }
        #[test]
        fn i32_ok() {
            let text = "i32";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger32)
            );
        }
        #[test]
        fn i64_ok() {
            let text = "i64";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger64)
            );
        }
        #[test]
        fn f32_ok() {
            let text = "f32";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Float32)
            );
        }
        #[test]
        fn f64_ok() {
            let text = "f64";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Float64)
            );
        }
        #[test]
        fn string_ok() {
            let text = "string";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo.dtype.unwrap());
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Str)
            );
        }
        #[test]
        fn empty_string() {
            let text = "";
            let dtype = Dtype::from(text);
            pretty_assertions::assert_eq!(
                dtype,
                Err(ElucidatorError::IllegalSpecification {
                    offender: text.to_string(),
                    reason: SpecificationFailure::IllegalDataType,
                })
            );
        }
    
        #[test]
        fn leading_whitespace_ok() {
            let text = "\u{85}\tu8";
            let dtype = Dtype::from(text);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Byte)
            );
        }
    
        #[test]
        fn trailing_whitespace_ok() {
            let text = "u8   \u{85}";
            let dtype = Dtype::from(text);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Byte)
            );
        }

        #[test]
        fn null_character() {
            let text = "\0";
            let dtype = Dtype::from(text);
            pretty_assertions::assert_eq!(
                dtype,
                Err(ElucidatorError::IllegalSpecification {
                    offender: text.to_string(),
                    reason: SpecificationFailure::IllegalDataType,
                })
            );
        }
    }
    mod sizing {
        use super::*;

        #[test]
        fn dynamic_ok() {
            let text = "";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo.sizing.unwrap());
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Dynamic)
            );
        }

        
        #[test]
        fn dynamic_whitespace_ok() {
            let text = "  \u{85}";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo.sizing.unwrap());
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Dynamic)
            );
        }

        #[test]
        fn fixed_ok() {
            let text = "10";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo.sizing.unwrap());
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Fixed(10))
            );
        }

        
        #[test]
        fn fixed_whitespace_ok() {
            let text = "  10\u{85}";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo.sizing.unwrap());
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Fixed(10))
            );
        }

        #[test]
        fn fixed_zero() {
            let text = "0";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo.sizing.unwrap());
            pretty_assertions::assert_eq!(
                sizing,
                Err(ElucidatorError::IllegalSpecification {
                    offender: text.to_string(),
                    reason: SpecificationFailure::IllegalArraySizing
            }));
        } 

        #[test]
        fn fixed_negative_fails() {
            let text = "-10";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo.sizing.unwrap());
            pretty_assertions::assert_eq!(
                sizing,
                Err(ElucidatorError::IllegalSpecification { 
                    offender: text.to_string(),
                    reason: SpecificationFailure::IllegalArraySizing,
                })
            );
        }

        #[test]
        fn fixed_text_fails() {
            let text = "foobar";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo.sizing.unwrap());
            pretty_assertions::assert_eq!(
                sizing,
                Err(ElucidatorError::IllegalSpecification { 
                    offender: text.to_string(),
                    reason: SpecificationFailure::IllegalArraySizing,
                })
            );
        }         
    }
    mod memberspec {
        use super::*;

        #[test]
        fn singleton_ok() {
            let text = "foo: u32";
            let mpo = parsing::get_memberspec(text, 0);
            let member = validating::validate_memberspec(&mpo);
            pretty_assertions::assert_eq!(
                member,
                Ok(
                    MemberSpecification::from_parts(
                        "foo",
                        &Sizing::Singleton,
                        &Dtype::UnsignedInteger32,
                    )
                )
            );
        }

        #[test]
        fn string_non_singleton_err() {
            let ident = "foo";
            let text = &format!("{ident}: string[]");
            let mpo = parsing::get_memberspec(text, 0);
            let member = validating::validate_memberspec(&mpo);
            pretty_assertions::assert_eq!(
                member,
                Err(
                    ElucidatorError::IllegalSpecification{
                        offender: ident.to_string(),
                        reason: SpecificationFailure::IllegalArraySizing,
                    },
                )
            );
        }

        #[test]
        fn empty_err() {
            let text = "";
            let mpo = parsing::get_memberspec(text, 0);
            let member = validating::validate_memberspec(&mpo);
            assert!(member.is_err());
        }

        #[test]
        fn ident_missing_err() {
            let text = ": u32";
            let mpo = parsing::get_memberspec(text, 0);
            let member = validating::validate_memberspec(&mpo);
            // TODO: convert this after error refactor for promoting EOE to ZeroLengthIdentifer
            pretty_assertions::assert_eq!(
                member,
                Err(
                    ElucidatorError::Parsing{
                        offender: TokenClone::new("", 0),
                        reason: ParsingFailure::UnexpectedEndOfExpression,
                    }
                )
            )
        }

        #[test]
        fn dtype_missing_err() {
            let text = "foo: []";
            let mpo = parsing::get_memberspec(text, 0);
            let member = validating::validate_memberspec(&mpo);
            // TODO: convert this after error refactor for promoting EOE to ZeroLengthIdentifer
            pretty_assertions::assert_eq!(
                member,
                Err(
                    ElucidatorError::Parsing{
                        offender: TokenClone::new(" ", 4),
                        reason: ParsingFailure::UnexpectedEndOfExpression,
                    }
                )
            )
        }

        #[test]
        fn multiple_failures_parsing_spec_err() {
            let text = "5eva: [";
            let mpo = parsing::get_memberspec(text, 0);
            let member = validating::validate_memberspec(&mpo);
            pretty_assertions::assert_eq!(
                member,
                Err(ElucidatorError::merge(&vec![
                    ElucidatorError::Parsing{
                        offender: TokenClone::new("", 6),
                        reason: ParsingFailure::UnexpectedEndOfExpression,
                    },
                    ElucidatorError::Parsing{
                        offender: TokenClone::new(" ", 5),
                        reason: ParsingFailure::UnexpectedEndOfExpression,
                    },
                    ElucidatorError::IllegalSpecification{
                        offender: "5eva".to_string(),
                        reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                    },
                ]))
            );
        }

        #[test]
        fn multiple_failures_spec_err() {
            let text = "5eva: u32[cat]";
            let mpo = parsing::get_memberspec(text, 0);
            let member = validating::validate_memberspec(&mpo);
            pretty_assertions::assert_eq!(
                member,
                Err(ElucidatorError::merge(&vec![
                    ElucidatorError::IllegalSpecification{
                        offender: "5eva".to_string(),
                        reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                    },
                    ElucidatorError::IllegalSpecification{
                        offender: "cat".to_string(),
                        reason: SpecificationFailure::IllegalArraySizing,
                    },
                ]))
            );
        }
    }
}
