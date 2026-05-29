//! Pure value types and stateless calculations for the periodic table.

pub mod config;
pub mod element;
pub mod error;

pub use config::{electron_configuration, ElectronConfiguration, Orbital, Subshell};
pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
