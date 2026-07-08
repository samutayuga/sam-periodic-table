//! End-to-end reaction solver: classify → predict products → balance →
//! limiting-reactant stoichiometry. Ports the iOS ChemCore
//! `Reaction/ReactionSolver.swift`.

use std::collections::BTreeMap;

use crate::engine::balancer::balance;
use crate::engine::product_prediction::{predict_products, Prediction};
use crate::engine::reactant::Reactant;
use crate::engine::reaction_class::{classify_reaction, ReactionClass};
use crate::engine::stoichiometry::{
    is_naturally_diatomic, AmountResult, LimitingSide, QuantityUnit, ReactantEntry,
};

/// One balanced reactant or product term: its coefficient, display formula,
/// molar mass, and elemental composition.
#[derive(Debug, Clone, PartialEq)]
pub struct BalancedTerm {
    pub coeff: i64,
    pub formula: String,
    pub molar_mass: f64,
    pub composition: BTreeMap<String, i64>,
}

/// The full result of solving a two-reactant reaction end to end.
#[derive(Debug, Clone, PartialEq)]
pub struct ReactionResult {
    pub reaction_class: ReactionClass,
    pub reactants: Vec<BalancedTerm>,
    pub products: Vec<BalancedTerm>,
    pub limiting: LimitingSide,
    pub yields: Vec<AmountResult>,
    pub excess: AmountResult,
    pub messages: Vec<String>,
    pub feasible: bool,
}

/// Why [`solve_reaction`] could not produce a result.
#[derive(Debug, Clone, PartialEq)]
pub enum ReactionError {
    Unbalanceable,
    NoProducts,
    UnknownReactionClass,
    MissingAtomicMass(String),
}

fn molar_mass(
    comp: &BTreeMap<String, i64>,
    atomic_mass: &impl Fn(&str) -> Option<f64>,
) -> Option<f64> {
    let mut total = 0.0;
    for (sym, n) in comp {
        let m = atomic_mass(sym)?;
        total += m * (*n as f64);
    }
    Some(total)
}

fn first_missing(
    comp: &BTreeMap<String, i64>,
    atomic_mass: &impl Fn(&str) -> Option<f64>,
) -> String {
    comp.keys()
        .find(|sym| atomic_mass(sym).is_none())
        .cloned()
        .unwrap_or_else(|| "?".to_string())
}

