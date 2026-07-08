//! Oxidation-state analysis of a solved reaction: redox verdict,
//! oxidising/reducing agents, per-element changes, and a template narrative.
//! Ports the iOS ChemCore `Reaction/RedoxAnalysis.swift`.

use std::collections::BTreeMap;

use crate::engine::oxidation_state::oxidation_state;
use crate::engine::reaction_solver::{BalancedTerm, ReactionResult};

/// The direction an element's oxidation state moved across a reaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxidationChange {
    Oxidised,
    Reduced,
    Unchanged,
}

/// One element's oxidation-state change from a reactant term to a product term.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementRedox {
    pub symbol: String,
    pub before: i64,
    pub after: i64,
    pub change: OxidationChange,
    pub reactant_formula: String,
    pub product_formula: String,
}

/// Redox analysis of a solved reaction.
#[derive(Debug, Clone, PartialEq)]
pub struct RedoxAnalysis {
    pub is_redox: bool,
    pub oxidising_agent: Option<String>,
    pub reducing_agent: Option<String>,
    pub changes: Vec<ElementRedox>,
    pub oxidation_states: BTreeMap<String, BTreeMap<String, i64>>,
    pub indeterminate: Vec<String>,
    pub narrative: Vec<String>,
}

/// Render a signed integer as `+n` / `0` / `−n`, using the Unicode minus sign
/// (U+2212), not the ASCII hyphen.
fn signed(n: i64) -> String {
    if n > 0 {
        format!("+{n}")
    } else if n < 0 {
        format!("\u{2212}{}", -n)
    } else {
        "0".to_string()
    }
}

fn empty_analysis() -> RedoxAnalysis {
    RedoxAnalysis {
        is_redox: false,
        oxidising_agent: None,
        reducing_agent: None,
        changes: Vec::new(),
        oxidation_states: BTreeMap::new(),
        indeterminate: Vec::new(),
        narrative: Vec::new(),
    }
}

/// One element's occurrence within a term: the term's formula and the
/// element's oxidation state in that term.
struct Occurrence<'a> {
    formula: &'a str,
    state: i64,
}

/// Map each element to its occurrences (formula, state) across `terms`,
/// skipping terms whose oxidation states could not be resolved.
fn occurrences<'a>(
    terms: &'a [BalancedTerm],
    states_by_formula: &BTreeMap<String, BTreeMap<String, i64>>,
) -> BTreeMap<String, Vec<Occurrence<'a>>> {
    let mut map: BTreeMap<String, Vec<Occurrence<'a>>> = BTreeMap::new();
    for term in terms {
        let Some(states) = states_by_formula.get(&term.formula) else {
            continue;
        };
        for (sym, &state) in states {
            map.entry(sym.clone()).or_default().push(Occurrence {
                formula: term.formula.as_str(),
                state,
            });
        }
    }
    map
}

