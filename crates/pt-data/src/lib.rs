//! YAML loading and the in-memory element repository.

mod error;
mod parse;
mod raw;
mod repository;

#[cfg(feature = "bundled")]
pub mod bundled;
#[cfg(feature = "bundled")]
pub use bundled::load_bundled;

pub use error::DataError;
pub use parse::parse_element_file;
pub use repository::{global, init_global, ElementRepository};
