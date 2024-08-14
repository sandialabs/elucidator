use crate::member::{sizing::Sizing, dtype::Dtype};

pub struct MemberSpecification {
    identifier: String,
    sizing: Sizing,
    dtype: Dtype,
}

impl MemberSpecification {
    pub fn from_parts(identifier: &str, sizing: &Sizing, dtype: &Dtype) -> Self {
        MemberSpecification {
            identifier: identifier.to_string(),
            sizing: sizing.clone(),
            dtype: dtype.clone(),
        }
    }
}