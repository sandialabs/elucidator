//! Main elucidator library.
use crate::error::*;
pub use representable::Representable;

pub mod designation;
pub mod error;
pub mod member;
mod parsing;
pub mod representable;
mod test_utils;
mod token;
mod util;
mod validating;
pub mod value;
