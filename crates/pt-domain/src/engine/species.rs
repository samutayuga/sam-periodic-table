//! One placed species: a neutral element or a (poly)atomic ion.

use crate::ElementClass;
use std::collections::BTreeMap;

/// One placed species: a neutral element or a (poly)atomic ion.
#[derive(Debug, Clone, PartialEq)]
pub struct Species {
    pub symbol: String,
    pub atomic_mass: f64,
    pub charge: Option<i8>,
    pub element_class: ElementClass,
    pub is_polyatomic: bool,
    pub valence_electrons: u8,
    pub group: u8,
    pub period: u8,
    pub composition: BTreeMap<String, i64>,
}

impl Species {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        atomic_mass: f64,
        charge: Option<i8>,
        element_class: ElementClass,
        is_polyatomic: bool,
        valence_electrons: u8,
        group: u8,
        period: u8,
        composition: BTreeMap<String, i64>,
    ) -> Self {
        Self {
            symbol,
            atomic_mass,
            charge,
            element_class,
            is_polyatomic,
            valence_electrons,
            group,
            period,
            composition,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_species_with_expected_fields() {
        let mut composition = BTreeMap::new();
        composition.insert("O".to_string(), 4);
        composition.insert("S".to_string(), 1);

        let species = Species::new(
            "SO₄".to_string(),
            96.06,
            Some(-2),
            ElementClass::NonMetal,
            true,
            0,
            0,
            0,
            composition.clone(),
        );

        assert_eq!(species.symbol, "SO₄");
        assert_eq!(species.atomic_mass, 96.06);
        assert_eq!(species.charge, Some(-2));
        assert_eq!(species.element_class, ElementClass::NonMetal);
        assert!(species.is_polyatomic);
        assert_eq!(species.valence_electrons, 0);
        assert_eq!(species.group, 0);
        assert_eq!(species.period, 0);
        assert_eq!(species.composition, composition);
    }
}
