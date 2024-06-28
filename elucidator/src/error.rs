use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ElucidatorError {
    /// Errors related to parsing strings, see [`Invalidity`] for reasons parsing might fail
    ParsingError{offender: String, reason: Invalidity},
    /// Errors related to converting between incompatible types
    ConversionError{from: String, to: String},
    /// Errors related to attempt to cast from high precision or range to low precision or range
    NarrowingError{from: String, to: String},
}

impl ElucidatorError {
    pub fn new_conversion<T>(from: &str, to: &str) -> Result<T, ElucidatorError> {
        Err(ElucidatorError::ConversionError{
            from: from.to_string(),
            to: to.to_string(),
        })
    }
    pub fn new_narrowing<T>(from: &str, to: &str) -> Result<T, ElucidatorError> {
        Err(ElucidatorError::NarrowingError{
            from: from.to_string(),
            to: to.to_string(),
        })
    }
}

impl fmt::Display for ElucidatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::ParsingError{offender, reason} => {
                format!("Failed to parse expression \"{offender}\": {reason}")
            },
            Self::ConversionError{from, to} => {
                format!("Cannot convert {from} to {to}")
            },
            Self::NarrowingError{from, to} => {
                format!("Conversion from {from} to {to} would cause narrowing")
            },
        };
        write!(f, "{m}")
    }
}

#[derive(Debug, PartialEq)]
pub enum Invalidity {
    NonAsciiEncoding,
    IdentifierStartsNonAlphabetical,
    IllegalCharacters(Vec<char>),
    IllegalDataType,
    MissingIdSpecDelimiter,
    UnexpectedEndOfExpression,
}

impl fmt::Display for Invalidity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::NonAsciiEncoding => { "Non ASCII encoding".to_string() },
            Self::IdentifierStartsNonAlphabetical => {
                "Identifiers must begin with an alphabetical character".to_string()
            },
            Self::IllegalCharacters(clist) => {
                let offending_list = clist
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("Illegal characters encountered: {offending_list}")
            },
            Self::IllegalDataType => { "Illegal data type".to_string() },
            Self::MissingIdSpecDelimiter => {
                "Missing delimeter : between identifier and type specification".to_string()
            },
            Self::UnexpectedEndOfExpression => {
                "Unexpected end of expression".to_string()
            },
        };
        write!(f, "{m}")
    }
}
