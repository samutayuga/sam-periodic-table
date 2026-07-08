//! Product prediction: given a classified reaction and its two reactants,
//! predict the resulting products. Ports the iOS ChemCore
//! `Reaction/ProductPrediction.swift`.

use std::collections::BTreeMap;

use crate::classification::ElementClass;
use crate::engine::activity_series::displaces;
use crate::engine::composition::ionic_compound;
use crate::engine::formula_text::{binary_formula, crossover_subscripts, formula_subscript};
use crate::engine::reactant::{make_reactant, Reactant};
use crate::engine::reaction_class::{is_dioxygen, ReactionClass};
use crate::engine::species::Species;
use crate::engine::stoichiometry::is_naturally_diatomic;

/// A predicted reaction product: its display formula and elemental composition.
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub formula: String,
    pub composition: BTreeMap<String, i64>,
}

/// The result of predicting products for a classified reaction.
#[derive(Debug, Clone, PartialEq)]
pub enum Prediction {
    Products(Vec<Product>),
    Infeasible(String),
}

fn water() -> Product {
    Product {
        formula: "H₂O".to_string(),
        composition: [("H".to_string(), 2), ("O".to_string(), 1)]
            .into_iter()
            .collect(),
    }
}

fn carbon_dioxide() -> Product {
    Product {
        formula: "CO₂".to_string(),
        composition: [("C".to_string(), 1), ("O".to_string(), 2)]
            .into_iter()
            .collect(),
    }
}

/// A freed element from a displacement, as its stable molecular form
/// (diatomic for H, N, O, F, Cl, Br, I).
fn freed_element(symbol: &str) -> Product {
    if is_naturally_diatomic(symbol) {
        return Product {
            formula: format!("{symbol}{}", formula_subscript(2)),
            composition: [(symbol.to_string(), 2)].into_iter().collect(),
        };
    }
    Product {
        formula: symbol.to_string(),
        composition: [(symbol.to_string(), 1)].into_iter().collect(),
    }
}

/// Neutralise a cation with an anion into a single ionic Product.
fn ionic_product(cation: &Species, anion: &Species) -> Product {
    let (formula, composition, _molar_mass) = ionic_compound(cation, anion);
    Product {
        formula,
        composition,
    }
}

