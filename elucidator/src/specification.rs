use crate::{
    error::*,
    helper::validated_trimmed_or_err,
    member::MemberSpecification,
};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

//! Facilities for ingesting metadata specifications

/// Representation of a Metadata Specification. The designation is the identifier associated with
/// an ordered set of member specifications. This represents only the specification; an Interpreter
/// must be used to extract values from data buffers/blobs.
pub struct MetadataSpecification {
    designation: String,
    members: Vec<MemberSpecification>,
}

impl MetadataSpecification {
    fn from_str(designation: &str, specification_text: &str) -> Result<Self> {
        todo!();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     mod designation {
//         #[test]
//         fn not_ascii_err() {
//             todo!();
//         }
//         #[test]
//         fn non_alphabetical_start_err() {
//             todo!();
//         }
//         #[test]
//         fn contains_illegal_chars_err() {
//             todo!();
//         }
//         #[test]
//         fn is_ok() {
//             todo!();
//         }
//     }
// 
//     mod spec_test {
//         #[test]
//         fn not_ascii_err() {
//             todo!();
//         }
//         #[test]
//         fn contains_illegal_chars_err() {
//             todo!()
//         }
//         #[test]
//         fn member_repeated_err() {
//             todo!()
//         }
// }
