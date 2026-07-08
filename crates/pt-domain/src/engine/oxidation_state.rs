//! Assign oxidation states to every element in a neutral compound. Ports the
//! iOS ChemCore `OxidationState.swift` engine onto the shared domain types.

use std::collections::BTreeMap;

use crate::engine::polyatomic::{PolyatomicIon, POLYATOMIC_IONS};

const GROUP1: [&str; 6] = ["Li", "Na", "K", "Rb", "Cs", "Fr"];
const GROUP2: [&str; 6] = ["Be", "Mg", "Ca", "Sr", "Ba", "Ra"];
const HALOGENS_MINUS_ONE: [&str; 3] = ["Cl", "Br", "I"];

/// The fixed oxidation state for elements governed by a simple rule, else `None`.
/// No peroxide/hydride exceptions — this engine never produces them.
fn fixed_state(symbol: &str) -> Option<i64> {
    match symbol {
        "F" => Some(-1),
        "O" => Some(-2),
        "H" => Some(1),
        _ => {
            if HALOGENS_MINUS_ONE.contains(&symbol) {
                Some(-1)
            } else if GROUP1.contains(&symbol) {
                Some(1)
            } else if GROUP2.contains(&symbol) {
                Some(2)
            } else {
                None
            }
        }
    }
}

