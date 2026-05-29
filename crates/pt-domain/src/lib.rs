//! Pure value types and stateless calculations for the periodic table.

pub mod element;
pub mod error;

pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
