//! A reactant compound built from 1 or 2 [`Species`] using the existing bonding
//! rules. Ports the iOS ChemCore `Reactant.swift` `makeReactant` builder.

use std::collections::BTreeMap;

use crate::classification::ElementClass;
use crate::engine::bonding::{determine_bonding, BondingType};
use crate::engine::composition::{ionic_compound, merge, scaled};
use crate::engine::covalent::{covalent_stoich, iupac_first};
use crate::engine::formula_text::{binary_formula, formula_subscript};
use crate::engine::species::Species;
use crate::engine::stoichiometry::is_naturally_diatomic;

/// A reactant compound built from 1 or 2 species using the existing bonding rules.
#[derive(Debug, Clone, PartialEq)]
pub struct Reactant {
    pub species: Vec<Species>,
    pub formula: String,
    pub composition: BTreeMap<String, i64>,
    pub molar_mass: f64,
    pub cation: Option<Species>,
    pub anion: Option<Species>,
    pub is_bare_element: bool,
}

/// Metals bias toward cation, non-metals toward anion, when charge is absent.
fn cation_bias(s: &Species) -> i64 {
    if s.element_class == ElementClass::Metal {
        1
    } else {
        -1
    }
}

/// Build a [`Reactant`] compound from 1 or 2 species.
///
/// - 1 species: a bare element, doubled to a diatomic molecule when it is one of
///   the [`crate::NATURALLY_DIATOMIC`] elements and isn't itself a polyatomic ion.
/// - 2 species: ionic (crossover of cation/anion charges) when either species is
///   polyatomic, the broad element-class bonding rule says ionic, or the two
///   species carry explicit opposite charges (the acid-as-ionic rule, e.g. HCl);
///   otherwise covalent, using octet stoichiometry and IUPAC element ordering.
pub fn make_reactant(species: &[Species]) -> Reactant {
    if species.len() == 1 {
        let s = &species[0];
        let diatomic = is_naturally_diatomic(&s.symbol) && !s.is_polyatomic;
        let count = if diatomic { 2 } else { 1 };
        let composition = scaled(&s.composition, count);
        let formula = format!("{}{}", s.symbol, formula_subscript(count));
        return Reactant {
            species: species.to_vec(),
            formula,
            composition,
            molar_mass: s.atomic_mass * count as f64,
            cation: None,
            anion: None,
            is_bare_element: !s.is_polyatomic,
        };
    }

    let a = &species[0];
    let b = &species[1];

    // Check for explicit opposite charges (e.g., H+ and Cl-).
    // An acid like HCl is covalent by electronegativity (both non-metals),
    // but is modeled as ionic (H+ cation + anion) for reaction purposes.
    // This relies on callers setting `charge` only to express ionic intent;
    // neutral elements must carry `charge == None` for the covalent path to apply.
    let has_opposite_charges = match (a.charge, b.charge) {
        (Some(ac), Some(bc)) => (ac as i64) * (bc as i64) < 0,
        _ => false,
    };
    let ionic = a.is_polyatomic
        || b.is_polyatomic
        || has_opposite_charges
        || determine_bonding(a.element_class, b.element_class) == BondingType::Ionic;

    if ionic {
        // Cation = positive charge (or the metal); anion = the other.
        let a_key = a.charge.map(|c| c as i64).unwrap_or_else(|| cation_bias(a));
        let b_key = b.charge.map(|c| c as i64).unwrap_or_else(|| cation_bias(b));
        let (cation, anion) = if a_key >= b_key { (a, b) } else { (b, a) };
        let (formula, composition, molar_mass) = ionic_compound(cation, anion);
        return Reactant {
            species: species.to_vec(),
            formula,
            composition,
            molar_mass,
            cation: Some(cation.clone()),
            anion: Some(anion.clone()),
            is_bare_element: false,
        };
    }

    // Covalent.
    let stoich = covalent_stoich(
        a.valence_electrons as i64,
        a.group as i64,
        a.period as i64,
        b.valence_electrons as i64,
        b.group as i64,
        b.period as i64,
    );
    let a_first = iupac_first(&a.symbol, &b.symbol);
    let (first, first_n, second, second_n) = if a_first {
        (a, stoich.n_a, b, stoich.n_b)
    } else {
        (b, stoich.n_b, a, stoich.n_a)
    };
    let composition = merge(
        scaled(&a.composition, stoich.n_a),
        scaled(&b.composition, stoich.n_b),
    );
    let formula = binary_formula(&first.symbol, first_n, &second.symbol, second_n, false);
    let molar_mass = a.atomic_mass * stoich.n_a as f64 + b.atomic_mass * stoich.n_b as f64;
    Reactant {
        species: species.to_vec(),
        formula,
        composition,
        molar_mass,
        cation: None,
        anion: None,
        is_bare_element: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(
        symbol: &str,
        mass: f64,
        class: ElementClass,
        charge: Option<i8>,
        ve: u8,
        group: u8,
        period: u8,
    ) -> Species {
        let mut composition = BTreeMap::new();
        composition.insert(symbol.to_string(), 1);
        Species::new(
            symbol.to_string(),
            mass,
            charge,
            class,
            false,
            ve,
            group,
            period,
            composition,
        )
    }

    fn poly(symbol: &str, mass: f64, charge: i8, comp: &[(&str, i64)]) -> Species {
        let composition = comp.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        Species::new(
            symbol.to_string(),
            mass,
            Some(charge),
            ElementClass::NonMetal,
            true,
            0,
            0,
            0,
            composition,
        )
    }

    #[test]
    fn test_bare_metal() {
        let r = make_reactant(&[element("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0)]);
        assert!(r.is_bare_element);
        assert_eq!(r.formula, "Zn");
        let mut expected = BTreeMap::new();
        expected.insert("Zn".to_string(), 1);
        assert_eq!(r.composition, expected);
    }

    #[test]
    fn test_bare_diatomic() {
        let r = make_reactant(&[element(
            "O",
            16.0,
            ElementClass::NonMetal,
            Some(-2),
            0,
            0,
            0,
        )]);
        assert_eq!(r.formula, "O₂");
        let mut expected = BTreeMap::new();
        expected.insert("O".to_string(), 2);
        assert_eq!(r.composition, expected);
        assert!((r.molar_mass - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_ionic_nacl() {
        let na = element("Na", 23.0, ElementClass::Metal, Some(1), 0, 0, 0);
        let cl = element("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0);
        let r = make_reactant(&[na, cl]);
        assert_eq!(r.formula, "NaCl");
        let mut expected = BTreeMap::new();
        expected.insert("Na".to_string(), 1);
        expected.insert("Cl".to_string(), 1);
        assert_eq!(r.composition, expected);
        assert_eq!(r.cation.as_ref().map(|s| s.symbol.as_str()), Some("Na"));
        assert_eq!(r.anion.as_ref().map(|s| s.symbol.as_str()), Some("Cl"));
        assert!(!r.is_bare_element);
    }

    #[test]
    fn test_ionic_with_polyatomic_sulfate() {
        let na = element("Na", 23.0, ElementClass::Metal, Some(1), 0, 0, 0);
        let so4 = poly("SO₄", 96.06, -2, &[("S", 1), ("O", 4)]);
        let r = make_reactant(&[na, so4]);
        assert_eq!(r.formula, "Na₂SO₄");
        let mut expected = BTreeMap::new();
        expected.insert("Na".to_string(), 2);
        expected.insert("S".to_string(), 1);
        expected.insert("O".to_string(), 4);
        assert_eq!(r.composition, expected);
        assert!((r.molar_mass - (2.0 * 23.0 + 96.06)).abs() < 1e-6);
    }

    #[test]
    fn test_covalent_methane() {
        let c = element("C", 12.011, ElementClass::NonMetal, None, 4, 14, 2);
        let h = element("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1);
        let r = make_reactant(&[c, h]);
        let mut expected = BTreeMap::new();
        expected.insert("C".to_string(), 1);
        expected.insert("H".to_string(), 4);
        assert_eq!(r.composition, expected);
        assert!(r.cation.is_none());
    }

    #[test]
    fn test_opposite_charges_treated_as_ionic() {
        let h = element("H", 1.008, ElementClass::NonMetal, Some(1), 0, 0, 0);
        let cl = element("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0);
        let r = make_reactant(&[h, cl]);
        assert_eq!(r.formula, "HCl");
        assert_eq!(r.cation.as_ref().map(|s| s.symbol.as_str()), Some("H"));
        assert_eq!(r.anion.as_ref().map(|s| s.symbol.as_str()), Some("Cl"));
        assert!(!r.is_bare_element);
    }

    #[test]
    fn test_chargeless_nonmetals_stay_covalent() {
        let c = element("C", 12.011, ElementClass::NonMetal, None, 4, 14, 2);
        let h = element("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1);
        let r = make_reactant(&[c, h]);
        assert!(r.cation.is_none());
        assert!(r.anion.is_none());
    }
}
