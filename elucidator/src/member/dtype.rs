use crate::error::*;
use crate::helper;
use crate::Representable;

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
        let dt = match helper::ascii_trimmed_or_err(s)? {
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
                let buffer_len = helper::buff_size_or_err::<u8>(buffer)?;
                Ok(Box::new(u8::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::UnsignedInteger16 => {
                let buffer_len = helper::buff_size_or_err::<u16>(buffer)?;
                Ok(Box::new(u16::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::UnsignedInteger32 => {
                let buffer_len = helper::buff_size_or_err::<u32>(buffer)?;
                Ok(Box::new(u32::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::UnsignedInteger64 => {
                let buffer_len = helper::buff_size_or_err::<u64>(buffer)?;
                Ok(Box::new(u64::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger8 => {
                let buffer_len = helper::buff_size_or_err::<i8>(buffer)?;
                Ok(Box::new(i8::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger16 => {
                let buffer_len = helper::buff_size_or_err::<i16>(buffer)?;
                Ok(Box::new(i16::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger32 => {
                let buffer_len = helper::buff_size_or_err::<i32>(buffer)?;
                Ok(Box::new(i32::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::SignedInteger64 => {
                let buffer_len = helper::buff_size_or_err::<i64>(buffer)?;
                Ok(Box::new(i64::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::Float32 => {
                let buffer_len = helper::buff_size_or_err::<f32>(buffer)?;
                Ok(Box::new(f32::from_le_bytes(
                    buffer.iter().take(buffer_len).copied().collect::<Vec<u8>>().try_into().unwrap()
                )))
            },
            Self::Float64 => {
                let buffer_len = helper::buff_size_or_err::<f64>(buffer)?;
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
}