/// Analyse a solved reaction for redox activity: the oxidation state of every
/// compound, per-element before/after changes, the oxidising/reducing agents,
/// and a template narrative built from formulas. Only feasible reactions with
/// products are analysed; anything else yields an empty analysis.
pub fn analyze_redox(result: &ReactionResult) -> RedoxAnalysis {
    if !result.feasible || result.products.is_empty() {
        return empty_analysis();
    }

    // Oxidation states per compound; unresolved compounds are recorded and skipped.
    let mut states_by_formula: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    let mut indeterminate: Vec<String> = Vec::new();
    for term in result.reactants.iter().chain(result.products.iter()) {
        match oxidation_state(&term.composition) {
            Some(states) => {
                states_by_formula.insert(term.formula.clone(), states);
            }
            None => indeterminate.push(term.formula.clone()),
        }
    }

    let reactant_occ = occurrences(&result.reactants, &states_by_formula);
    let product_occ = occurrences(&result.products, &states_by_formula);

    let mut changes: Vec<ElementRedox> = Vec::new();
    for sym in reactant_occ.keys() {
        let Some(product_occs) = product_occ.get(sym) else {
            continue;
        };
        let reactant_occs = &reactant_occ[sym];

        let before_states: std::collections::BTreeSet<i64> =
            reactant_occs.iter().map(|o| o.state).collect();
        let after_states: std::collections::BTreeSet<i64> =
            product_occs.iter().map(|o| o.state).collect();
        if before_states.len() != 1 || after_states.len() != 1 {
            // Conflicting states on one side → direction undeterminable; flag the compounds.
            indeterminate.extend(reactant_occs.iter().map(|o| o.formula.to_string()));
            indeterminate.extend(product_occs.iter().map(|o| o.formula.to_string()));
            continue;
        }
        let before = *before_states.iter().next().unwrap();
        let after = *after_states.iter().next().unwrap();
        if before == after {
            continue;
        }
        changes.push(ElementRedox {
            symbol: sym.clone(),
            before,
            after,
            change: if after > before {
                OxidationChange::Oxidised
            } else {
                OxidationChange::Reduced
            },
            reactant_formula: reactant_occs[0].formula.to_string(),
            product_formula: product_occs[0].formula.to_string(),
        });
    }

    let is_redox = !changes.is_empty();
    let reducing = changes
        .iter()
        .find(|c| c.change == OxidationChange::Oxidised);
    let oxidising = changes
        .iter()
        .find(|c| c.change == OxidationChange::Reduced);

    let mut narrative: Vec<String> = Vec::new();
    if !is_redox {
        narrative.push("This is a non-redox reaction — no oxidation states change.".to_string());
    } else {
        for c in &changes {
            let verb = if c.change == OxidationChange::Oxidised {
                "oxidised"
            } else {
                "reduced"
            };
            let dir = if c.change == OxidationChange::Oxidised {
                "increases"
            } else {
                "decreases"
            };
            narrative.push(format!(
                "{} is {} because {}'s oxidation state {} from {} in {} to {} in {}.",
                c.reactant_formula,
                verb,
                c.symbol,
                dir,
                signed(c.before),
                c.reactant_formula,
                signed(c.after),
                c.product_formula
            ));
        }
        if let (Some(ox), Some(red)) = (oxidising, reducing) {
            narrative.push(format!(
                "{} is the oxidising agent — it oxidises {} and is itself reduced, its oxidation state decreasing from {} to {}.",
                ox.reactant_formula, red.reactant_formula, signed(ox.before), signed(ox.after)
            ));
            narrative.push(format!(
                "{} is the reducing agent — it reduces {} and is itself oxidised, its oxidation state increasing from {} to {}.",
                red.reactant_formula, ox.reactant_formula, signed(red.before), signed(red.after)
            ));
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    let unique_indeterminate: Vec<String> = indeterminate
        .into_iter()
        .filter(|f| seen.insert(f.clone()))
        .collect();

    RedoxAnalysis {
        is_redox,
        oxidising_agent: oxidising.map(|c| c.reactant_formula.clone()),
        reducing_agent: reducing.map(|c| c.reactant_formula.clone()),
        changes,
        oxidation_states: states_by_formula,
        indeterminate: unique_indeterminate,
        narrative,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::reaction_class::ReactionClass;
    use crate::engine::stoichiometry::{AmountResult, LimitingSide};

    fn term(coeff: i64, formula: &str, comp: &[(&str, i64)]) -> BalancedTerm {
        BalancedTerm {
            coeff,
            formula: formula.to_string(),
            molar_mass: 0.0,
            composition: comp.iter().map(|(s, n)| (s.to_string(), *n)).collect(),
        }
    }

    fn result(
        reactants: Vec<BalancedTerm>,
        products: Vec<BalancedTerm>,
        feasible: bool,
    ) -> ReactionResult {
        ReactionResult {
            reaction_class: ReactionClass::SingleDisplacement,
            reactants,
            products,
            limiting: LimitingSide::Both,
            yields: Vec::new(),
            excess: AmountResult {
                moles: 0.0,
                mass: 0.0,
            },
            messages: Vec::new(),
            feasible,
        }
    }

    #[test]
    fn synthesis_is_redox() {
        let r = result(
            vec![term(2, "Na", &[("Na", 1)]), term(1, "Cl₂", &[("Cl", 2)])],
            vec![term(2, "NaCl", &[("Na", 1), ("Cl", 1)])],
            true,
        );
        let a = analyze_redox(&r);
        assert!(a.is_redox);
        assert_eq!(a.reducing_agent, Some("Na".to_string())); // Na 0 → +1 (oxidised)
        assert_eq!(a.oxidising_agent, Some("Cl₂".to_string())); // Cl 0 → −1 (reduced)
        let symbols: std::collections::BTreeSet<String> =
            a.changes.iter().map(|c| c.symbol.clone()).collect();
        assert_eq!(
            symbols,
            ["Na", "Cl"].into_iter().map(String::from).collect()
        );
    }

    #[test]
    fn single_displacement_agents() {
        let r = result(
            vec![
                term(1, "Zn", &[("Zn", 1)]),
                term(1, "CuSO₄", &[("Cu", 1), ("S", 1), ("O", 4)]),
            ],
            vec![
                term(1, "ZnSO₄", &[("Zn", 1), ("S", 1), ("O", 4)]),
                term(1, "Cu", &[("Cu", 1)]),
            ],
            true,
        );
        let a = analyze_redox(&r);
        assert!(a.is_redox);
        assert_eq!(a.reducing_agent, Some("Zn".to_string()));
        assert_eq!(a.oxidising_agent, Some("CuSO₄".to_string()));
        let zn = a.changes.iter().find(|c| c.symbol == "Zn").unwrap();
        assert_eq!((zn.before, zn.after), (0, 2));
        assert!(!a.changes.iter().any(|c| c.symbol == "S" || c.symbol == "O")); // unchanged
    }

    #[test]
    fn combustion_is_redox() {
        let r = result(
            vec![
                term(1, "CH₄", &[("C", 1), ("H", 4)]),
                term(2, "O₂", &[("O", 2)]),
            ],
            vec![
                term(1, "CO₂", &[("C", 1), ("O", 2)]),
                term(2, "H₂O", &[("H", 2), ("O", 1)]),
            ],
            true,
        );
        let a = analyze_redox(&r);
        assert!(a.is_redox);
        assert_eq!(a.reducing_agent, Some("CH₄".to_string())); // C −4 → +4
        assert_eq!(a.oxidising_agent, Some("O₂".to_string())); // O 0 → −2
    }

    #[test]
    fn neutralisation_is_non_redox() {
        let r = result(
            vec![
                term(1, "NaOH", &[("Na", 1), ("O", 1), ("H", 1)]),
                term(1, "HCl", &[("H", 1), ("Cl", 1)]),
            ],
            vec![
                term(1, "NaCl", &[("Na", 1), ("Cl", 1)]),
                term(1, "H₂O", &[("H", 2), ("O", 1)]),
            ],
            true,
        );
        let a = analyze_redox(&r);
        assert!(!a.is_redox);
        assert_eq!(a.oxidising_agent, None);
        assert_eq!(a.reducing_agent, None);
        assert!(a.changes.is_empty());
        assert_eq!(
            a.narrative,
            vec!["This is a non-redox reaction — no oxidation states change.".to_string()]
        );
    }

    #[test]
    fn infeasible_is_empty() {
        let r = result(vec![term(1, "Cu", &[("Cu", 1)])], Vec::new(), false);
        let a = analyze_redox(&r);
        assert!(!a.is_redox);
        assert!(a.changes.is_empty() && a.narrative.is_empty());
    }

    #[test]
    fn narrative_uses_formulas() {
        let r = result(
            vec![
                term(1, "Zn", &[("Zn", 1)]),
                term(1, "CuSO₄", &[("Cu", 1), ("S", 1), ("O", 4)]),
            ],
            vec![
                term(1, "ZnSO₄", &[("Zn", 1), ("S", 1), ("O", 4)]),
                term(1, "Cu", &[("Cu", 1)]),
            ],
            true,
        );
        let a = analyze_redox(&r);
        assert!(a
            .narrative
            .iter()
            .any(|s| s.contains("Zn is oxidised") && s.contains("from 0 in Zn to +2 in ZnSO₄")));
        assert!(a
            .narrative
            .iter()
            .any(|s| s.contains("CuSO₄ is the oxidising agent")));
    }

    #[test]
    fn conflicting_same_side_states_go_to_indeterminate() {
        // O appears at 0 (in O₂) and −2 (in H₂O) on the reactant side → ambiguous, must be flagged.
        let r = result(
            vec![
                term(1, "O₂", &[("O", 2)]),
                term(1, "H₂O", &[("H", 2), ("O", 1)]),
            ],
            vec![term(1, "H₂O", &[("H", 2), ("O", 1)])],
            true,
        );
        let a = analyze_redox(&r);
        assert!(!a.changes.iter().any(|c| c.symbol == "O")); // O skipped, not in changes
        assert!(a.indeterminate.contains(&"O₂".to_string()));
        assert!(a.indeterminate.contains(&"H₂O".to_string()));
        let unique: std::collections::BTreeSet<&String> = a.indeterminate.iter().collect();
        assert_eq!(a.indeterminate.len(), unique.len()); // de-duplicated
    }
}
