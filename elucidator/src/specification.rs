use crate::{
    error::*,
    parsing,
    member::MemberSpecification,
};

use std::collections::HashSet;
use std::collections::HashMap;
use std::vec::Vec;


type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

/// Representation of a Metadata Specification. The designation is the identifier associated with
/// an ordered set of member specifications. This represents only the specification; an Interpreter
/// must be used to extract values from data buffers/blobs.
#[derive(Debug, PartialEq)]
pub struct MetadataSpecification {
    designation: String,
    members: Vec<MemberSpecification>,
}
impl MetadataSpecification {
    fn from_str(designation: &str, specification_text: &str) -> Result<Self> {
        let mut errors: Vec<ElucidatorError> = Vec::new();
        let validated_designation = MetadataSpecification::validate_designation(designation);
        let validated_members = MetadataSpecification::validate_members(specification_text);
        if validated_members.is_ok() && validated_designation.is_ok() {
            Ok(MetadataSpecification {
                designation: validated_designation.unwrap(),
                members: validated_members.unwrap(),
            })
        }
        else {
            if validated_designation.is_err() {
                errors.push(validated_designation.err().unwrap())
            }
            if validated_members.is_err() {
                errors.push(validated_members.err().unwrap())
            }
            if errors.len() == 1 {
                let first_error = errors.into_iter().next().unwrap();
                Err(first_error)
            }
            else {
                Err(ElucidatorError::MultipleFailures(Box::new(errors)))
            }
        }
    }


    fn parse_designation(designation: &str) -> Result<&str> {
        let trimmed = parsing::ascii_trimmed_or_err(designation)?;
        let mut illegal_chars: Vec<char> = trimmed.chars()
            .filter(|c| !parsing::is_valid_identifier_char(*c))
            .collect();
        illegal_chars.sort();
        illegal_chars.dedup();

        if illegal_chars.is_empty() {
            Ok(trimmed)
        }
        else {
            Err(
                ElucidatorError::Parsing {
                    offender: designation.to_string(),
                    reason: ParsingFailure::IllegalCharacters(illegal_chars)
            })
        }
    }

    fn parse_spec_text(specification_text: &str) -> Vec<Result<MemberSpecification>> {
        let sections: Vec<&str> = specification_text.split(",")
            .map(|s| s.trim())
            .collect();
        if sections.len() == 1 && sections.first().unwrap().len() == 0 {
            // Special case, we allow a spec text with no members
            Vec::new()
        }
        else {
            let members: Vec<Result<MemberSpecification>> = sections
                .iter()
                .map(|s| MemberSpecification::from(*s))
                .collect();
            members
            // let (members, errors): (Vec<_>, Vec<_>) = sections.iter()
            //     .map(|s| MemberSpecification::from(s))
            //     .partition(Result::is_ok);

            // let members: Vec<MemberSpecification> = members.into_iter()
            //     .map(Result::unwrap)
            //     .collect();

            // let errors: Vec<ElucidatorError> = errors.into_iter()
            //     .map(Result::unwrap_err)
            //     .collect();

            // if errors.is_empty() {
            //     Ok(members)
            // }
            // else if errors.len() == 1 {
            //     let first_error = errors.into_iter().next().unwrap();
            //     Err(first_error)
            // }
            // else {
            //     Err(ElucidatorError::MultipleFailures(
            //         Box::new(errors)
            //     ))
            // }
        }
    }
    fn validate_designation(designation: &str) -> Result<String> {
        let mut errors: Vec<ElucidatorError> = Vec::new();
        let parsed_designation = MetadataSpecification::parse_designation(designation)?;

        match (&parsed_designation).chars().next() {
            Some(char) => {
                if !char.is_alphabetic() {
                    errors.push(ElucidatorError::IllegalSpecification{
                        offender: designation.to_string(),
                        reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                    });
                }
            }
            None => {
                errors.push(ElucidatorError::IllegalSpecification {
                    offender: designation.to_string(),
                    reason: SpecificationFailure::ZeroLengthIdentifier
                });
            },
        }
        if errors.is_empty() {
            Ok(parsed_designation.to_string())
        }
        else if errors.len() == 1 {
            let first_error = errors.into_iter().next().unwrap();
            Err(first_error)
        }
        else {
            Err(ElucidatorError::MultipleFailures(Box::new(errors)))
        }
    }

