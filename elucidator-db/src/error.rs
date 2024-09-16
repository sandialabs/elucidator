use elucidator::error::ElucidatorError;
use rusqlite::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum DatabaseError {
    /// Errors related to creating databases.
    RusqliteError{
        reason: rusqlite::Error,
    },
    ElucidatorError{
        reason: elucidator::error::ElucidatorError,
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::RusqliteError{reason} => {
                format!("SQL Error: {reason}")
            },
            Self::ElucidatorError{reason} => {
                format!("Elucidator Error: {reason}")
            },
        };
        write!(f, "{m}")
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        DatabaseError::RusqliteError { reason: error }
    }
}

impl From<elucidator::error::ElucidatorError> for DatabaseError {
    fn from(error: elucidator::error::ElucidatorError) -> Self {
        DatabaseError::ElucidatorError { reason: error }
    }
}