//! Reaction classification: combustion, single/double displacement, synthesis.
//! Ports the iOS ChemCore `Reaction/ReactionClass.swift`.

use crate::engine::reactant::Reactant;

/// The kind of reaction two [`Reactant`]s undergo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionClass {
    Synthesis,
    DoubleDisplacement,
    SingleDisplacement,
    Combustion,
    None,
}

pub(crate) fn is_dioxygen(r: &Reactant) -> bool {
    r.composition.len() == 1 && r.composition.get("O") == Some(&2)
}

/// A fuel for combustion must contain carbon or hydrogen. A bare metal/element + O₂
/// is direct combination (synthesis / oxidation), not combustion — e.g. Fe + O₂ → FeO.
fn is_fuel(r: &Reactant) -> bool {
    r.composition.contains_key("C") || r.composition.contains_key("H")
}

fn is_ionic_compound(r: &Reactant) -> bool {
    r.cation.is_some() && r.anion.is_some()
}

/// Classify a reaction between two [`Reactant`]s, in priority order:
/// combustion → single displacement → double displacement → synthesis → none.
pub fn classify_reaction(r1: &Reactant, r2: &Reactant) -> ReactionClass {
    // 1. Combustion: one side is O₂, the other burns.
    if (is_dioxygen(r1) && is_fuel(r2) && !is_dioxygen(r2))
        || (is_dioxygen(r2) && is_fuel(r1) && !is_dioxygen(r1))
    {
        return ReactionClass::Combustion;
    }
    // 2. Single displacement: exactly one bare element + one ionic compound.
    if (r1.is_bare_element && is_ionic_compound(r2))
        || (r2.is_bare_element && is_ionic_compound(r1))
    {
        return ReactionClass::SingleDisplacement;
    }
    // 3. Double displacement: both ionic compounds.
    if is_ionic_compound(r1) && is_ionic_compound(r2) {
        return ReactionClass::DoubleDisplacement;
    }
    // 4. Synthesis: two bare elements.
    if r1.is_bare_element && r2.is_bare_element {
        return ReactionClass::Synthesis;
    }
    ReactionClass::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classification::ElementClass;
    use crate::engine::reactant::make_reactant;
    use crate::engine::species::Species;
    use std::collections::BTreeMap;

    fn el(
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

    fn poly_ion(symbol: &str, mass: f64, charge: i8, comp: &[(&str, i64)]) -> Species {
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
    fn test_combustion_methane_and_o2() {
        let ch4 = make_reactant(&[
            el("C", 12.0, ElementClass::NonMetal, None, 4, 14, 2),
            el("H", 1.0, ElementClass::NonMetal, None, 1, 1, 1),
        ]);
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        assert_eq!(classify_reaction(&ch4, &o2), ReactionClass::Combustion);
    }

    #[test]
    fn test_single_displacement_zn_and_cuso4() {
        let zn = make_reactant(&[el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let cuso4 = make_reactant(&[
            el("Cu", 63.55, ElementClass::Metal, Some(2), 0, 0, 0),
            poly_ion("SO₄", 96.06, -2, &[("S", 1), ("O", 4)]),
        ]);
        assert_eq!(
            classify_reaction(&zn, &cuso4),
            ReactionClass::SingleDisplacement
        );
    }

    #[test]
    fn test_double_displacement_naoh_and_hcl() {
        let naoh = make_reactant(&[
            el("Na", 23.0, ElementClass::Metal, Some(1), 0, 0, 0),
            poly_ion("OH", 17.0, -1, &[("O", 1), ("H", 1)]),
        ]);
        let hcl = make_reactant(&[
            el("H", 1.0, ElementClass::NonMetal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        assert_eq!(
            classify_reaction(&naoh, &hcl),
            ReactionClass::DoubleDisplacement
        );
    }

    #[test]
    fn test_synthesis_two_bare_elements() {
        let na = make_reactant(&[el("Na", 23.0, ElementClass::Metal, Some(1), 0, 0, 0)]);
        let cl = make_reactant(&[el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0)]);
        assert_eq!(classify_reaction(&na, &cl), ReactionClass::Synthesis);
    }

    /// Regression: a bare metal + O₂ is direct combination (oxidation), not
    /// combustion — combustion requires a C/H fuel.
    #[test]
    fn test_metal_plus_o2_is_synthesis_not_combustion() {
        let fe = make_reactant(&[el("Fe", 55.85, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        assert_eq!(classify_reaction(&fe, &o2), ReactionClass::Synthesis);
    }
}
