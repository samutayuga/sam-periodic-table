//! Pure value types and stateless calculations for the periodic table.

pub mod classification;
pub mod config;
pub mod element;
pub mod error;

pub use classification::{block, group, period, Block};
pub use config::{electron_configuration, ElectronConfiguration, Orbital, Subshell};
pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
