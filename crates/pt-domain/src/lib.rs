//! Pure value types and stateless calculations for the periodic table.

pub mod calc;
pub mod classification;
pub mod config;
pub mod element;
pub mod error;

pub use calc::{atomic_mass_from_isotopes, isotope_mass_matches, state_at};
pub use classification::{
    block, category, group, oxidation_states, period, Block, Category, OxidationStates,
};
pub use config::{electron_configuration, ElectronConfiguration, Orbital, Subshell};
pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
