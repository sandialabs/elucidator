use std::collections::HashSet;
use crate::error::*;

pub(crate) fn buff_size_or_err<T>(buffer: &[u8]) -> Result<usize, ElucidatorError> {
    let expected_buff_size = std::mem::size_of::<T>();
    if buffer.len() != expected_buff_size {
        Err(ElucidatorError::BufferSizing { expected: expected_buff_size, found: buffer.len() })?
    }
    Ok(expected_buff_size)
}

pub(crate) fn ascii_or_err(s: &str) -> Result<(), ElucidatorError> {
    if !s.is_ascii() {
            Err(
                ElucidatorError::Parsing{
                    offender: s.to_string(),
                    reason: ParsingFailure::NonAsciiEncoding
                }
            )
    } else {
        Ok(())
    }

}

pub(crate) fn ascii_trimmed_or_err(s: &str) -> Result<&str, ElucidatorError> {
    ascii_or_err(s)?;
    Ok(s.trim())
}

pub(crate) fn is_valid_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '[' || c == ']' ||  c.is_whitespace()
}

pub(crate) fn validated_trimmed_or_err(s: &str) -> Result<&str, ElucidatorError> {
    let trimmed = ascii_trimmed_or_err(s)?;
    let illegal_chars = trimmed
        .chars()
        .filter(|c| !is_valid_char(*c))
        .collect::<Vec<char>>();
    if illegal_chars.is_empty() {
        Ok(trimmed)
    } else {
        Err(ElucidatorError::Parsing{
            offender: s.to_string(),
            reason: ParsingFailure::IllegalCharacters(illegal_chars),
        })
    }
}

pub(crate) fn validate_identifier(s: &str) -> Result<&str, ElucidatorError> {
    let ss = validated_trimmed_or_err(s)?;
    if ss.chars().any(|c| c.is_whitespace()) {
        let mut illegal_chars: Vec<char> = ss.chars()
            .filter(|c| c.is_whitespace())
            .collect::<HashSet<char>>()
            .into_iter()
            .collect();
        illegal_chars.sort();
        Err(
            ElucidatorError::Parsing {
                offender: s.to_string(),
                reason: ParsingFailure::IllegalCharacters(illegal_chars)
        })?
    }
    match ss.chars().next() {
        Some(c) => {
            if !c.is_alphabetic() {
                Err(
                    ElucidatorError::IllegalSpecification{
                        offender: s.to_string(),
                        reason: SpecificationFailure::IdentifierStartsNonAlphabetical,
                    }
                )?;
            }
        },
        None => {
            Err(ElucidatorError::Parsing{offender: s.to_string(), reason: ParsingFailure::UnexpectedEndOfExpression})?;
        },
    }
    Ok(ss)
}
