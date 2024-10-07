use crate::token::TokenClone;
use std::{collections::HashSet, fmt, string::FromUtf8Error};

#[derive(Debug, PartialEq, Clone)]
pub enum ElucidatorError {
    /// Errors related to converting between incompatible types
    Conversion { from: String, to: String },
    /// Errors related to attempt to cast from high precision or range to low precision or range
    Narrowing { from: String, to: String },
    /// Errors related to interpreting a dtype from a given buffer
    BufferSizing { expected: usize, found: usize },
    /// Errors when parsing from UTF8
    FromUtf8 { source: FromUtf8Error },
    /// Errors related to illegal or malformed specification
    Specification {
        context: String,
        column_start: usize,
        column_end: usize,
        reason: String,
    },
    /// Multiple, simultaneous failures
    MultipleErrors(Box<Vec<ElucidatorError>>),
}

impl ElucidatorError {
    pub fn new_conversion<T>(from: &str, to: &str) -> Result<T, ElucidatorError> {
        Err(ElucidatorError::Conversion {
            from: from.to_string(),
            to: to.to_string(),
        })
    }
    pub fn new_narrowing<T>(from: &str, to: &str) -> Result<T, ElucidatorError> {
        Err(ElucidatorError::Narrowing {
            from: from.to_string(),
            to: to.to_string(),
        })
    }
    fn expand(&self) -> Vec<ElucidatorError> {
        match &self {
            Self::MultipleErrors(errs) => errs.iter().flat_map(|e| e.expand()).collect(),
            _ => {
                vec![self.clone()]
            }
        }
    }
    pub fn merge_with(left: &ElucidatorError, right: &ElucidatorError) -> ElucidatorError {
        Self::merge(&[left.clone(), right.clone()])
    }
    pub fn merge(errs: &[ElucidatorError]) -> ElucidatorError {
        let errors: Vec<ElucidatorError> = errs.iter().flat_map(ElucidatorError::expand).collect();
        if errors.len() == 1 {
            errors[0].clone()
        } else {
            ElucidatorError::MultipleErrors(Box::new(errors))
        }
    }
}

impl fmt::Display for ElucidatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::Conversion { from, to } => {
                format!("Cannot convert {from} to {to}")
            }
            Self::Narrowing { from, to } => {
                format!("Conversion from {from} to {to} would cause narrowing")
            }
            Self::BufferSizing { expected, found } => {
                format!("Buffer expected size of {expected} bytes, found {found} instead")
            }
            Self::FromUtf8 { source } => {
                format!("{source}")
            }
            Self::Specification {
                context,
                column_start,
                column_end,
                reason,
            } => {
                format!("Error {reason} between positions {column_start} and {column_end}:\n{context}\n")
            }
            Self::MultipleErrors(errs) => errs
                .iter()
                .map(|x| format!("{x}"))
                .collect::<Vec<String>>()
                .join(""),
        };
        write!(f, "{m}")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum InternalError {
    /// Errors related to parsing strings, see [`ParsingFailure`] for reasons parsing might fail
    Parsing {
        offender: TokenClone,
        reason: ParsingFailure,
    },
    /// Errors related to illegal specification
    IllegalSpecification {
        offender: TokenClone,
        reason: SpecificationFailure,
    },
    /// Multiple errors have occurred
    MultipleFailures(Vec<InternalError>),
}

impl InternalError {
    fn expand(&self) -> Vec<InternalError> {
        match &self {
            Self::MultipleFailures(errs) => errs.iter().flat_map(|e| e.expand()).collect(),
            _ => {
                vec![self.clone()]
            }
        }
    }
    pub fn merge(errs: &[InternalError]) -> InternalError {
        let mut entries: HashSet<String> = HashSet::new();
        let mut errors: Vec<InternalError> = errs.iter().flat_map(InternalError::expand).collect();
        errors.retain(|x| entries.insert(format!("{x}")));
        if errors.len() == 1 {
            errors[0].clone()
        } else {
            InternalError::MultipleFailures(errors)
        }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::Parsing { offender, reason } => {
                format!("Failed to parse due to {reason}: {offender}")
            }
            Self::IllegalSpecification { offender, reason } => {
                format!("Illegal specification \"{offender}\": {reason}")
            }
            Self::MultipleFailures(errors) => {
                let error_text = errors
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("Multiple errors occurred:\n{error_text}")
            }
        };
        write!(f, "{m}")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ParsingFailure {
    MissingIdSpecDelimiter,
    UnexpectedEndOfExpression,
}

impl fmt::Display for ParsingFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::MissingIdSpecDelimiter => {
                "Missing delimeter : between identifier and type specification".to_string()
            }
            Self::UnexpectedEndOfExpression => "Unexpected end of expression".to_string(),
        };
        write!(f, "{m}")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum SpecificationFailure {
    RepeatedIdentifier { first: TokenClone },
    IdentifierStartsNonAlphabetical,
    IllegalDataType,
    ZeroLengthIdentifier,
    IllegalArraySizing,
    IllegalCharacters(Vec<char>),
}

impl fmt::Display for SpecificationFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::RepeatedIdentifier { first } => {
                format!(
                    "Identifier \"{}\" is repeated after first occurrence at positions {} to {}, causing a naming collision",
                    first.data,
                    first.column_start,
                    first.column_end
                )
            }
            Self::IdentifierStartsNonAlphabetical => {
                "Identifiers must start with alphabetical character".to_string()
            }
            Self::IllegalDataType => "Illegal data type".to_string(),
            Self::ZeroLengthIdentifier => "Identifiers must have non-zero length".to_string(),
            Self::IllegalCharacters(clist) => {
                let offending_list = clist
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("Illegal characters encountered: {offending_list}")
            }
            Self::IllegalArraySizing => {
                "The size of the array is not valid; valid sizes must be unsigned integers or empty"
                    .to_string()
            }
        };
        write!(f, "{m}")
    }
}
