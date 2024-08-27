use std::collections::HashMap;

use crate::member::{Dtype, Sizing, MemberSpecification};
use crate::token::{TokenClone, DtypeToken, IdentifierToken, SizingToken};
use crate::error::*;
use crate::parsing::*;

type Result<T, E = InternalError> = std::result::Result<T, E>;

fn valid_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub(crate) fn validate_identifier(itoken: &IdentifierToken) -> Result<String> {
    let mut errors: Vec<InternalError> = Vec::new();
    let identifier = itoken.data.data;
    match &identifier.chars().next() {
        None => {
            errors.push(InternalError::IllegalSpecification { 
                offender: TokenClone::from_token_data(&itoken.data),
                reason: SpecificationFailure::ZeroLengthIdentifier
            });
        }
        Some(c) => {
            if !c.is_alphabetic() {
                errors.push(InternalError::IllegalSpecification { 
                    offender: TokenClone::from_token_data(&itoken.data),
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
    if !illegal_chars.is_empty() {
        errors.push(
            InternalError::IllegalSpecification { 
                offender: TokenClone::from_token_data(&itoken.data),
                reason: SpecificationFailure::IllegalCharacters(illegal_chars)
            }
        );
    }
    if errors.is_empty() {
        Ok(identifier.to_string())
    } else {
        Err(InternalError::merge(&errors))
    }
}

pub(crate) fn validate_dtype(dtoken: &DtypeToken) -> Result<Dtype> {
    let s = dtoken.data.data;
    let dt = match s.trim() {
        "u8" => Dtype::Byte,
        "u16" => Dtype::UnsignedInteger16,
        "u32" => Dtype::UnsignedInteger32,
        "u64" => Dtype::UnsignedInteger64,
        "i8"  => Dtype::SignedInteger8,
        "i16" => Dtype::SignedInteger16,
        "i32" => Dtype::SignedInteger32,
        "i64" => Dtype::SignedInteger64,
        "f32" => Dtype::Float32,
        "f64" => Dtype::Float64,
        "string" => Dtype::Str,
        _ => {
            Err(
                InternalError::IllegalSpecification{
                    offender: TokenClone::from_token_data(&dtoken.data),
                    reason: SpecificationFailure::IllegalDataType,
                }   
            )?  
        },  
    };  
    Ok(dt)
}


pub(crate) fn validate_sizing(stoken: &SizingToken) -> Result<Sizing> {
    let data = stoken.data.data;
    let trimmed_data = data.trim();
    if trimmed_data.is_empty() {
        return Ok(Sizing::Dynamic);
    }
    match data.parse::<u64>() {
        Ok(0) | Err(_) => {Err(
            InternalError::IllegalSpecification {
                offender: TokenClone::from_token_data(&stoken.data),
                reason: SpecificationFailure::IllegalArraySizing 
            }
        )},
        Ok(v) => {Ok(Sizing::Fixed(v))},
    }
}

pub(crate) fn validate_memberspec(mpo: &MemberSpecParserOutput) -> Result<MemberSpecification, InternalError> {
    let mut errors: Vec<InternalError> = mpo.errors.clone();

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
                match validate_sizing(stoken) {
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
                InternalError::IllegalSpecification {
                    offender: TokenClone::from_token_data(
                        &mpo.identifier.clone().unwrap().data
                    ),
                    reason: SpecificationFailure::IllegalArraySizing,
                }
            );
            Err(InternalError::merge(&errors)) 
        }
        else {
            Ok(MemberSpecification::from_parts(
                &ident.unwrap(), 
                &sizing.unwrap(), 
                &dtype.unwrap())
            )
        }
    } else {
        Err(InternalError::merge(&errors))
    }
}

fn repeated_identifiers<'a>(member_names: &'a Vec<&'a str>) -> Vec<&'a str> {
    let mut identifier_counts: HashMap<&str, usize> = HashMap::new();
    for identifier in member_names {
        identifier_counts
            .entry(identifier)
            .and_modify(|id| *id += 1)
            .or_insert(1);
    }

    identifier_counts
        .iter()
        .filter(|(_, v)| **v > 1)
        .map(|(k, _)| *k)
        .collect()
}

fn perform_metadata_partition(mpo: &MetadataSpecParserOutput) ->
    (Vec<MemberSpecification>, Vec<Result<MemberSpecification>>)
{
    let results = mpo.member_outputs.iter()
        .map(|x| validate_memberspec(x))
        .collect::<Vec<Result<MemberSpecification>>>();

    type BigResult = Result<MemberSpecification, InternalError>;
    type LongPartition = (Vec<BigResult>, Vec<BigResult>);

    let (members, errs): LongPartition = results
      .into_iter()
      .partition(Result::is_ok);

    let members: Vec<MemberSpecification> = members
        .into_iter()
        .map(Result::unwrap)
        .collect();

    (members, errs)
}

fn err_from_repeat(mpo: &MetadataSpecParserOutput, repeat: &str) -> InternalError {
    // Find matching token
    let hits: Vec<TokenClone> = mpo.member_outputs
        .iter()
        .filter_map(|x| {
            if x.identifier.as_ref().unwrap().data.data == repeat {
                Some(TokenClone::from_token_data(
                    &x.identifier.as_ref().unwrap().data
                ))
            } else {
                None
            }
        })
        .take(2)
        .collect();
    InternalError::IllegalSpecification{
        offender: hits[1].clone(),
        reason: SpecificationFailure::RepeatedIdentifier{
            first: hits[0].clone(),
        }
    }
}

pub(crate) fn validate_metadataspec(mpo: &MetadataSpecParserOutput) -> Result<Vec<MemberSpecification>, InternalError> {
    let mut errors: Vec<InternalError> = mpo.errors.clone();

    let members: Vec<&str> = mpo.member_outputs
        .iter()
        .filter(|x| x.identifier.is_some())
        .map(|x| x.identifier.as_ref().unwrap().data.data)
        .collect();
    
    let (ok_members, errs) = perform_metadata_partition(mpo);
    errs.iter().for_each(|e| {
        errors.push(e.as_ref().unwrap_err().clone())
    });
    repeated_identifiers(&members).iter().for_each(|e| {
        errors.push(err_from_repeat(mpo, e))
    });

    if errors.is_empty() {
        Ok(ok_members)
    } else {
        Err(InternalError::merge(&errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parsing, validating, token::TokenClone, test_utils};
    

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
        fn crab_emoji_err() {
            let ident_text = test_utils::crab_emoji();
            let ipo = parsing::get_identifier(ident_text.as_str(), 0);
            let ident = validating::validate_identifier(&ipo.identifier.unwrap());
            pretty_assertions::assert_eq!(
                ident,
                Err(InternalError::merge(&vec![
                    InternalError::IllegalSpecification {
                        offender: TokenClone::new(ident_text.trim(), 0),
                        reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                    },
                    InternalError::IllegalSpecification {
                        offender: TokenClone::new(ident_text.trim(), 0),
                        reason: SpecificationFailure::IllegalCharacters(
                            vec![
                                test_utils::crab_emoji()
                                    .chars().next()
                                    .unwrap()
                            ]
                        ),
                    },
                ]))
            );

        }

        #[test]
        fn invalid_ident_err() {
            let ident_text = "5foo  ";
            let ipo = parsing::get_identifier(ident_text, 0);
            let ident = validating::validate_identifier(&ipo.identifier.unwrap());
            assert_eq!(
                ident,
                Err(InternalError::IllegalSpecification {
                    offender: TokenClone::new(ident_text.trim(), 0),
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
                Err(InternalError::IllegalSpecification {
                    offender: TokenClone::new(ident_text.to_string().trim(), 1),
                    reason: SpecificationFailure::IllegalCharacters(vec!['\n', '\r', ' ', '(', ')', '\u{85}']),
                })
            );
        }
    }

    mod dtype {
        use super::*;

        use crate::token::TokenData;

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
            let dtype = validating::validate_dtype(
                & DtypeToken {
                    data: TokenData::new(text, 0, 0)
                }
            );
            pretty_assertions::assert_eq!(
                dtype,
                Err(InternalError::IllegalSpecification {
                    offender: TokenClone::new(text, 0),
                    reason: SpecificationFailure::IllegalDataType,
                })
            );
        }
    
        #[test]
        fn leading_whitespace_ok() {
            let text = "\u{85}\tu8";
            let dtype = validating::validate_dtype(
                &parsing::get_dtype(text, 0).dtype.unwrap()
            );
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Byte)
            );
        }
    
        #[test]
        fn trailing_whitespace_ok() {
            let text = "u8   \u{85}";
            let dtype = validating::validate_dtype(
                &parsing::get_dtype(text, 0).dtype.unwrap()
            );
            pretty_assertions::assert_eq!(
                dtype,
                Ok(Dtype::Byte)
            );
        }

        #[test]
        fn null_character() {
            let text = "\0";
            let dtype = validating::validate_dtype(
                &parsing::get_dtype(text, 0).dtype.unwrap()
            );
            pretty_assertions::assert_eq!(
                dtype,
                Err(InternalError::IllegalSpecification {
                    offender: TokenClone::new(text, 0),
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
                Err(InternalError::IllegalSpecification {
                    offender: TokenClone::new(text, 0),
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
                Err(InternalError::IllegalSpecification { 
                    offender: TokenClone::new(text, 0),
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
                Err(InternalError::IllegalSpecification { 
                    offender: TokenClone::new(text, 0),
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
                    InternalError::IllegalSpecification{
                        offender: TokenClone::new(ident, 0),
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
                    InternalError::Parsing{
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
                    InternalError::Parsing{
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
                Err(InternalError::merge(&vec![
                    InternalError::Parsing{
                        offender: TokenClone::new("", 6),
                        reason: ParsingFailure::UnexpectedEndOfExpression,
                    },
                    InternalError::Parsing{
                        offender: TokenClone::new(" ", 5),
                        reason: ParsingFailure::UnexpectedEndOfExpression,
                    },
                    InternalError::IllegalSpecification{
                        offender: TokenClone::new("5eva", 0),
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
                Err(InternalError::merge(&vec![
                    InternalError::IllegalSpecification{
                        offender: TokenClone::new("5eva", 0),
                        reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                    },
                    InternalError::IllegalSpecification{
                        offender: TokenClone::new("cat", 10),
                        reason: SpecificationFailure::IllegalArraySizing,
                    },
                ]))
            );
        }
    }

    mod metadata_spec {
        use super::*;

        #[test]
        fn metadata_single_ok() {
            let text = "foo: u32";
            let mpo = parsing::get_metadataspec(text);
            let spec = validating::validate_metadataspec(&mpo);
            pretty_assertions::assert_eq!(
                spec,
                Ok(vec![
                    MemberSpecification::from_parts(
                        "foo", &Sizing::Singleton, &Dtype::UnsignedInteger32
                    ),
                ])
            );
        }

        #[test]
        fn metadata_multiple_ok() {
            let text = "foo: u32, bar: u8[], baz: string";
            let mpo = parsing::get_metadataspec(text);
            let spec = validating::validate_metadataspec(&mpo);
            pretty_assertions::assert_eq!(
                spec,
                Ok(vec![
                    MemberSpecification::from_parts(
                        "foo", &Sizing::Singleton, &Dtype::UnsignedInteger32
                    ),
                    MemberSpecification::from_parts(
                        "bar", &Sizing::Dynamic, &Dtype::Byte
                    ),
                    MemberSpecification::from_parts(
                        "baz", &Sizing::Singleton, &Dtype::Str
                    ),
                ])
            );
        }

        #[test]
        fn metadata_mixed_ok_err() {
            let text = "5ever: u32, bar: u8[], baz: string[5]";
            let mpo = parsing::get_metadataspec(text);
            let spec = validating::validate_metadataspec(&mpo);
            pretty_assertions::assert_eq!(
                spec,
                Err(InternalError::merge(&vec![
                    InternalError::IllegalSpecification {
                        offender: TokenClone {
                            data: "5ever".to_string(),
                            column_start: 0,
                            column_end: 5,
                        },
                        reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                    },
                    InternalError::IllegalSpecification {
                        offender: TokenClone {
                            data: "baz".to_string(),
                            column_start: 23,
                            column_end: 26,
                        },
                        reason: SpecificationFailure::IllegalArraySizing,
                    },
                ])),
            );
        }

        #[test]
        fn metadata_repeated_identifier_err() {
            let text = "foo: u32, foo: u8[]";
            let mpo = parsing::get_metadataspec(text);
            let spec = validating::validate_metadataspec(&mpo);
            pretty_assertions::assert_eq!(
                spec,
                Err(InternalError::IllegalSpecification {
                    offender: TokenClone {
                        data: "foo".to_string(),
                        column_start: 10,
                        column_end: 13,
                    },
                    reason: SpecificationFailure::RepeatedIdentifier{
                        first: TokenClone {
                            data: "foo".to_string(),
                            column_start: 0,
                            column_end: 3,
                        },
                    }
                })
            );
        }

        #[test]
        fn metadata_repeated_identifier_one_wrong_err() {
            let text = "foo: bar, foo: u32";
            let mpo = parsing::get_metadataspec(text);
            let spec = validating::validate_metadataspec(&mpo);
            pretty_assertions::assert_eq!(
                spec,
                Err(InternalError::merge(&vec![
                    InternalError::IllegalSpecification {
                        offender: TokenClone::new("bar", 5),
                        reason: SpecificationFailure::IllegalDataType,
                    },
                    InternalError::IllegalSpecification {
                        offender: TokenClone {
                            data: "foo".to_string(),
                            column_start: 10,
                            column_end: 13,
                        },
                        reason: SpecificationFailure::RepeatedIdentifier{
                            first: TokenClone {
                                data: "foo".to_string(),
                                column_start: 0,
                                column_end: 3,
                            },
                        }
                    },
                ]))
            );
        }

    }
}
