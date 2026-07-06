//! Pure value types and stateless calculations for the periodic table.

pub mod calc;
pub mod classification;
pub mod config;
pub mod element;
pub mod engine;
pub mod error;

pub use calc::{atomic_mass_from_isotopes, isotope_mass_matches, state_at};
pub use classification::{
    block, category, element_class, group, oxidation_states, period, Block, Category, ElementClass,
    OxidationStates,
};
pub use config::{electron_configuration, ElectronConfiguration, Orbital, Subshell};
pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;

pub use engine::bonding::{bonding_type, determine_bonding, reaction_glyph, BondingType};
pub use engine::covalent::{
    calc_stoich, covalent_stoich, is_orbital_mismatch_double_bond, iupac_first, CovalentStoich,
};
pub use engine::math::{gcd, lcm};
pub use engine::metallic::{metallic_electron_count, metallic_electron_count_capped};
pub use engine::polyatomic::{PolyatomicIon, POLYATOMIC_IONS};
pub use engine::product_state::{predict_product_state, ProductState};
pub use engine::stoichiometry::{
    balance_equation, is_naturally_diatomic, molecularity, solve_stoichiometry, AmountResult,
    BalancedEquation, LimitingSide, QuantityUnit, ReactantEntry, ReactantSpec, StoichResult,
    NATURALLY_DIATOMIC,
};
pub use engine::valence::{
    group_to_valence_fallback, is_transition_metal, parse_valence_electrons,
};