/// Predict the products of a classified reaction between two reactants.
pub fn predict_products(class: ReactionClass, r1: &Reactant, r2: &Reactant) -> Prediction {
    match class {
        ReactionClass::DoubleDisplacement => {
            let (Some(c1), Some(a1), Some(c2), Some(a2)) = (
                r1.cation.as_ref(),
                r1.anion.as_ref(),
                r2.cation.as_ref(),
                r2.anion.as_ref(),
            ) else {
                return Prediction::Infeasible("both reactants must be ionic".to_string());
            };
            let mut products = Vec::new();
            for (cat, an) in [(c1, a2), (c2, a1)] {
                if cat.symbol == "H" && an.symbol == "OH" {
                    products.push(water());
                } else if cat.symbol == "H" && an.symbol == "CO₃" {
                    products.push(carbon_dioxide());
                    products.push(water());
                } else {
                    products.push(ionic_product(cat, an));
                }
            }
            Prediction::Products(products)
        }

        ReactionClass::SingleDisplacement => {
            let (free, salt) = if r1.is_bare_element {
                (r1, r2)
            } else {
                (r2, r1)
            };
            let (Some(bound_cation), Some(anion), Some(free_species)) = (
                salt.cation.as_ref(),
                salt.anion.as_ref(),
                free.species.first(),
            ) else {
                return Prediction::Infeasible("salt reactant must be ionic".to_string());
            };
            // Metal free element displaces the salt's cation; halogen displaces the anion.
            if free_species.element_class == ElementClass::Metal {
                match displaces(&free_species.symbol, &bound_cation.symbol) {
                    Some(true) => {
                        let new_salt = ionic_product(free_species, anion);
                        let freed = freed_element(&bound_cation.symbol);
                        Prediction::Products(vec![new_salt, freed])
                    }
                    _ => Prediction::Infeasible(format!(
                        "{} is below {} in the activity series",
                        free_species.symbol, bound_cation.symbol
                    )),
                }
            } else {
                match displaces(&free_species.symbol, &anion.symbol) {
                    Some(true) => {
                        let new_salt = ionic_product(bound_cation, free_species);
                        let freed = freed_element(&anion.symbol);
                        Prediction::Products(vec![new_salt, freed])
                    }
                    _ => Prediction::Infeasible(format!(
                        "{} is below {} in the activity series",
                        free_species.symbol, anion.symbol
                    )),
                }
            }
        }

        ReactionClass::Combustion => {
            let fuel = if is_dioxygen(r1) { r2 } else { r1 };
            if fuel.composition.contains_key("C") {
                // hydrocarbon path
                let mut products = vec![carbon_dioxide()];
                if fuel.composition.contains_key("H") {
                    products.push(water());
                }
                return Prediction::Products(products);
            }
            if fuel.composition.contains_key("H") {
                return Prediction::Products(vec![water()]);
            }
            // Bare-element fuel → oxide via crossover against oxygen (charge -2).
            let Some(e) = fuel.species.first() else {
                return Prediction::Infeasible("no fuel".to_string());
            };
            // Requires e.charge to encode the fuel element's oxidation magnitude;
            // when charge is None this falls back to 2, which is only correct for +2 elements.
            let cation_charge = e.charge.map(|c| (c as i64).abs()).unwrap_or(2);
            let (cation_sub, anion_sub) = crossover_subscripts(cation_charge, -2);
            let oxide_comp: BTreeMap<String, i64> =
                [(e.symbol.clone(), cation_sub), ("O".to_string(), anion_sub)]
                    .into_iter()
                    .collect();
            let formula = binary_formula(&e.symbol, cation_sub, "O", anion_sub, false);
            Prediction::Products(vec![Product {
                formula,
                composition: oxide_comp,
            }])
        }

        ReactionClass::Synthesis => {
            let compound = make_reactant(&[r1.species[0].clone(), r2.species[0].clone()]);
            Prediction::Products(vec![Product {
                formula: compound.formula,
                composition: compound.composition,
            }])
        }

        ReactionClass::None => Prediction::Infeasible("no recognised reaction".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn formulas(p: &Prediction) -> Vec<String> {
        if let Prediction::Products(list) = p {
            let mut fs: Vec<String> = list.iter().map(|p| p.formula.clone()).collect();
            fs.sort();
            fs
        } else {
            Vec::new()
        }
    }

    #[test]
    fn test_neutralisation_to_water() {
        let naoh = make_reactant(&[
            el("Na", 23.0, ElementClass::Metal, Some(1), 0, 0, 0),
            poly_ion("OH", 17.0, -1, &[("O", 1), ("H", 1)]),
        ]);
        let hcl = make_reactant(&[
            el("H", 1.0, ElementClass::NonMetal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        let mut expected = vec!["H₂O".to_string(), "NaCl".to_string()];
        expected.sort();
        assert_eq!(
            formulas(&predict_products(
                ReactionClass::DoubleDisplacement,
                &naoh,
                &hcl
            )),
            expected
        );
    }

    #[test]
    fn test_carbonate_gives_co2_and_water() {
        let na2co3 = make_reactant(&[
            el("Na", 23.0, ElementClass::Metal, Some(1), 0, 0, 0),
            poly_ion("CO₃", 60.0, -2, &[("C", 1), ("O", 3)]),
        ]);
        let hcl = make_reactant(&[
            el("H", 1.0, ElementClass::NonMetal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        let mut expected = vec!["CO₂".to_string(), "H₂O".to_string(), "NaCl".to_string()];
        expected.sort();
        assert_eq!(
            formulas(&predict_products(
                ReactionClass::DoubleDisplacement,
                &na2co3,
                &hcl
            )),
            expected
        );
    }

    #[test]
    fn test_single_displacement_feasible() {
        let zn = make_reactant(&[el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let cuso4 = make_reactant(&[
            el("Cu", 63.55, ElementClass::Metal, Some(2), 0, 0, 0),
            poly_ion("SO₄", 96.06, -2, &[("S", 1), ("O", 4)]),
        ]);
        let mut expected = vec!["Cu".to_string(), "ZnSO₄".to_string()];
        expected.sort();
        assert_eq!(
            formulas(&predict_products(
                ReactionClass::SingleDisplacement,
                &zn,
                &cuso4
            )),
            expected
        );
    }

    #[test]
    fn test_single_displacement_infeasible() {
        let cu = make_reactant(&[el("Cu", 63.55, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let znso4 = make_reactant(&[
            el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0),
            poly_ion("SO₄", 96.06, -2, &[("S", 1), ("O", 4)]),
        ]);
        match predict_products(ReactionClass::SingleDisplacement, &cu, &znso4) {
            Prediction::Infeasible(reason) => assert!(reason.contains("activity series")),
            other => panic!("expected infeasible, got {other:?}"),
        }
    }

    #[test]
    fn test_combustion_hydrocarbon() {
        let ch4 = make_reactant(&[
            el("C", 12.0, ElementClass::NonMetal, None, 4, 14, 2),
            el("H", 1.0, ElementClass::NonMetal, None, 1, 1, 1),
        ]);
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        let mut expected = vec!["CO₂".to_string(), "H₂O".to_string()];
        expected.sort();
        assert_eq!(
            formulas(&predict_products(ReactionClass::Combustion, &ch4, &o2)),
            expected
        );
    }

    #[test]
    fn test_halogen_displacement_frees_diatomic() {
        let cl2 = make_reactant(&[el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0)]);
        let kbr = make_reactant(&[
            el("K", 39.1, ElementClass::Metal, Some(1), 0, 0, 0),
            el("Br", 79.9, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        let mut expected = vec!["Br₂".to_string(), "KCl".to_string()];
        expected.sort();
        assert_eq!(
            formulas(&predict_products(
                ReactionClass::SingleDisplacement,
                &cl2,
                &kbr
            )),
            expected
        );
    }

    #[test]
    fn test_metal_displaces_hydrogen_as_h2() {
        let zn = make_reactant(&[el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let hcl = make_reactant(&[
            el("H", 1.008, ElementClass::NonMetal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        let mut expected = vec!["H₂".to_string(), "ZnCl₂".to_string()];
        expected.sort();
        assert_eq!(
            formulas(&predict_products(
                ReactionClass::SingleDisplacement,
                &zn,
                &hcl
            )),
            expected
        );
    }

    /// `DoubleDisplacement` requires both reactants to be ionic (have a
    /// resolved cation and anion); a covalent compound on either side must
    /// be reported as infeasible rather than mispredicted.
    #[test]
    fn test_double_displacement_requires_both_ionic() {
        let ch4 = make_reactant(&[
            el("C", 12.011, ElementClass::NonMetal, None, 4, 14, 2),
            el("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1),
        ]);
        let naoh = make_reactant(&[
            el("Na", 23.0, ElementClass::Metal, Some(1), 0, 0, 0),
            poly_ion("OH", 17.0, -1, &[("O", 1), ("H", 1)]),
        ]);
        match predict_products(ReactionClass::DoubleDisplacement, &ch4, &naoh) {
            Prediction::Infeasible(reason) => assert_eq!(reason, "both reactants must be ionic"),
            other => panic!("expected infeasible, got {other:?}"),
        }
    }

    /// `SingleDisplacement` locates the bare element regardless of which
    /// side it's on: the salt-first, free-element-second ordering must
    /// resolve to the same products as the free-first ordering.
    #[test]
    fn test_single_displacement_salt_listed_first() {
        let zn = make_reactant(&[el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let cuso4 = make_reactant(&[
            el("Cu", 63.55, ElementClass::Metal, Some(2), 0, 0, 0),
            poly_ion("SO₄", 96.06, -2, &[("S", 1), ("O", 4)]),
        ]);
        let mut expected = vec!["Cu".to_string(), "ZnSO₄".to_string()];
        expected.sort();
        assert_eq!(
            formulas(&predict_products(
                ReactionClass::SingleDisplacement,
                &cuso4,
                &zn
            )),
            expected
        );
    }

    /// `SingleDisplacement` requires the non-bare-element side to be an
    /// ionic salt; a covalent compound there must be infeasible.
    #[test]
    fn test_single_displacement_requires_ionic_salt() {
        let zn = make_reactant(&[el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let ch4 = make_reactant(&[
            el("C", 12.011, ElementClass::NonMetal, None, 4, 14, 2),
            el("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1),
        ]);
        match predict_products(ReactionClass::SingleDisplacement, &zn, &ch4) {
            Prediction::Infeasible(reason) => assert_eq!(reason, "salt reactant must be ionic"),
            other => panic!("expected infeasible, got {other:?}"),
        }
    }

    /// A halogen that is below the salt's existing halogen in the activity
    /// series cannot displace it (iodine cannot displace chlorine).
    #[test]
    fn test_halogen_displacement_infeasible_when_outranked() {
        let i2 = make_reactant(&[el("I", 126.9, ElementClass::NonMetal, Some(-1), 0, 0, 0)]);
        let kcl = make_reactant(&[
            el("K", 39.1, ElementClass::Metal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        match predict_products(ReactionClass::SingleDisplacement, &i2, &kcl) {
            Prediction::Infeasible(reason) => assert!(reason.contains("activity series")),
            other => panic!("expected infeasible, got {other:?}"),
        }
    }

    /// A pure-hydrogen fuel (no carbon) burns to water only, without a CO2
    /// byproduct.
    #[test]
    fn test_combustion_pure_hydrogen_makes_only_water() {
        let h2 = make_reactant(&[el("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1)]);
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        assert_eq!(
            formulas(&predict_products(ReactionClass::Combustion, &h2, &o2)),
            vec!["H₂O".to_string()]
        );
    }

    #[test]
    fn test_bare_element_combustion_makes_correct_oxide() {
        let mg = make_reactant(&[el("Mg", 24.3, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        assert_eq!(
            formulas(&predict_products(ReactionClass::Combustion, &mg, &o2)),
            vec!["MgO".to_string()]
        );

        let al = make_reactant(&[el("Al", 27.0, ElementClass::Metal, Some(3), 0, 0, 0)]);
        assert_eq!(
            formulas(&predict_products(ReactionClass::Combustion, &al, &o2)),
            vec!["Al₂O₃".to_string()]
        );
    }
}