/// Solve a reaction between `r1` and `r2` end to end: classify, predict
/// products, balance, then resolve limiting-reactant amounts from the
/// optional entered quantities.
pub fn solve_reaction(
    r1: &Reactant,
    r2: &Reactant,
    entry1: Option<ReactantEntry>,
    entry2: Option<ReactantEntry>,
    atomic_mass: impl Fn(&str) -> Option<f64>,
) -> Result<ReactionResult, ReactionError> {
    let cls = classify_reaction(r1, r2);
    if cls == ReactionClass::None {
        return Err(ReactionError::UnknownReactionClass);
    }

    // Product prediction — infeasible is a valid, non-error result.
    let prediction = predict_products(cls, r1, r2);
    let product_list = match prediction {
        Prediction::Infeasible(reason) => {
            let reactants = [r1, r2]
                .into_iter()
                .map(|r| BalancedTerm {
                    coeff: 1,
                    formula: r.formula.clone(),
                    molar_mass: r.molar_mass,
                    composition: r.composition.clone(),
                })
                .collect();
            return Ok(ReactionResult {
                reaction_class: cls,
                reactants,
                products: Vec::new(),
                limiting: LimitingSide::Both,
                yields: Vec::new(),
                excess: AmountResult {
                    moles: 0.0,
                    mass: 0.0,
                },
                messages: vec![reason],
                feasible: false,
            });
        }
        Prediction::Products(list) => {
            if list.is_empty() {
                return Err(ReactionError::NoProducts);
            }
            list
        }
    };

    // Balance.
    let reactant_comps = vec![r1.composition.clone(), r2.composition.clone()];
    let product_comps: Vec<BTreeMap<String, i64>> =
        product_list.iter().map(|p| p.composition.clone()).collect();
    let coeffs = balance(&reactant_comps, &product_comps).ok_or(ReactionError::Unbalanceable)?;
    let coeff_a = coeffs[0];
    let coeff_b = coeffs[1];
    let product_coeffs = &coeffs[2..];

    // Molar masses.
    let mm_a = molar_mass(&r1.composition, &atomic_mass).ok_or_else(|| {
        ReactionError::MissingAtomicMass(first_missing(&r1.composition, &atomic_mass))
    })?;
    let mm_b = molar_mass(&r2.composition, &atomic_mass).ok_or_else(|| {
        ReactionError::MissingAtomicMass(first_missing(&r2.composition, &atomic_mass))
    })?;
    let mut product_terms: Vec<BalancedTerm> = Vec::new();
    let mut product_masses: Vec<f64> = Vec::new();
    for (p, &c) in product_list.iter().zip(product_coeffs.iter()) {
        let mm = molar_mass(&p.composition, &atomic_mass).ok_or_else(|| {
            ReactionError::MissingAtomicMass(first_missing(&p.composition, &atomic_mass))
        })?;
        product_masses.push(mm);
        product_terms.push(BalancedTerm {
            coeff: c,
            formula: p.formula.clone(),
            molar_mass: mm,
            composition: p.composition.clone(),
        });
    }

    // Extent ξ from limiting reactant.
    let moles = |entry: &Option<ReactantEntry>, mm: f64| -> Option<f64> {
        entry.map(|e| match e.unit {
            QuantityUnit::Mole => e.value,
            QuantityUnit::Mass => e.value / mm,
        })
    };
    let mol_a = moles(&entry1, mm_a);
    let mol_b = moles(&entry2, mm_b);
    let extent_a = mol_a.map(|m| m / coeff_a as f64);
    let extent_b = mol_b.map(|m| m / coeff_b as f64);

    let (xi, limiting) = match (extent_a, extent_b) {
        (None, None) => (1.0, LimitingSide::Both),
        (Some(ea), None) => (ea, LimitingSide::A),
        (None, Some(eb)) => (eb, LimitingSide::B),
        (Some(ea), Some(eb)) => {
            if ea < eb {
                (ea, LimitingSide::A)
            } else if eb < ea {
                (eb, LimitingSide::B)
            } else {
                (ea, LimitingSide::Both)
            }
        }
    };

    let yields: Vec<AmountResult> = product_coeffs
        .iter()
        .zip(product_masses.iter())
        .map(|(&c, &mm)| AmountResult {
            moles: c as f64 * xi,
            mass: c as f64 * xi * mm,
        })
        .collect();

    let mut excess = AmountResult {
        moles: 0.0,
        mass: 0.0,
    };
    if limiting == LimitingSide::A {
        if let Some(mb) = mol_b {
            let left = (mb - coeff_b as f64 * xi).max(0.0);
            excess = AmountResult {
                moles: left,
                mass: left * mm_b,
            };
        }
    } else if limiting == LimitingSide::B {
        if let Some(ma) = mol_a {
            let left = (ma - coeff_a as f64 * xi).max(0.0);
            excess = AmountResult {
                moles: left,
                mass: left * mm_a,
            };
        }
    }

    let mut messages: Vec<String> = Vec::new();
    if r1.formula.ends_with('₂')
        && is_naturally_diatomic(r1.species.first().map(|s| s.symbol.as_str()).unwrap_or(""))
    {
        messages.push(format!(
            "{} only exists as {}₂",
            r1.species[0].symbol, r1.species[0].symbol
        ));
    }
    if r2.formula.ends_with('₂')
        && is_naturally_diatomic(r2.species.first().map(|s| s.symbol.as_str()).unwrap_or(""))
    {
        messages.push(format!(
            "{} only exists as {}₂",
            r2.species[0].symbol, r2.species[0].symbol
        ));
    }

    let reactants = vec![
        BalancedTerm {
            coeff: coeff_a,
            formula: r1.formula.clone(),
            molar_mass: mm_a,
            composition: r1.composition.clone(),
        },
        BalancedTerm {
            coeff: coeff_b,
            formula: r2.formula.clone(),
            molar_mass: mm_b,
            composition: r2.composition.clone(),
        },
    ];

    Ok(ReactionResult {
        reaction_class: cls,
        reactants,
        products: product_terms,
        limiting,
        yields,
        excess,
        messages,
        feasible: true,
    })
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

    /// Small hard-coded atomic-mass table mirroring the Swift test fixture.
    fn mass(symbol: &str) -> Option<f64> {
        match symbol {
            "H" => Some(1.008),
            "O" => Some(16.0),
            "Na" => Some(22.99),
            "Cl" => Some(35.45),
            "C" => Some(12.011),
            "S" => Some(32.06),
            "Zn" => Some(65.38),
            "Cu" => Some(63.55),
            _ => None,
        }
    }

    #[test]
    fn test_neutralisation_balances_and_is_feasible() {
        let naoh = make_reactant(&[
            el("Na", 22.99, ElementClass::Metal, Some(1), 0, 0, 0),
            poly_ion("OH", 17.008, -1, &[("O", 1), ("H", 1)]),
        ]);
        let hcl = make_reactant(&[
            el("H", 1.008, ElementClass::NonMetal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        let out = solve_reaction(&naoh, &hcl, None, None, mass);
        let r = out.expect("expected success");
        assert_eq!(r.reaction_class, ReactionClass::DoubleDisplacement);
        assert!(r.feasible);
        assert_eq!(
            r.reactants.iter().map(|t| t.coeff).collect::<Vec<_>>(),
            vec![1, 1]
        );
        let mut formulas: Vec<&str> = r.products.iter().map(|p| p.formula.as_str()).collect();
        formulas.sort();
        assert_eq!(formulas, vec!["H₂O", "NaCl"]);
    }

    #[test]
    fn test_combustion_methane_coefficients() {
        let ch4 = make_reactant(&[
            el("C", 12.011, ElementClass::NonMetal, None, 4, 14, 2),
            el("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1),
        ]);
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        let out = solve_reaction(&ch4, &o2, None, None, mass);
        let r = out.expect("expected success");
        assert_eq!(r.reaction_class, ReactionClass::Combustion);
        // CH4 + 2O2 -> CO2 + 2H2O
        assert_eq!(
            r.reactants.iter().map(|t| t.coeff).collect::<Vec<_>>(),
            vec![1, 2]
        );
        let co2 = r.products.iter().find(|p| p.formula == "CO₂");
        let h2o = r.products.iter().find(|p| p.formula == "H₂O");
        assert_eq!(co2.map(|p| p.coeff), Some(1));
        assert_eq!(h2o.map(|p| p.coeff), Some(2));
    }

    #[test]
    fn test_single_displacement_infeasible_result() {
        let cu = make_reactant(&[el("Cu", 63.55, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let znso4 = make_reactant(&[
            el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0),
            poly_ion("SO₄", 96.06, -2, &[("S", 1), ("O", 4)]),
        ]);
        let out = solve_reaction(&cu, &znso4, None, None, mass);
        let r = out.expect("expected success");
        assert!(!r.feasible);
        assert!(r.messages.iter().any(|m| m.contains("activity series")));
    }

    #[test]
    fn test_yield_scales_with_limiting_reactant() {
        // 2 mol NaOH + 1 mol HCl -> HCl limits, 1 mol NaCl + 1 mol H2O, 1 mol NaOH excess.
        let naoh = make_reactant(&[
            el("Na", 22.99, ElementClass::Metal, Some(1), 0, 0, 0),
            poly_ion("OH", 17.008, -1, &[("O", 1), ("H", 1)]),
        ]);
        let hcl = make_reactant(&[
            el("H", 1.008, ElementClass::NonMetal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        let out = solve_reaction(
            &naoh,
            &hcl,
            Some(ReactantEntry {
                value: 2.0,
                unit: QuantityUnit::Mole,
            }),
            Some(ReactantEntry {
                value: 1.0,
                unit: QuantityUnit::Mole,
            }),
            mass,
        );
        let r = out.expect("expected success");
        assert_eq!(r.limiting, LimitingSide::B);
        let yield_index = r.products.iter().position(|p| p.formula == "NaCl").unwrap();
        let nacl = &r.products[yield_index];
        assert_eq!(nacl.coeff, 1);
        assert!((r.yields[yield_index].moles - 1.0).abs() < 1e-6);
        assert!((r.excess.moles - 1.0).abs() < 1e-6);
    }

    fn naoh_fixture() -> Reactant {
        make_reactant(&[
            el("Na", 22.99, ElementClass::Metal, Some(1), 0, 0, 0),
            poly_ion("OH", 17.008, -1, &[("O", 1), ("H", 1)]),
        ])
    }

    fn hcl_fixture() -> Reactant {
        make_reactant(&[
            el("H", 1.008, ElementClass::NonMetal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ])
    }

    /// Neither combustion, single displacement, double displacement, nor
    /// synthesis: a covalent compound (CH4) paired with an ionic compound
    /// (NaCl) that isn't a bare element on either side.
    #[test]
    fn test_unclassifiable_pair_is_unknown_reaction_class() {
        let ch4 = make_reactant(&[
            el("C", 12.011, ElementClass::NonMetal, None, 4, 14, 2),
            el("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1),
        ]);
        let nacl = make_reactant(&[
            el("Na", 22.99, ElementClass::Metal, Some(1), 0, 0, 0),
            el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0),
        ]);
        let out = solve_reaction(&ch4, &nacl, None, None, mass);
        assert_eq!(out, Err(ReactionError::UnknownReactionClass));
    }

    /// `mm_a` fails to resolve when the atomic-mass table is missing an
    /// element from the first reactant (Na in NaOH); the error should name
    /// the first alphabetically missing symbol.
    #[test]
    fn test_missing_atomic_mass_reports_first_reactant_symbol() {
        fn mass_missing_na(symbol: &str) -> Option<f64> {
            match symbol {
                "O" => Some(16.0),
                "H" => Some(1.008),
                "Cl" => Some(35.45),
                _ => None,
            }
        }
        let naoh = naoh_fixture();
        let hcl = hcl_fixture();
        let out = solve_reaction(&naoh, &hcl, None, None, mass_missing_na);
        assert_eq!(out, Err(ReactionError::MissingAtomicMass("Na".to_string())));
    }

    /// `mm_b` fails to resolve when the first reactant's elements are all
    /// known but the second reactant (HCl) has an unknown element (Cl).
    #[test]
    fn test_missing_atomic_mass_reports_second_reactant_symbol() {
        fn mass_missing_cl(symbol: &str) -> Option<f64> {
            match symbol {
                "Na" => Some(22.99),
                "O" => Some(16.0),
                "H" => Some(1.008),
                _ => None,
            }
        }
        let naoh = naoh_fixture();
        let hcl = hcl_fixture();
        let out = solve_reaction(&naoh, &hcl, None, None, mass_missing_cl);
        assert_eq!(out, Err(ReactionError::MissingAtomicMass("Cl".to_string())));
    }

    /// Only reactant A's amount is entered (in moles): it alone determines
    /// the extent, so A is the limiting side and no B-side excess is
    /// computed (mol_b is None).
    #[test]
    fn test_only_reactant_a_entered_is_limiting() {
        let naoh = naoh_fixture();
        let hcl = hcl_fixture();
        let out = solve_reaction(
            &naoh,
            &hcl,
            Some(ReactantEntry {
                value: 1.0,
                unit: QuantityUnit::Mole,
            }),
            None,
            mass,
        );
        let r = out.expect("expected success");
        assert_eq!(r.limiting, LimitingSide::A);
        assert!((r.excess.moles - 0.0).abs() < 1e-9);
        let nacl_index = r.products.iter().position(|p| p.formula == "NaCl").unwrap();
        assert!((r.yields[nacl_index].moles - 1.0).abs() < 1e-9);
    }

    /// Only reactant B's amount is entered, given as a mass (grams) rather
    /// than moles: it alone determines the extent, so B is the limiting
    /// side. Exercises the `QuantityUnit::Mass` conversion branch.
    #[test]
    fn test_only_reactant_b_entered_by_mass_is_limiting() {
        let naoh = naoh_fixture();
        let hcl = hcl_fixture();
        // 18.229 g HCl / 36.458 g/mol (H 1.008 + Cl 35.45) = 0.5 mol.
        let out = solve_reaction(
            &naoh,
            &hcl,
            None,
            Some(ReactantEntry {
                value: 18.229,
                unit: QuantityUnit::Mass,
            }),
            mass,
        );
        let r = out.expect("expected success");
        assert_eq!(r.limiting, LimitingSide::B);
        let nacl_index = r.products.iter().position(|p| p.formula == "NaCl").unwrap();
        assert!((r.yields[nacl_index].moles - 0.5).abs() < 1e-3);
    }

    /// Both reactants entered with amounts that give exactly equal extents:
    /// neither runs out first, so limiting is `Both` and no excess remains.
    #[test]
    fn test_equal_extents_give_limiting_both() {
        let naoh = naoh_fixture();
        let hcl = hcl_fixture();
        let out = solve_reaction(
            &naoh,
            &hcl,
            Some(ReactantEntry {
                value: 1.0,
                unit: QuantityUnit::Mole,
            }),
            Some(ReactantEntry {
                value: 1.0,
                unit: QuantityUnit::Mole,
            }),
            mass,
        );
        let r = out.expect("expected success");
        assert_eq!(r.limiting, LimitingSide::Both);
        assert!((r.excess.moles - 0.0).abs() < 1e-9);
        let nacl_index = r.products.iter().position(|p| p.formula == "NaCl").unwrap();
        assert!((r.yields[nacl_index].moles - 1.0).abs() < 1e-9);
    }

    /// Both reactants entered with unequal extents (A smaller): A is
    /// limiting and leftover B is reported as excess.
    #[test]
    fn test_reactant_a_limits_with_b_excess() {
        let naoh = naoh_fixture();
        let hcl = hcl_fixture();
        let out = solve_reaction(
            &naoh,
            &hcl,
            Some(ReactantEntry {
                value: 1.0,
                unit: QuantityUnit::Mole,
            }),
            Some(ReactantEntry {
                value: 2.0,
                unit: QuantityUnit::Mole,
            }),
            mass,
        );
        let r = out.expect("expected success");
        assert_eq!(r.limiting, LimitingSide::A);
        assert!((r.excess.moles - 1.0).abs() < 1e-9);
        // 1 mol HCl left over at 36.458 g/mol.
        assert!((r.excess.mass - 36.458).abs() < 1e-3);
    }

    /// When the first reactant itself is a naturally diatomic element (O2
    /// listed before the fuel), the "only exists as X2" message is emitted
    /// for reactant A, not just for reactant B.
    #[test]
    fn test_diatomic_message_emitted_for_first_reactant() {
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        let ch4 = make_reactant(&[
            el("C", 12.011, ElementClass::NonMetal, None, 4, 14, 2),
            el("H", 1.008, ElementClass::NonMetal, None, 1, 1, 1),
        ]);
        let out = solve_reaction(&o2, &ch4, None, None, mass);
        let r = out.expect("expected success");
        assert_eq!(r.reaction_class, ReactionClass::Combustion);
        assert!(r.feasible);
        assert!(r.messages.contains(&"O only exists as O₂".to_string()));
    }

    /// A full single-displacement solve (not just prediction) that reaches
    /// molar-mass lookups for every element involved (Zn, Cu, S, O),
    /// with no entered amounts so both sides default to a 1-mol basis.
    #[test]
    fn test_single_displacement_full_solve_uses_all_elements() {
        let zn = make_reactant(&[el("Zn", 65.38, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let cuso4 = make_reactant(&[
            el("Cu", 63.55, ElementClass::Metal, Some(2), 0, 0, 0),
            poly_ion("SO₄", 96.06, -2, &[("S", 1), ("O", 4)]),
        ]);
        let out = solve_reaction(&zn, &cuso4, None, None, mass);
        let r = out.expect("expected success");
        assert!(r.feasible);
        assert_eq!(r.limiting, LimitingSide::Both);
        let mut formulas: Vec<&str> = r.products.iter().map(|p| p.formula.as_str()).collect();
        formulas.sort();
        assert_eq!(formulas, vec!["Cu", "ZnSO₄"]);
        let cu = r.products.iter().find(|p| p.formula == "Cu").unwrap();
        assert!((cu.molar_mass - 63.55).abs() < 1e-6);
    }

    /// A genuine synthesis reaction (two bare elements, neither ionic nor a
    /// fuel/O2 pair) exercises the `ReactionClass::Synthesis` product-
    /// prediction branch end to end via `solve_reaction`: 2Na + Cl2 -> 2NaCl.
    #[test]
    fn test_synthesis_na_and_cl_forms_nacl() {
        let na = make_reactant(&[el("Na", 22.99, ElementClass::Metal, Some(1), 0, 0, 0)]);
        let cl = make_reactant(&[el("Cl", 35.45, ElementClass::NonMetal, Some(-1), 0, 0, 0)]);
        let out = solve_reaction(&na, &cl, None, None, mass);
        let r = out.expect("expected success");
        assert_eq!(r.reaction_class, ReactionClass::Synthesis);
        assert!(r.feasible);
        assert_eq!(r.products.len(), 1);
        assert_eq!(r.products[0].formula, "NaCl");
        assert_eq!(
            r.reactants.iter().map(|t| t.coeff).collect::<Vec<_>>(),
            vec![2, 1]
        );
        assert_eq!(r.products[0].coeff, 2);
    }

    /// When the shared atomic-mass table (rather than a purpose-built
    /// closure) is simply missing an element outside its known set, the
    /// catch-all `None` arm surfaces as a `MissingAtomicMass` error naming
    /// that element.
    #[test]
    fn test_missing_atomic_mass_via_shared_table_catch_all() {
        let mg = make_reactant(&[el("Mg", 24.3, ElementClass::Metal, Some(2), 0, 0, 0)]);
        let o2 = make_reactant(&[el("O", 16.0, ElementClass::NonMetal, Some(-2), 0, 0, 0)]);
        let out = solve_reaction(&mg, &o2, None, None, mass);
        assert_eq!(out, Err(ReactionError::MissingAtomicMass("Mg".to_string())));
    }
}
