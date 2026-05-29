//! YAML loading and the in-memory element repository.

mod error;
mod parse;
mod raw;
mod repository;

pub use error::DataError;
pub use parse::parse_element_file;
pub use repository::{global, init_global, ElementRepository};
