use crate::member::{Dtype, Sizing};
use crate::{error::*};
use crate::parsing::{DtypeParserOutput, IdentifierParserOutput, SizingParserOutput};

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

pub fn validate_dtype(dpo: &DtypeParserOutput) -> Result<Dtype> {
    if !dpo.errors.is_empty() {
        Err(ElucidatorError::merge(&dpo.errors))?
    }
    match &dpo.dtype {
        None => {
            unreachable!("Could not validate dtype: no errors found, but no dtype found either.")
        },
        Some(dtoken) => {
            Dtype::from(dtoken.data.data)
        }
    }
}


pub fn validate_sizing(spo: &SizingParserOutput) -> Result<Sizing, ElucidatorError> {
    if !spo.errors.is_empty() {
        Err(ElucidatorError::merge(&spo.errors))?;
    }
    match &spo.sizing {
        None => {
            unreachable!("Could not validate sizing: no errors found, but no sizing found either.")
        },
        Some(stoken) => {
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

    mod dtype {
        use super::*;

        #[test]
        fn u8_ok() {
            let text = "u8";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Byte)
            );
        }
        #[test]
        fn u16_ok() {
            let text = "u16";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::UnsignedInteger16)
            );
        }
        #[test]
        fn u32_ok() {
            let text = "u32";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::UnsignedInteger32)
            );
        }
        #[test]
        fn u64_ok() {
            let text = "u64";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::UnsignedInteger64)
            );
        }
        #[test]
        fn i8_ok() {
            let text = "i8";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger8)
            );
        }
        #[test]
        fn i16_ok() {
            let text = "i16";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger16)
            );
        }
        #[test]
        fn i32_ok() {
            let text = "i32";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger32)
            );
        }
        #[test]
        fn i64_ok() {
            let text = "i64";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::SignedInteger64)
            );
        }
        #[test]
        fn f32_ok() {
            let text = "f32";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Float32)
            );
        }
        #[test]
        fn f64_ok() {
            let text = "f64";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Float64)
            );
        }
        #[test]
        fn string_ok() {
            let text = "string";
            let dpo = parsing::get_dtype(text, 0);
            let dtype = validating::validate_dtype(&dpo);
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
            let sizing = validating::validate_sizing(&spo);
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Dynamic)
            );
        }

        
        #[test]
        fn dynamic_whitespace_ok() {
            let text = "  \u{85}";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo);
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Dynamic)
            );
        }

        #[test]
        fn fixed_ok() {
            let text = "10";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo);
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Fixed(10))
            );
        }

        
        #[test]
        fn fixed_whitespace_ok() {
            let text = "  10\u{85}";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo);
            pretty_assertions::assert_eq!(
                sizing,
                Ok(Sizing::Fixed(10))
            );
        }

        #[test]
        fn fixed_zero() {
            let text = "0";
            let spo = parsing::get_sizing(text, 0);
            let sizing = validating::validate_sizing(&spo);
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
            let sizing = validating::validate_sizing(&spo);
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
            let sizing = validating::validate_sizing(&spo);
            pretty_assertions::assert_eq!(
                sizing,
                Err(ElucidatorError::IllegalSpecification { 
                    offender: text.to_string(),
                    reason: SpecificationFailure::IllegalArraySizing,
                })
            );
        }         
    }
    mod type_spec {
        use super::*;
    }
}