fn atom_count(composition: &[(&'static str, u8)]) -> i64 {
    composition.iter().map(|(_, n)| *n as i64).sum()
}

/// Largest k with `comp ⊇ k·ion`, or `None` if the ion is not wholly contained.
fn max_multiple(comp: &BTreeMap<String, i64>, ion: &[(&'static str, u8)]) -> Option<i64> {
    let mut k = i64::MAX;
    for (sym, n) in ion {
        let n = *n as i64;
        let have = *comp.get(*sym).unwrap_or(&0);
        if have < n {
            return None;
        }
        k = k.min(have / n);
    }
    if k == i64::MAX {
        None
    } else {
        Some(k)
    }
}

/// Oxidation states inside a polyatomic ion: O/H fixed, the central atom solved so
/// the ion's atoms sum to its charge. `None` if not resolvable to a single unknown.
fn states_within_ion(ion: &PolyatomicIon) -> Option<BTreeMap<String, i64>> {
    let mut states: BTreeMap<String, i64> = BTreeMap::new();
    let mut assigned_sum: i64 = 0;
    let mut unknown: Option<&'static str> = None;
    for (sym, n) in ion.composition {
        let n = *n as i64;
        if let Some(fx) = fixed_state(sym) {
            states.insert(sym.to_string(), fx);
            assigned_sum += fx * n;
        } else if unknown.is_none() {
            unknown = Some(sym);
        } else {
            return None;
        }
    }
    if let Some(u) = unknown {
        let count = ion
            .composition
            .iter()
            .find(|(sym, _)| *sym == u)
            .map(|(_, n)| *n as i64)
            .unwrap();
        let need = ion.charge as i64 - assigned_sum;
        if need % count != 0 {
            return None;
        }
        states.insert(u.to_string(), need / count);
    } else if assigned_sum != ion.charge as i64 {
        return None;
    }
    Some(states)
}

/// Try to read the compound as (counter-ion)·(known polyatomic ion): factor the ion
/// out, assign the disjoint remainder element the charge that balances it.
fn factor_polyatomic(comp: &BTreeMap<String, i64>) -> Option<BTreeMap<String, i64>> {
    let mut ions: Vec<&PolyatomicIon> = POLYATOMIC_IONS.iter().collect();
    ions.sort_by_key(|ion| std::cmp::Reverse(atom_count(ion.composition)));

    for ion in ions {
        let k = match max_multiple(comp, ion.composition) {
            Some(k) if k >= 1 => k,
            _ => continue,
        };
        let mut remainder = comp.clone();
        for (sym, n) in ion.composition {
            let entry = remainder.entry(sym.to_string()).or_insert(0);
            *entry -= (*n as i64) * k;
            if *entry == 0 {
                remainder.remove(*sym);
            }
        }
        // Remainder must be a single element, disjoint from the ion's elements.
        if remainder.len() != 1 {
            continue;
        }
        let (counter_sym, counter_count) = remainder.iter().next().unwrap();
        let ion_syms: Vec<&str> = ion.composition.iter().map(|(s, _)| *s).collect();
        if ion_syms.contains(&counter_sym.as_str()) {
            continue;
        }
        let ion_states = match states_within_ion(ion) {
            Some(s) => s,
            None => continue,
        };
        let counter_total = -(ion.charge as i64) * k;
        if *counter_count == 0 || counter_total % counter_count != 0 {
            continue;
        }
        let mut result = ion_states;
        result.insert(counter_sym.clone(), counter_total / counter_count);
        return Some(result);
    }
    None
}

/// Element rules + solve the single remaining unknown so a neutral compound sums to 0.
fn by_element_rules(comp: &BTreeMap<String, i64>) -> Option<BTreeMap<String, i64>> {
    let mut states: BTreeMap<String, i64> = BTreeMap::new();
    let mut assigned_sum: i64 = 0;
    let mut unknowns: Vec<&String> = Vec::new();
    for (sym, n) in comp {
        if let Some(fx) = fixed_state(sym) {
            states.insert(sym.clone(), fx);
            assigned_sum += fx * n;
        } else {
            unknowns.push(sym);
        }
    }
    if unknowns.len() > 1 {
        return None;
    }
    if let Some(u) = unknowns.first() {
        let count = comp[*u];
        let need = -assigned_sum;
        if need % count != 0 {
            return None;
        }
        states.insert((*u).clone(), need / count);
    } else if assigned_sum != 0 {
        return None;
    }
    Some(states)
}

/// Oxidation state of every element in a NEUTRAL compound, or `None` if the standard
/// rules leave it under-determined.
/// Known limitation: a compound with the same element in two oxidation environments
/// (e.g. NH₄NO₃) collapses to one composition entry and yields a single averaged
/// value rather than `None` — out of scope for this engine's reaction set.
pub fn oxidation_state(composition: &BTreeMap<String, i64>) -> Option<BTreeMap<String, i64>> {
    if composition.len() == 1 {
        let sym = composition.keys().next().unwrap();
        let mut result = BTreeMap::new();
        result.insert(sym.clone(), 0);
        return Some(result);
    }
    if let Some(by_ion) = factor_polyatomic(composition) {
        return Some(by_ion);
    }
    by_element_rules(composition)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comp(pairs: &[(&str, i64)]) -> BTreeMap<String, i64> {
        pairs.iter().map(|(s, n)| (s.to_string(), *n)).collect()
    }

    #[test]
    fn free_element_is_zero() {
        assert_eq!(
            oxidation_state(&comp(&[("Zn", 1)])),
            Some(comp(&[("Zn", 0)]))
        );
        assert_eq!(oxidation_state(&comp(&[("O", 2)])), Some(comp(&[("O", 0)])));
        // O₂
    }

    #[test]
    fn binary_ionic() {
        assert_eq!(
            oxidation_state(&comp(&[("Na", 1), ("Cl", 1)])),
            Some(comp(&[("Na", 1), ("Cl", -1)]))
        );
        assert_eq!(
            oxidation_state(&comp(&[("Mg", 1), ("O", 1)])),
            Some(comp(&[("Mg", 2), ("O", -2)]))
        );
    }

    #[test]
    fn solve_by_difference() {
        assert_eq!(
            oxidation_state(&comp(&[("C", 1), ("O", 2)])),
            Some(comp(&[("C", 4), ("O", -2)]))
        ); // CO₂
        assert_eq!(
            oxidation_state(&comp(&[("K", 1), ("Mn", 1), ("O", 4)])),
            Some(comp(&[("K", 1), ("Mn", 7), ("O", -2)]))
        ); // KMnO₄
        assert_eq!(
            oxidation_state(&comp(&[("Fe", 1), ("Cl", 3)])),
            Some(comp(&[("Fe", 3), ("Cl", -1)]))
        ); // FeCl₃
    }

    #[test]
    fn water_uses_element_rules() {
        assert_eq!(
            oxidation_state(&comp(&[("H", 2), ("O", 1)])),
            Some(comp(&[("H", 1), ("O", -2)]))
        );
    }

    #[test]
    fn polyatomic_factoring() {
        assert_eq!(
            oxidation_state(&comp(&[("Na", 1), ("O", 1), ("H", 1)])),
            Some(comp(&[("Na", 1), ("O", -2), ("H", 1)]))
        ); // NaOH
        assert_eq!(
            oxidation_state(&comp(&[("Na", 2), ("S", 1), ("O", 4)])),
            Some(comp(&[("Na", 1), ("S", 6), ("O", -2)]))
        ); // Na₂SO₄
        assert_eq!(
            oxidation_state(&comp(&[("Cu", 1), ("S", 1), ("O", 4)])),
            Some(comp(&[("Cu", 2), ("S", 6), ("O", -2)]))
        ); // CuSO₄
        assert_eq!(
            oxidation_state(&comp(&[("Na", 2), ("C", 1), ("O", 3)])),
            Some(comp(&[("Na", 1), ("C", 4), ("O", -2)]))
        ); // Na₂CO₃
        assert_eq!(
            oxidation_state(&comp(&[("N", 1), ("H", 4), ("Cl", 1)])),
            Some(comp(&[("N", -3), ("H", 1), ("Cl", -1)]))
        ); // NH₄Cl
    }

    #[test]
    fn indeterminate_returns_none() {
        assert_eq!(oxidation_state(&comp(&[("Cu", 1), ("S", 1)])), None); // CuS: two rule-less elements
    }
}