    fn validate_members(specification_text: &str) -> Result<Vec<MemberSpecification>> {
        // let mut errors: Vec<ElucidatorError> = Vec::new();
        let (members, errors): (Vec<_>, Vec<_>) = MetadataSpecification::parse_spec_text(specification_text)
            .into_iter()
            .partition(Result::is_ok);

        let members: Vec<MemberSpecification> = members.into_iter()
            .map(Result::unwrap)
            .collect();

        let mut errors: Vec<ElucidatorError> = errors.into_iter()
            .map(Result::unwrap_err)
            .collect();


        todo!();
        // let names: HashSet<String> = HashSet::from_iter(
        //     members.iter().map(|m| m.identifier.clone())
        // );
        // let repeated_names: Vec<String> = members.iter()
        //     .map(|m| m.identifier.clone())
        //     .fold(HashMap::new(), |mut acc, name| {
        //         *acc.entry(name).or_insert(0) += 1;
        //         acc
        //     })
        //     .into_iter()
        //     .filter(|&(_, count)| count > 1)
        //     .map(|(name, _)| name)
        //     .collect();

        // if repeated_names.is_empty() {
        //     Ok(members)
        // }
        // if names.len() != members.len() {
        //     Err(ElucidatorError::IllegalSpecification { 
        //         offender: specification_text.to_string(),
        //         reason: SpecificationFailure::RepeatedIdentifier,
        //     })
        // }
        // else {
            
        // }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    mod designation {
        use crate::{
            error::*,
            parsing,
            member,
            test_utils,
        };
        use super::MetadataSpecification;

        #[test]
        fn not_ascii_err() {
            let designation = test_utils::crab_emoji();
            let md_spec = MetadataSpecification::from_str(
                &designation.as_str(),
                "foo: i32, bar: u8",
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::Parsing { 
                    offender: designation,
                    reason: ParsingFailure::NonAsciiEncoding,
                })
            );
        }
        #[test]
        fn non_alphabetical_start_err() {
            let designation = "123abc";
            let md_spec = MetadataSpecification::from_str(
                &designation,
                "foo: i32, bar: u8",
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::IllegalSpecification { 
                    offender: designation.to_string(),
                    reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                })
            );
        }
        #[test]
        fn designation_first_char_illegal_err() {
            let designation = "[my_designation]";
            let md_spec = MetadataSpecification::from_str(
                &designation,
                "foo: i32, bar: u8",
            );
            assert!(md_spec.is_err());
        }
        #[test]
        fn contains_illegal_chars_err() {
            let designation = "my_designation[]";
            let md_spec = MetadataSpecification::from_str(
                &designation,
                "foo: i32, bar: u8",
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::Parsing { 
                    offender: designation.to_string(),
                    reason: ParsingFailure::IllegalCharacters(vec!['[', ']']),
                })
            );
        }
        #[test]
        fn is_ok() {
            let designation = "my_designation";
            let md_spec = MetadataSpecification::from_str(
                &designation,
                "foo: i32, bar: u8",
            );

            let members = vec![
                member::MemberSpecification::from("foo: i32").unwrap(),
                member::MemberSpecification::from("bar: u8").unwrap(),
            ];
            let gt_md_spec = MetadataSpecification{
                designation: designation.to_string(),
                members,
            };
            assert_eq!(
                md_spec,
                Ok(gt_md_spec)
            );
        }
    }

    mod spec_text {
        use crate::{
            error::*,
            test_utils
        };

        use super::MetadataSpecification;
        
        #[test]
        fn not_ascii_err() {
            let designation = "my_designation";
            let spec_text = test_utils::crab_emoji();
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text.as_str(),
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::Parsing { 
                    offender: spec_text.to_string(),
                    reason: ParsingFailure::NonAsciiEncoding,
                })
            );
        }
        #[test]
        fn contains_illegal_chars_err() {
            let designation = "my_designation";
            let spec_text = "foo[]: i32, bar: u8"; 
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text,
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::Parsing { 
                    offender: spec_text.to_string(),
                    reason: ParsingFailure::IllegalCharacters(vec!['[', ']']),
                })
            );
        }
        #[test]
        fn illegal_first_char_err() {
            let designation = "my_designation";
            let spec_text = "<foo>: i32, bar: u8"; 
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text,
            );
            assert!(md_spec.is_err());
        }
        #[test]
        fn member_repeated_err() {
            let designation = "my_designation";
            let spec_text = "foo: i32, foo: u8"; 
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text,
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::IllegalSpecification { 
                    offender: "foo".to_string(),
                    reason: SpecificationFailure::RepeatedIdentifier,
                })
            );
        }
        #[test]
        fn invalid_dtype_err() {
            let designation = "my_designation";
            let spec_text = "foo: i32, bar: u9"; 
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text,
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::IllegalSpecification { 
                    offender: "u9".to_string(),
                    reason: SpecificationFailure::IllegalDataType,
                })
            );
        }
        #[test]
        fn multiple_failures_err() {
            let designation = "my_designation";
            let spec_text = "foo: i32, foo: u9"; 
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text,
            );

            let dtype_error = ElucidatorError::IllegalSpecification { 
                offender: spec_text.to_string(),
                reason: SpecificationFailure::IllegalDataType,
            };
            let repeated_identifier_error = ElucidatorError::IllegalSpecification { 
                offender: "foo".to_string(),
                reason: SpecificationFailure::RepeatedIdentifier,
            };
            let multiple_errors = vec![dtype_error, repeated_identifier_error];

            assert_eq!(
                md_spec,
                Err(ElucidatorError::MultipleFailures(Box::new(multiple_errors)))
            );
        }
        #[test]
        fn unexpected_eoe_after_delimiter() {
            let designation = "my_designation";
            let spec_text = "foo: i32,"; 
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text,
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::Parsing { 
                    offender: spec_text.to_string(),
                    reason: ParsingFailure::UnexpectedEndOfExpression,
                })
            );
        }
        #[test]
        fn unexpected_eoe_in_dtype() {
            let designation = "my_designation";
            let spec_text = "foo: [i32, bar: u8"; 
            let md_spec = MetadataSpecification::from_str(
                designation,
                spec_text,
            );
            assert_eq!(
                md_spec,
                Err(ElucidatorError::Parsing { 
                    offender: spec_text.to_string(),
                    reason: ParsingFailure::UnexpectedEndOfExpression,
                })
            );
        }
    }
}
