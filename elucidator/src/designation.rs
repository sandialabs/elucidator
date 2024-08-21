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

fn subselect_text(text: &str, start: usize, end: usize) -> (&str, usize) {
    let end = if text.chars().count() <= end {
        text.chars().count() - 1
    } else {
        end
    };
    let (start_byte_pos, _) = text.char_indices().nth(start).unwrap();
    let (end_byte_pos, _) = text.char_indices().nth(end).unwrap();
    let last_comma_pos = text[..start_byte_pos]
        .rfind(',');
    let selection_end = match text[end_byte_pos..].find(',') {
        Some(pos) => pos + end_byte_pos,
        None => text.len(),
    };
    // Relative to last_spec_pos
    let selection_start = match last_comma_pos {
        Some(pos) => {
            let (char_after_last_spec_pos, _) = text[pos..]
                .char_indices()
                .nth(1)
                .unwrap_or((pos, ' '));
            char_after_last_spec_pos + pos
        },
        None => 0,
    };
    let subselection = &text[selection_start..selection_end];
    let offset = text[..selection_start].chars().count();
    (subselection, offset)
}

fn produce_caret_string(
    subselection: &str,
    start: usize,
    end: usize,
    offset: usize
) -> String {
    (offset..(offset + subselection.chars().count()))
        .map(|x| {
            if x >= start && x < end {
                '^'
            } else {
                ' '
            }
        })
        .collect()
}

fn produce_context(text: &str, start: usize, end: usize) -> String {
    let (selection, offset) = subselect_text(text, start, end);
    let caret_string = produce_caret_string(selection, start, end, offset);
    format!("{selection}\n{caret_string}")
}

fn convert_error(error: &InternalError, text: &str) -> ElucidatorError {
    match error {
        InternalError::Parsing{offender, reason} => {
            let column_start = offender.column_start;
            let column_end = offender.column_end;
            ElucidatorError::Specification {
                context: produce_context(text, column_start, column_end), 
                column_start,
                column_end,
                reason: format!("{reason}\n"),
            }
        },
        InternalError::IllegalSpecification{offender, reason} => {
            let column_start = offender.column_start;
            let column_end = offender.column_end;
            ElucidatorError::Specification {
                context: produce_context(text, column_start, column_end),
                column_start,
                column_end,
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
