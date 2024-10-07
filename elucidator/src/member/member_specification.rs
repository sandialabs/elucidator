use crate::member::{dtype::Dtype, sizing::Sizing};

#[derive(Debug, Clone, PartialEq)]
pub struct MemberSpecification {
    pub(crate) identifier: String,
    pub(crate) sizing: Sizing,
    pub(crate) dtype: Dtype,
}

impl MemberSpecification {
    pub fn from_parts(identifier: &str, sizing: &Sizing, dtype: &Dtype) -> Self {
        if *dtype == Dtype::Str && *sizing != Sizing::Singleton {
            panic!("Dtype is string, but sizing is non-singleton for passed values {identifier:#?}, {sizing:#?}, {dtype:#?}. TODO: make this panic an error.");
        }
        MemberSpecification {
            identifier: identifier.to_string(),
            sizing: sizing.clone(),
            dtype: dtype.clone(),
        }
    }
}

impl std::fmt::Display for MemberSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let sizing_string = match self.sizing {
            Sizing::Singleton => {
                format!("")
            }
            Sizing::Dynamic => {
                format!("[]")
            }
            Sizing::Fixed(n) => {
                format!("[{n}]")
            }
        };
        let dtype_string = match self.dtype {
            Dtype::Byte => {
                format!("u8")
            }
            Dtype::UnsignedInteger16 => {
                format!("u16")
            }
            Dtype::UnsignedInteger32 => {
                format!("u32")
            }
            Dtype::UnsignedInteger64 => {
                format!("u64")
            }
            Dtype::SignedInteger8 => {
                format!("i8")
            }
            Dtype::SignedInteger16 => {
                format!("i16")
            }
            Dtype::SignedInteger32 => {
                format!("i32")
            }
            Dtype::SignedInteger64 => {
                format!("i64")
            }
            Dtype::Float32 => {
                format!("f32")
            }
            Dtype::Float64 => {
                format!("f64")
            }
            Dtype::Str => {
                format!("string")
            }
        };
        let m = format!("{}: {dtype_string}{sizing_string}", self.identifier);
        write!(f, "{m}")
    }
}
