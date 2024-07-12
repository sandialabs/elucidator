//! Main elucidator library.
use std::collections::HashSet;

use crate::error::*;
pub use representable::Representable;

pub mod error;
pub mod representable;

/// Represent array sizing for a Member.
/// Generally not useful except when constructing Members for users, though it is used in this
/// library.
/// ```
/// use elucidator::Sizing;
///
/// // Fixed Sizing of 10
/// let fixed_size = Sizing::Fixed(10 as u64);
/// // Dynamic Sizing based on the identifier "len"
/// let dynamic_size = Sizing::Dynamic;
/// assert_eq!(fixed_size, Sizing::Fixed(10));
/// assert_eq!(dynamic_size, Sizing::Dynamic);
/// ```
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Sizing {
    Singleton,
    Fixed(u64),
    Dynamic,
}

/// Possible Data Types allowed in The Elucidation Metadata Standard, most composable as arrays.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Dtype {
    Byte,
    UnsignedInteger16,
    UnsignedInteger32,
    UnsignedInteger64,
    SignedInteger8,
    SignedInteger16,
    SignedInteger32,
    SignedInteger64,
    Float32,
    Float64,
    Str,
}

impl Dtype {
    pub fn from(s: &str) -> Result<Dtype, ElucidatorError> {
        let dt = match ascii_trimmed_or_err(s)? {
            "u8" => Self::Byte,
            "u16" => Self::UnsignedInteger16,
            "u32" => Self::UnsignedInteger32,
            "u64" => Self::UnsignedInteger64,
            "i8"  => Self::SignedInteger8,
            "i16" => Self::SignedInteger16,
            "i32" => Self::SignedInteger32,
            "i64" => Self::SignedInteger64,
            "f32" => Self::Float32,
            "f64" => Self::Float64,
            "string" => Self::Str,
            ss => {
                Err(
                    ElucidatorError::Parsing{
                        offender: ss.to_string(),
                        reason: ParsingFailure::IllegalDataType,
                    }
                )?
            },
        };
        Ok(dt)
    }
    pub fn from_buffer(&self, buffer: &[u8]) -> Result<Box<dyn Representable>, ElucidatorError> {
        match self {
            Self::Byte => {
                let buffer_len = buff_size_or_err::<u8>(buffer)?;
                Ok(Box::new(u8::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::UnsignedInteger16 => {
                let buffer_len = buff_size_or_err::<u16>(buffer)?;
                Ok(Box::new(u16::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::UnsignedInteger32 => {
                let buffer_len = buff_size_or_err::<u32>(buffer)?;
                Ok(Box::new(u32::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::UnsignedInteger64 => {
                let buffer_len = buff_size_or_err::<u64>(buffer)?;
                Ok(Box::new(u64::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger8 => {
                let buffer_len = buff_size_or_err::<i8>(buffer)?;
                Ok(Box::new(i8::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger16 => {
                let buffer_len = buff_size_or_err::<i16>(buffer)?;
                Ok(Box::new(i16::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger32 => {
                let buffer_len = buff_size_or_err::<i32>(buffer)?;
                Ok(Box::new(i32::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger64 => {
                let buffer_len = buff_size_or_err::<i64>(buffer)?;
                Ok(Box::new(i64::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::Float32 => {
                let buffer_len = buff_size_or_err::<f32>(buffer)?;
                Ok(Box::new(f32::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::Float64 => {
                let buffer_len = buff_size_or_err::<f64>(buffer)?;
                Ok(Box::new(f64::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::Str => {
                let buffer_len = buffer.len();
                if buffer_len < 8 {
                    Err(ElucidatorError::BufferSizing { expected: 8, found: buffer_len })?
                }
                let string_length_buffer = buffer.iter().take(8).copied().collect::<Vec<u8>>();
                let dt = Dtype::UnsignedInteger64;
                let string_length = dt.from_buffer(&string_length_buffer).unwrap().as_u64().unwrap() as usize;

                let expected_buffer_len: usize = string_length + 8;

                if buffer_len != expected_buffer_len {
                    Err(ElucidatorError::BufferSizing { expected: expected_buffer_len, found: buffer_len })?
                }
                let s = String::from_utf8(
                    buffer[8..].to_vec()
                );
                match s {
                    Ok(o) => { Ok(Box::new(o)) }
                    Err(e) => {
                        Err(ElucidatorError::FromUtf8 { source: e })
                    }
                }

            },
        }
    }
}

fn buff_size_or_err<T>(buffer: &[u8]) -> Result<usize, ElucidatorError> {
    let expected_buff_size = std::mem::size_of::<T>();
    if buffer.len() != expected_buff_size {
        Err(ElucidatorError::BufferSizing { expected: expected_buff_size, found: buffer.len() })?
    }
    Ok(expected_buff_size)
}

fn ascii_or_err(s: &str) -> Result<(), ElucidatorError> {
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

fn ascii_trimmed_or_err(s: &str) -> Result<&str, ElucidatorError> {
    ascii_or_err(s)?;
    Ok(s.trim())
}

fn is_valid_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '[' || c == ']' ||  c.is_whitespace()
}

fn validated_trimmed_or_err(s: &str) -> Result<&str, ElucidatorError> {
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

fn validate_identifier(s: &str) -> Result<&str, ElucidatorError> {
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
                    ElucidatorError::Parsing{
                        offender: s.to_string(),
                        reason: ParsingFailure::IdentifierStartsNonAlphabetical,
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

#[derive(Debug, PartialEq)]
pub struct TypeSpecification {
    dtype: Dtype,
    sizing: Sizing,
}

impl TypeSpecification {
    pub fn from(s: &str) -> Result<TypeSpecification, ElucidatorError> {
        let type_spec = validated_trimmed_or_err(s)?;
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

#[derive(Debug, PartialEq)]
pub struct MemberSpecification {
    identifier: String,
    typespec: TypeSpecification,
}

impl MemberSpecification {
    pub fn from(s: &str) -> Result<MemberSpecification, ElucidatorError> {
        let ss = ascii_trimmed_or_err(s)?;
        if let Some((ident, typespec)) = ss.split_once(':') {
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
    use std::collections::HashMap;

    fn get_dtype_map() -> HashMap<&'static str, Result<Dtype, ElucidatorError>> {
        HashMap::from([
            ("u8",      Ok(Dtype::Byte)),
            ("u16",     Ok(Dtype::UnsignedInteger16)),
            ("u32",     Ok(Dtype::UnsignedInteger32)),
            ("u64",     Ok(Dtype::UnsignedInteger64)),
            ("i8",      Ok(Dtype::SignedInteger8)),
            ("i16",     Ok(Dtype::SignedInteger16)),
            ("i32",     Ok(Dtype::SignedInteger32)),
            ("i64",     Ok(Dtype::SignedInteger64)),
            ("f32",     Ok(Dtype::Float32)),
            ("f64",     Ok(Dtype::Float64)),
            ("string",  Ok(Dtype::Str))
        ])
    }


    #[test]
    fn get_u8_from_buffer() {
        let expected_value: u8 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::Byte;
        let value = dt.from_buffer(&buffer).unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value.as_u64().unwrap(), expected_value as u64);
    }

    #[test]
    fn get_u16_from_buffer() {
        let expected_value: u16 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::UnsignedInteger16;
        let value = dt.from_buffer(&buffer).unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value.as_u64().unwrap(), expected_value as u64);
    }

    #[test]
    fn get_u32_from_buffer() {
        let expected_value: u32 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::UnsignedInteger32;
        let value = dt.from_buffer(&buffer).unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value.as_u64().unwrap(), expected_value as u64);
    }

    #[test]
    fn get_u64_from_buffer() {
        let expected_value: u64 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::UnsignedInteger64;
        let value = dt.from_buffer(&buffer).unwrap().as_u64().unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value, expected_value);
    }

    // Signed integers
    #[test]
    fn get_i8_from_buffer() {
        let expected_value: i8 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::SignedInteger8;
        let value = dt.from_buffer(&buffer).unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value.as_i64().unwrap(), expected_value as i64);
    }

    #[test]
    fn get_i16_from_buffer() {
        let expected_value: i16 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::SignedInteger16;
        let value = dt.from_buffer(&buffer).unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value.as_i64().unwrap(), expected_value as i64);
    }

    #[test]
    fn get_i32_from_buffer() {
        let expected_value: i32 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::SignedInteger32;
        let value = dt.from_buffer(&buffer).unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value.as_i64().unwrap(), expected_value as i64);
    }

    #[test]
    fn get_i64_from_buffer() {
        let expected_value: i64 = 7;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::SignedInteger64;
        let value = dt.from_buffer(&buffer).unwrap().as_i64().unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value, expected_value);
    }

    // Floating points
    #[test]
    fn get_f32_from_buffer() {
        let expected_value: f32 = 7.0;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::Float32;
        let value = dt.from_buffer(&buffer).unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value.as_f64().unwrap(), expected_value as f64);
    }

    #[test]
    fn get_f64_from_buffer() {
        let expected_value: f64 = 7.0;
        let buffer = expected_value.as_buffer();
        let dt = Dtype::Float64;
        let value = dt.from_buffer(&buffer).unwrap().as_f64().unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value, expected_value);
    }

    // Strings

    #[test]
    fn get_string_from_buffer() {
        let expected_value: String = "Hello world!".to_string();
        let buffer = expected_value.as_buffer();
        let dt = Dtype::Str;
        let value = dt.from_buffer(&buffer).unwrap().as_string().unwrap();
        let resulting_buffer = value.as_buffer();
        assert_eq!(buffer, resulting_buffer);
        assert_eq!(value, expected_value);
    }

    #[test]
    fn get_string_from_buffer_fails() {
        // https://doc.rust-lang.org/std/string/struct.FromUtf8Error.html
        let buffer: Vec<u8> = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 159];
        let utf8_error = String::from_utf8(vec![0, 159]).err().unwrap();
        let dt = Dtype::Str;
        let value = dt.from_buffer(&buffer);
        assert_eq!(value.err().unwrap(), ElucidatorError::FromUtf8 { source: utf8_error });
    }

    #[test]
    fn dtype_non_ascii_str() {
        let crab_emoji = String::from('\u{1F980}');
        assert_eq!(
            Dtype::from(&crab_emoji),
            Err(
                ElucidatorError::Parsing {
                    offender: crab_emoji,
                    reason: ParsingFailure::NonAsciiEncoding,
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
                ElucidatorError::Parsing {
                    offender: invalid_dtype.to_string(),
                    reason: ParsingFailure::IllegalDataType,
                }
            )
        );

    }
    #[test]
    fn dtype_all_parsed_correct() {
        let result_map = get_dtype_map()
            .keys()
            .map(|x| {(*x, Dtype::from(x))})
            .collect::<HashMap<&str, Result<Dtype, ElucidatorError>>>();
        assert_eq!(result_map, get_dtype_map());
    }
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
