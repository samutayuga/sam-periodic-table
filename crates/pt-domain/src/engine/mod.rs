//! Reaction engine: bonding classification, covalent/metallic structure, and
//! stoichiometry for a two-reactant synthesis. Ports the iOS ChemCore `Engine`
//! layer onto the shared domain types.

pub mod bonding;
pub mod covalent;
pub mod math;
pub mod metallic;
pub mod polyatomic;
pub mod product_state;
pub mod stoichiometry;
pub mod valence;
