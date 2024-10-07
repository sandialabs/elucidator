use crate::{error::ElucidatorError, representable::Representable};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

/// Store data values that have been interpreted
#[derive(Clone, Debug, PartialEq)]
pub enum DataValue {
    Byte(u8),
    UnsignedInteger16(u16),
    UnsignedInteger32(u32),
    UnsignedInteger64(u64),
    SignedInteger8(i8),
    SignedInteger16(i16),
    SignedInteger32(i32),
    SignedInteger64(i64),
    Float32(f32),
    Float64(f64),
    Str(String),
    ByteArray(Vec<u8>),
    UnsignedInteger16Array(Vec<u16>),
    UnsignedInteger32Array(Vec<u32>),
    UnsignedInteger64Array(Vec<u64>),
    SignedInteger8Array(Vec<i8>),
    SignedInteger16Array(Vec<i16>),
    SignedInteger32Array(Vec<i32>),
    SignedInteger64Array(Vec<i64>),
    Float32Array(Vec<f32>),
    Float64Array(Vec<f64>),
}

impl DataValue {
    pub fn as_buffer(&self) -> Vec<u8> {
        match self {
            Self::Byte(v) => v.to_le_bytes().to_vec(),
            Self::UnsignedInteger16(v) => v.to_le_bytes().to_vec(),
            Self::UnsignedInteger32(v) => v.to_le_bytes().to_vec(),
            Self::UnsignedInteger64(v) => v.to_le_bytes().to_vec(),
            Self::SignedInteger8(v) => v.to_le_bytes().to_vec(),
            Self::SignedInteger16(v) => v.to_le_bytes().to_vec(),
            Self::SignedInteger32(v) => v.to_le_bytes().to_vec(),
            Self::SignedInteger64(v) => v.to_le_bytes().to_vec(),
            Self::Float32(v) => v.to_le_bytes().to_vec(),
            Self::Float64(v) => v.to_le_bytes().to_vec(),
            Self::Str(s) => s.as_buffer(),
            Self::ByteArray(v) => v.as_buffer(),
            Self::UnsignedInteger16Array(v) => v.as_buffer(),
            Self::UnsignedInteger32Array(v) => v.as_buffer(),
            Self::UnsignedInteger64Array(v) => v.as_buffer(),
            Self::SignedInteger8Array(v) => v.as_buffer(),
            Self::SignedInteger16Array(v) => v.as_buffer(),
            Self::SignedInteger32Array(v) => v.as_buffer(),
            Self::SignedInteger64Array(v) => v.as_buffer(),
            Self::Float32Array(v) => v.as_buffer(),
            Self::Float64Array(v) => v.as_buffer(),
        }
    }
}

pub(crate) trait LeBufferRead: Sized {
    fn get_one_le(buf: &[u8]) -> Result<Self>;
    fn get_n_le(buf: &[u8], n: usize) -> Result<Vec<Self>>;
    fn bytes_needed(n: usize) -> usize;
}

macro_rules! impl_le_bufread {
    ($($tt:ty), *)  => {
        $(
            impl LeBufferRead for $tt {
                fn get_one_le(buf: &[u8]) -> Result<Self> {
                    let expected_bytes = std::mem::size_of::<$tt>();
                    if buf.len() < std::mem::size_of::<$tt>() {
                        Err(ElucidatorError::BufferSizing{
                            expected: std::mem::size_of::<$tt>(),
                            found: buf.len()
                        })?
                    }
                    Ok(
                        <$tt>::from_le_bytes(buf[..expected_bytes].try_into().unwrap())
                    )
                }
                fn get_n_le(buf: &[u8], n: usize) -> Result<Vec<Self>> {
                    let expected_bytes = std::mem::size_of::<$tt>() * n;
                    if buf.len() < expected_bytes {
                        Err(ElucidatorError::BufferSizing{
                            expected: expected_bytes,
                            found: buf.len(),
                        })?
                    }
                    if n == 0 && buf.len() == 0 {
                        Ok(Vec::new())
                    } else {
                        Ok(buf[..expected_bytes]
                            .chunks_exact(std::mem::size_of::<$tt>())
                            .map(|x|
                                <$tt>::from_le_bytes(x.try_into().unwrap())
                            )
                            .collect()
                        )
                    }
                }
                fn bytes_needed(n: usize) -> usize {
                    std::mem::size_of::<$tt>() * n
                }
            }
        )*
    };
}

impl_le_bufread! {u8, u16, u32, u64, i8, i16, i32, i64, f32, f64}

impl LeBufferRead for String {
    fn get_one_le(buf: &[u8]) -> Result<Self> {
        if buf.len() != 8 {
            Err(ElucidatorError::BufferSizing {
                expected: 8,
                found: buf.len(),
            })?
        }
        let n_bytes = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        if n_bytes == 0 {
            Ok("".to_string())
        } else {
            let databuf = &buf[8..];
            match String::from_utf8(databuf.to_vec()) {
                Ok(o) => Ok(o),
                Err(e) => Err(ElucidatorError::FromUtf8 { source: e }),
            }
        }
    }
    fn get_n_le(_buf: &[u8], _n: usize) -> Result<Vec<Self>> {
        unreachable!("We don't do buffers of multiple strings");
    }
    fn bytes_needed(_n: usize) -> usize {
        unimplemented!();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions;
    use rand::random;

    macro_rules! singleton_round_trip {
        ($($tt:ty), *) => {
            $(
                let item: $tt = random();
                let buf = item.to_le_bytes().to_vec();
                let extracted = <$tt>::get_one_le(&buf);
                pretty_assertions::assert_eq!(
                    extracted,
                    Ok(item),
                    "Buffer is {buf:#?}, type is {}", stringify!($tt),
                );
            )*
        }
    }

    macro_rules! vec_round_trip {
        ($($tt:ty), *) => {
            $(
                let size: u8 = random();
                let vec: Vec<$tt> = (0..size)
                    .map(|_| random::<$tt>())
                    .collect();
                let buf: Vec<u8> = vec.iter()
                    .flat_map(|x| x.to_le_bytes())
                    .collect();
                let extracted = <$tt>::get_n_le(&buf, size as usize);
                assert_eq!(
                    extracted,
                    Ok(vec),
                    "Type is {}", stringify!($tt),
                );
            )*
        }
    }

    #[test]
    fn test_singleton_round_trips() {
        singleton_round_trip!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
    }

    #[test]
    fn test_vec_round_trips() {
        vec_round_trip!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
    }
}
