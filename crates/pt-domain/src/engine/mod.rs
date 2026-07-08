//! Reaction engine: bonding classification, covalent/metallic structure, and
//! stoichiometry for a two-reactant synthesis. Ports the iOS ChemCore `Engine`
//! layer onto the shared domain types.

pub mod activity_series;
pub mod balancer;
pub mod bonding;
pub(crate) mod composition;
pub mod covalent;
pub mod formula_text;
pub mod fraction;
pub mod math;
pub mod metallic;
pub mod oxidation_state;
pub mod polyatomic;
pub mod product_prediction;
pub mod product_state;
pub mod reactant;
pub mod reaction_class;
pub mod reaction_solver;
pub mod redox;
pub mod species;
pub mod stoichiometry;
pub mod valence;
