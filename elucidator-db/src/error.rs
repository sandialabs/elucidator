use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum DatabaseError {
    /// Errors related to creating databases.
    RusqliteError {
        reason: String,
    },
    ElucidatorError {
        reason: elucidator::error::ElucidatorError,
    },
    IOError {
        reason: String,
    },
    VersionError {
        reason: String,
    },
    ConfigError {
        reason: String,
    },
    LockError {
        reason: String,
    },
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = match self {
            Self::RusqliteError { reason } => {
                format!("SQL Error: {reason}")
            }
            Self::ElucidatorError { reason } => {
                format!("Elucidator Error: {reason}")
            }
            Self::IOError { reason } => {
                format!("IO Error: {reason}")
            }
            Self::VersionError { reason } => {
                format!("Version Error: {reason}")
            }
            Self::ConfigError { reason } => {
                format!("Config Error: {reason}")
            }
            Self::LockError { reason } => {
                format!("Lock Error: {reason}")
            }
        };
        write!(f, "{m}")
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        DatabaseError::RusqliteError {
            reason: format!("{error}"),
        }
    }
}

impl From<rusqlite::types::FromSqlError> for DatabaseError {
    fn from(error: rusqlite::types::FromSqlError) -> Self {
        DatabaseError::RusqliteError {
            reason: format!("{error}"),
        }
    }
}

impl From<elucidator::error::ElucidatorError> for DatabaseError {
    fn from(error: elucidator::error::ElucidatorError) -> Self {
        DatabaseError::ElucidatorError { reason: error }
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(error: std::io::Error) -> Self {
        DatabaseError::IOError {
            reason: format!("{error}"),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for DatabaseError {
    fn from(error: std::sync::PoisonError<T>) -> Self {
        DatabaseError::LockError {
            reason: format!("{error}"),
        }
    }
}
