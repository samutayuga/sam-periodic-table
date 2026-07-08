//! Shared composition arithmetic for building compounds from [`Species`].
//!
//! Extracted from duplicated helpers in `reactant.rs` and
//! `product_prediction.rs`: scaling/merging elemental composition maps, and
//! the crossover → scale → merge → formula assembly used to build an ionic
//! compound from a cation/anion pair.

use std::collections::BTreeMap;

use crate::engine::formula_text::{binary_formula, crossover_subscripts};
use crate::engine::species::Species;

pub(crate) fn scaled(comp: &BTreeMap<String, i64>, n: i64) -> BTreeMap<String, i64> {
    comp.iter()
        .map(|(symbol, count)| (symbol.clone(), count * n))
        .collect()
}

pub(crate) fn merge(a: BTreeMap<String, i64>, b: BTreeMap<String, i64>) -> BTreeMap<String, i64> {
    let mut out = a;
    for (symbol, count) in b {
        *out.entry(symbol).or_insert(0) += count;
    }
    out
}

/// Assemble an ionic compound from a cation/anion pair: crossover their
/// charges into subscripts, scale + merge their compositions, and build the
/// display formula and molar mass.
pub(crate) fn ionic_compound(
    cation: &Species,
    anion: &Species,
) -> (String, BTreeMap<String, i64>, f64) {
    let cation_charge = cation.charge.map(|c| c as i64).unwrap_or(1);
    let anion_charge = anion.charge.map(|c| c as i64).unwrap_or(-1);
    let (cation_sub, anion_sub) = crossover_subscripts(cation_charge, anion_charge);
    let composition = merge(
        scaled(&cation.composition, cation_sub),
        scaled(&anion.composition, anion_sub),
    );
    let formula = binary_formula(
        &cation.symbol,
        cation_sub,
        &anion.symbol,
        anion_sub,
        anion.is_polyatomic,
    );
    let molar_mass = cation.atomic_mass * cation_sub as f64 + anion.atomic_mass * anion_sub as f64;
    (formula, composition, molar_mass)
}
