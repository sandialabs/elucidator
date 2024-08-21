use crate::{
    error::*,
    member::MemberSpecification,
    parsing,
    validating,
};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

/// Representation of a Designation's specification.
/// Use to parse a specification for an individual designation.
/// To construct, it is typical to use the `from_str` method.
/// ```
/// use elucidator::designation::DesignationSpecification;
/// # use elucidator::member::{Dtype, MemberSpecification, Sizing};
///
/// let spec = DesignationSpecification::from_str("foo: u32");
///
/// # assert!(spec.is_ok())
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct DesignationSpecification {
    members: Vec<MemberSpecification>,
}

fn convert_error(error: &InternalError, text: &str) -> ElucidatorError {
    match error {
        InternalError::Parsing{offender, reason} => {
            ElucidatorError::Specification {
                context: offender.data.clone(),
                column_start: offender.column_start,
                column_end: offender.column_end,
                reason: format!("{reason}\n"),
            }
        },
        InternalError::IllegalSpecification{offender, reason} => {
            ElucidatorError::Specification {
                context: offender.data.clone(),
                column_start: offender.column_start,
                column_end: offender.column_end,
                reason: format!("{reason}"),
            }
        },
        InternalError::MultipleFailures(errs) => {
            let errors: Vec<ElucidatorError> = errs.iter()
                .map(|e| convert_error(e, text))
                .collect();
            ElucidatorError::merge(&errors)
        }
    }
}

impl DesignationSpecification {
    pub fn from_str(text: &str) -> Result<Self> {
        let parsed = parsing::get_metadataspec(text);
        let validated = validating::validate_metadataspec(&parsed);
        match validated {
            Ok(members) => Ok(DesignationSpecification{ members }),
            Err(e) => Err(convert_error(&e, text)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::member::{Sizing, Dtype};
    use pretty_assertions::assert_eq;

    #[test]
    fn multiple_members_ok() {
        let text = "foo: u32, bar: f32[10], baz: string";
        let dspec = DesignationSpecification::from_str(text);
        assert_eq!(
            dspec,
            Ok(DesignationSpecification{members: vec![
                MemberSpecification::from_parts(
                    "foo", &Sizing::Singleton, &Dtype::UnsignedInteger32,
                ),
                MemberSpecification::from_parts(
                    "bar", &Sizing::Fixed(10), &Dtype::Float32,
                ),
                MemberSpecification::from_parts(
                    "baz", &Sizing::Singleton, &Dtype::Str,
                ),
            ]})
        );
    }
}
