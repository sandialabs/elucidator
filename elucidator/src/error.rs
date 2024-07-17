use std::{fmt, string::FromUtf8Error};

#[derive(Debug, PartialEq)]
pub enum ElucidatorError {
    /// Errors related to parsing strings, see [`ParsingFailure`] for reasons parsing might fail
    Parsing{offender: String, reason: ParsingFailure},
    /// Errors related to converting between incompatible types
    Conversion{from: String, to: String},
    /// Errors related to attempt to cast from high precision or range to low precision or range
    Narrowing{from: String, to: String},
    /// Errors related to interpreting a dtype from a given buffer
    BufferSizing{expected: usize, found: usize},
    /// Errors when parsing from UTF8
    FromUtf8{source: FromUtf8Error},
    /// Errors related to illegal specification
    IllegalSpecification{offender: String, reason: SpecificationFailure},
    /// Multiple errors have occurred
    MultipleFailure(Box<Vec<ElucidatorError>>),
}

impl ElucidatorError {
    pub fn new_conversion<T>(from: &str, to: &str) -> Result<T, ElucidatorError> {
        Err(ElucidatorError::Conversion{
            from: from.to_string(),
            to: to.to_string(),
        })
    }
    pub fn new_narrowing<T>(from: &str, to: &str) -> Result<T, ElucidatorError> {
        Err(ElucidatorError::Narrowing{
            from: from.to_string(),
            to: to.to_string(),
        })
    }
}

impl fmt::Display for ElucidatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::Parsing{offender, reason} => {
                format!("Failed to parse expression \"{offender}\": {reason}")
            },
            Self::Conversion{from, to} => {
                format!("Cannot convert {from} to {to}")
            },
            Self::Narrowing{from, to} => {
                format!("Conversion from {from} to {to} would cause narrowing")
            },
            Self::BufferSizing{expected, found} => {
                format!("Buffer expected size of {expected} bytes, found {found} instead")
            },
            Self::FromUtf8{source} => {
                format!("{source}")
            },
            Self::IllegalSpecification{offender, reason} => {
                format!("Illegal specification \"{offender}\": {reason}")
            },
            Self::MultipleFailure(errors) => {
                let error_text = errors.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("Multiple errors occurred:\n{error_text}")
            },
        };
        write!(f, "{m}")
    }
}

#[derive(Debug, PartialEq)]
pub enum ParsingFailure {
    NonAsciiEncoding,
    IllegalCharacters(Vec<char>),
    MissingIdSpecDelimiter,
    UnexpectedEndOfExpression,
    IllegalArraySizing,
}

impl fmt::Display for ParsingFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::NonAsciiEncoding => { "Non ASCII encoding".to_string() },
            Self::IllegalCharacters(clist) => {
                let offending_list = clist
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("Illegal characters encountered: {offending_list}")
            },
            Self::MissingIdSpecDelimiter => {
                "Missing delimeter : between identifier and type specification".to_string()
            },
            Self::UnexpectedEndOfExpression => {
                "Unexpected end of expression".to_string()
            },
            Self::IllegalArraySizing => {
                "The size of the array is not valid; valid sizes must be unsigned integers or empty".to_string()
            }
        };
        write!(f, "{m}")
    }
}

#[derive(Debug, PartialEq)]
pub enum SpecificationFailure {
    RepeatedIdentifier,
    IdentifierStartsNonAlphabetical,
    IllegalDataType,
}

impl fmt::Display for SpecificationFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::RepeatedIdentifier => {
                format!("Identifier is repeated, causing a naming collision")
            },
            Self::IdentifierStartsNonAlphabetical => {
                format!("Identifiers must start with alphabetical character")
            },
            Self::IllegalDataType => { "Illegal data type".to_string() },
        };
        write!(f, "{m}")
    }
}
