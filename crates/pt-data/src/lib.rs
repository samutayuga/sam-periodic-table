//! YAML loading and the in-memory element repository.

mod error;
mod parse;
mod raw;

pub use error::DataError;
pub use parse::parse_element_file;
