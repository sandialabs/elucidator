use crate::member::{sizing::Sizing, dtype::Dtype};

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