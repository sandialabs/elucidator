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
            Sizing::Singleton => String::new(),
            Sizing::Dynamic => "[]".to_string(),
            Sizing::Fixed(n) => {
                format!("[{n}]")
            }
        };
        let dtype_string = match self.dtype {
            Dtype::Byte => "u8".to_string(),
            Dtype::UnsignedInteger16 => "u16".to_string(),
            Dtype::UnsignedInteger32 => "u32".to_string(),
            Dtype::UnsignedInteger64 => "u64".to_string(),
            Dtype::SignedInteger8 => "i8".to_string(),
            Dtype::SignedInteger16 => "i16".to_string(),
            Dtype::SignedInteger32 => "i32".to_string(),
            Dtype::SignedInteger64 => "i64".to_string(),
            Dtype::Float32 => "f32".to_string(),
            Dtype::Float64 => "f64".to_string(),
            Dtype::Str => "string".to_string(),
        };
        let m = format!("{}: {dtype_string}{sizing_string}", self.identifier);
        write!(f, "{m}")
    }
}
