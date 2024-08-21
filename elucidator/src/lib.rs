//! Main elucidator library.
use crate::error::*;
pub use representable::Representable;

mod parsing;
mod validating;
mod test_utils;
mod token;
pub mod error;
pub mod representable;
pub mod member;
pub mod designation;
