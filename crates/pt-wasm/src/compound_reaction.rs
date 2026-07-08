//! `solve_compound_reaction` binding: resolves JS-supplied species (elements or
//! polyatomic ions) into `pt_domain::Species`, applies the pair-aware ionic/
//! covalent charge rule (ported from the iOS ChemCore `SpeciesMapping.swift`),
//! then runs `make_reactant` + `solve_reaction` + `analyze_redox`.
//!
//! `Vec<T>` of a custom Tsify struct cannot be used directly as a
//! `#[wasm_bindgen]` parameter (wasm-bindgen's vector ABI requires
//! `T: ErasableGeneric`, which Tsify-derived structs don't implement) — so the
//! zones are accepted as `JsValue` with an `unchecked_param_type` override for
//! the generated `.d.ts`, and deserialised by hand via `serde_wasm_bindgen`.

use std::collections::BTreeMap;

use pt_domain::{
    analyze_redox, is_transition_metal, make_reactant, parse_valence_electrons, solve_reaction,
    ElementClass, LimitingSide, QuantityUnit, Reactant, ReactantEntry, ReactionError,
    RedoxAnalysis, Species, POLYATOMIC_IONS,
};
use pt_services::PeriodicTable as ServiceTable;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

/// One placed species: a bare element or a polyatomic ion, identified by
/// symbol. `charge` is the caller's explicit ionic-charge pick — required for
/// transition metals (multiple common oxidation states), ignored otherwise.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub struct WasmSpecies {
    pub symbol: String,
    pub is_polyatomic: bool,
    pub charge: Option<i8>,
}

/// An entered reactant amount: a value plus its unit ("mole" or "mass";
/// anything else defaults to moles).
#[derive(Debug, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
pub struct WasmQuantity {
    pub value: f64,
    pub unit: String,
}

/// One balanced reactant or product term.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmTerm {
    pub coeff: i32,
    pub formula: String,
    pub molar_mass: f64,
    pub composition: Vec<(String, i32)>,
}

/// One element's oxidation-state change from a reactant term to a product term.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmElementRedox {
    pub symbol: String,
    pub before: i32,
    pub after: i32,
    /// "Oxidised" or "Reduced".
    pub change: String,
    pub reactant_formula: String,
    pub product_formula: String,
}

/// Redox analysis of a solved reaction.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmRedox {
    pub is_redox: bool,
    pub oxidising_agent: Option<String>,
    pub reducing_agent: Option<String>,
    pub changes: Vec<WasmElementRedox>,
    pub narrative: Vec<String>,
}

/// The full result of solving a compound reaction end to end. Infeasible /
/// unknown-class / unbalanceable / unresolvable-species outcomes surface as
/// `feasible=false` and/or `error`, never a JS throw.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmReactionResult {
    pub feasible: bool,
    /// "Synthesis", "DoubleDisplacement", "SingleDisplacement", "Combustion", or "None".
    pub reaction_class: String,
    pub reactants: Vec<WasmTerm>,
    pub products: Vec<WasmTerm>,
    /// "A", "B", or "Both".
    pub limiting: String,
    pub yields: Vec<(f64, f64)>,
    pub excess: (f64, f64),
    pub messages: Vec<String>,
    pub error: Option<String>,
    pub redox: Option<WasmRedox>,
}

/// A `WasmSpecies` resolved against the periodic table (or the polyatomic-ion
/// table): everything needed to compute its pair-aware ionic charge and build
/// a `pt_domain::Species`.
struct Resolved {
    symbol: String,
    atomic_mass: f64,
    element_class: ElementClass,
    is_polyatomic: bool,
    valence_electrons: u8,
    group: u8,
    period: u8,
    composition: BTreeMap<String, i64>,
    is_transition: bool,
    oxidation_states: Vec<i8>,
    /// The polyatomic ion's fixed charge; meaningless when `!is_polyatomic`.
    ion_charge: i8,
    /// The caller-supplied charge pick (`WasmSpecies::charge`).
    derived_charge: Option<i8>,
}

fn resolve(table: &ServiceTable, sp: &WasmSpecies) -> Option<Resolved> {
    if sp.is_polyatomic {
        let ion = POLYATOMIC_IONS.iter().find(|i| i.symbol == sp.symbol)?;
        let mut composition = BTreeMap::new();
        let mut mass = 0.0;
        for &(elem_symbol, count) in ion.composition {
            let view = table.by_symbol(elem_symbol)?;
            mass += view.element().atomic_mass * count as f64;
            composition.insert(elem_symbol.to_string(), count as i64);
        }
        Some(Resolved {
            symbol: sp.symbol.clone(),
            atomic_mass: mass,
            element_class: ElementClass::NonMetal,
            is_polyatomic: true,
            valence_electrons: 0,
            group: 0,
            period: 0,
            composition,
            is_transition: false,
            oxidation_states: vec![ion.charge],
            ion_charge: ion.charge,
            derived_charge: sp.charge,
        })
    } else {
        let view = table.by_symbol(&sp.symbol)?;
        let group = view.group();
        let mut composition = BTreeMap::new();
        composition.insert(sp.symbol.clone(), 1);
        Some(Resolved {
            symbol: sp.symbol.clone(),
            atomic_mass: view.element().atomic_mass,
            element_class: view.element_class(),
            is_polyatomic: false,
            valence_electrons: parse_valence_electrons(
                &view.electron_configuration().to_string(),
                group,
            ),
            group,
            period: view.period(),
            composition,
            is_transition: is_transition_metal(group),
            oxidation_states: view.oxidation_states().0,
            ion_charge: 0,
            derived_charge: sp.charge,
        })
    }
}

/// The charge a species carries when its zone is ionic (or a bare element
/// whose charge a later product crossover needs). Ports Swift `ionicCharge`.
fn ionic_charge(r: &Resolved) -> Option<i8> {
    if r.is_polyatomic {
        return Some(r.ion_charge);
    }
    if r.is_transition {
        return r.derived_charge;
    }
    if r.symbol == "H" {
        return Some(1);
    }
    if r.element_class == ElementClass::Metal {
        return r
            .derived_charge
            .or_else(|| r.oxidation_states.iter().find(|&&c| c > 0).copied());
    }
    r.oxidation_states.iter().find(|&&c| c < 0).copied()
}

/// Ports Swift `isAcidPair`.
fn is_acid_pair(a: &Resolved, b: &Resolved) -> bool {
    (a.symbol == "H" && b.group == 17) || (b.symbol == "H" && a.group == 17)
}

/// Ports Swift `isIonicPair`.
fn is_ionic_pair(a: &Resolved, b: &Resolved) -> bool {
    if a.is_polyatomic || b.is_polyatomic {
        return true;
    }
    let metals = [a, b]
        .iter()
        .filter(|s| s.element_class == ElementClass::Metal)
        .count();
    if metals == 1 {
        return true;
    }
    if metals == 2 {
        return false;
    }
    is_acid_pair(a, b)
}

/// Ports Swift `buildReactant`: resolve 1-2 zone species, apply the
/// pair-aware charge rule, then build the `Reactant`.
fn build_reactant(table: &ServiceTable, zone: &[WasmSpecies]) -> Result<Reactant, String> {
    if zone.is_empty() || zone.len() > 2 {
        return Err("a reactant zone must have 1 or 2 species".to_string());
    }

    let resolved: Vec<Resolved> = zone
        .iter()
        .map(|z| resolve(table, z).ok_or_else(|| format!("unknown species: {}", z.symbol)))
        .collect::<Result<_, _>>()?;

    for r in &resolved {
        if r.is_transition && r.derived_charge.is_none() {
            return Err(format!(
                "{} is a transition metal and requires an explicit charge",
                r.symbol
            ));
        }
    }

    let charges: Vec<Option<i8>> = if resolved.len() == 1 {
        vec![ionic_charge(&resolved[0])]
    } else if is_ionic_pair(&resolved[0], &resolved[1]) {
        vec![ionic_charge(&resolved[0]), ionic_charge(&resolved[1])]
    } else {
        vec![None, None]
    };

    let species: Vec<Species> = resolved
        .into_iter()
        .zip(charges)
        .map(|(r, charge)| {
            Species::new(
                r.symbol,
                r.atomic_mass,
                charge,
                r.element_class,
                r.is_polyatomic,
                r.valence_electrons,
                r.group,
                r.period,
                r.composition,
            )
        })
        .collect();

    Ok(make_reactant(&species))
}

fn quantity_entry(q: Option<&WasmQuantity>) -> Option<ReactantEntry> {
    q.map(|q| ReactantEntry {
        value: q.value,
        unit: if q.unit == "mass" {
            QuantityUnit::Mass
        } else {
            QuantityUnit::Mole
        },
    })
}

fn to_wasm_term(t: &pt_domain::BalancedTerm) -> WasmTerm {
    WasmTerm {
        coeff: t.coeff as i32,
        formula: t.formula.clone(),
        molar_mass: t.molar_mass,
        composition: t
            .composition
            .iter()
            .map(|(k, v)| (k.clone(), *v as i32))
            .collect(),
    }
}

fn limiting_label(side: LimitingSide) -> &'static str {
    match side {
        LimitingSide::A => "A",
        LimitingSide::B => "B",
        LimitingSide::Both => "Both",
    }
}

fn to_wasm_redox(a: &RedoxAnalysis) -> WasmRedox {
    WasmRedox {
        is_redox: a.is_redox,
        oxidising_agent: a.oxidising_agent.clone(),
        reducing_agent: a.reducing_agent.clone(),
        changes: a
            .changes
            .iter()
            .map(|c| WasmElementRedox {
                symbol: c.symbol.clone(),
                before: c.before as i32,
                after: c.after as i32,
                change: format!("{:?}", c.change),
                reactant_formula: c.reactant_formula.clone(),
                product_formula: c.product_formula.clone(),
            })
            .collect(),
        narrative: a.narrative.clone(),
    }
}

fn reaction_error_message(err: &ReactionError) -> String {
    match err {
        ReactionError::Unbalanceable => "the reaction could not be balanced".to_string(),
        ReactionError::NoProducts => "no products were predicted".to_string(),
        ReactionError::UnknownReactionClass => {
            "the reactant pair does not match a known reaction class".to_string()
        }
        ReactionError::MissingAtomicMass(sym) => format!("missing atomic mass for {sym}"),
    }
}

fn error_result(msg: &str) -> WasmReactionResult {
    WasmReactionResult {
        feasible: false,
        reaction_class: "None".to_string(),
        reactants: Vec::new(),
        products: Vec::new(),
        limiting: "Both".to_string(),
        yields: Vec::new(),
        excess: (0.0, 0.0),
        messages: Vec::new(),
        error: Some(msg.to_string()),
        redox: None,
    }
}

/// Solve a compound reaction between `zone1` and `zone2` (each 1-2 species)
/// end to end: resolve species, apply the pair-aware charge rule, classify,
/// predict products, balance, resolve limiting-reactant amounts, and analyse
/// redox. Never throws — resolution/classification/balancing failures surface
/// as `feasible=false` with `error` set.
pub(crate) fn solve(
    table: &ServiceTable,
    zone1: &[WasmSpecies],
    zone2: &[WasmSpecies],
    q1: Option<&WasmQuantity>,
    q2: Option<&WasmQuantity>,
) -> WasmReactionResult {
    let r1 = match build_reactant(table, zone1) {
        Ok(r) => r,
        Err(e) => return error_result(&e),
    };
    let r2 = match build_reactant(table, zone2) {
        Ok(r) => r,
        Err(e) => return error_result(&e),
    };

    let entry1 = quantity_entry(q1);
    let entry2 = quantity_entry(q2);
    let atomic_mass = |sym: &str| table.by_symbol(sym).map(|v| v.element().atomic_mass);

    match solve_reaction(&r1, &r2, entry1, entry2, atomic_mass) {
        Ok(result) => {
            let redox = result
                .feasible
                .then(|| to_wasm_redox(&analyze_redox(&result)));
            WasmReactionResult {
                feasible: result.feasible,
                reaction_class: format!("{:?}", result.reaction_class),
                reactants: result.reactants.iter().map(to_wasm_term).collect(),
                products: result.products.iter().map(to_wasm_term).collect(),
                limiting: limiting_label(result.limiting).to_string(),
                yields: result.yields.iter().map(|y| (y.moles, y.mass)).collect(),
                excess: (result.excess.moles, result.excess.mass),
                messages: result.messages.clone(),
                error: None,
                redox,
            }
        }
        Err(err) => error_result(&reaction_error_message(&err)),
    }
}

/// Solve a compound reaction between two reactant zones (each 1-2 species —
/// bare elements or polyatomic ions, identified by symbol). Loads the bundled
/// periodic table on every call.
///
/// `zone1`/`zone2` are accepted as `JsValue` (typed `WasmSpecies[]` in the
/// generated `.d.ts`) because wasm-bindgen cannot bind `Vec<CustomStruct>`
/// directly as a function parameter.
#[wasm_bindgen]
pub fn solve_compound_reaction(
    #[wasm_bindgen(unchecked_param_type = "WasmSpecies[]")] zone1: JsValue,
    #[wasm_bindgen(unchecked_param_type = "WasmSpecies[]")] zone2: JsValue,
    q1: Option<WasmQuantity>,
    q2: Option<WasmQuantity>,
) -> WasmReactionResult {
    let zone1: Vec<WasmSpecies> = match serde_wasm_bindgen::from_value(zone1) {
        Ok(v) => v,
        Err(e) => return error_result(&format!("invalid zone1: {e}")),
    };
    let zone2: Vec<WasmSpecies> = match serde_wasm_bindgen::from_value(zone2) {
        Ok(v) => v,
        Err(e) => return error_result(&format!("invalid zone2: {e}")),
    };

    let table = match ServiceTable::load_bundled() {
        Ok(t) => t,
        Err(e) => return error_result(&e.to_string()),
    };

    solve(&table, &zone1, &zone2, q1.as_ref(), q2.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    fn table() -> ServiceTable {
        ServiceTable::load_bundled().unwrap()
    }

    fn element(symbol: &str) -> WasmSpecies {
        WasmSpecies {
            symbol: symbol.to_string(),
            is_polyatomic: false,
            charge: None,
        }
    }

    fn element_charged(symbol: &str, charge: i8) -> WasmSpecies {
        WasmSpecies {
            symbol: symbol.to_string(),
            is_polyatomic: false,
            charge: Some(charge),
        }
    }

    fn ion(symbol: &str) -> WasmSpecies {
        WasmSpecies {
            symbol: symbol.to_string(),
            is_polyatomic: true,
            charge: None,
        }
    }

    #[wasm_bindgen_test]
    fn naoh_plus_hcl_is_non_redox_double_displacement() {
        let t = table();
        let naoh = vec![element("Na"), ion("OH")];
        let hcl = vec![element("H"), element("Cl")];
        let r = solve(&t, &naoh, &hcl, None, None);
        assert!(r.feasible, "expected feasible: {:?}", r.error);
        assert_eq!(r.reaction_class, "DoubleDisplacement");
        let mut formulas: Vec<&str> = r.products.iter().map(|p| p.formula.as_str()).collect();
        formulas.sort();
        assert_eq!(formulas, vec!["H₂O", "NaCl"]);
        let redox = r.redox.expect("redox present for feasible reaction");
        assert!(!redox.is_redox);
    }

    #[wasm_bindgen_test]
    fn hcl_plus_na2co3_gives_three_products() {
        let t = table();
        let hcl = vec![element("H"), element("Cl")];
        let na2co3 = vec![element("Na"), ion("CO₃")];
        let r = solve(&t, &hcl, &na2co3, None, None);
        assert!(r.feasible, "expected feasible: {:?}", r.error);
        assert_eq!(r.products.len(), 3);
        let mut formulas: Vec<&str> = r.products.iter().map(|p| p.formula.as_str()).collect();
        formulas.sort();
        assert_eq!(formulas, vec!["CO₂", "H₂O", "NaCl"]);
    }

    #[wasm_bindgen_test]
    fn zn_plus_cuso4_is_redox_single_displacement() {
        let t = table();
        // Zn (group 12) and Cu (group 11) are transition metals -> explicit charge picks required.
        let zn = vec![element_charged("Zn", 2)];
        let cuso4 = vec![element_charged("Cu", 2), ion("SO₄")];
        let r = solve(&t, &zn, &cuso4, None, None);
        assert!(r.feasible, "expected feasible: {:?}", r.error);
        assert_eq!(r.reaction_class, "SingleDisplacement");
        let redox = r.redox.expect("redox present for feasible reaction");
        assert!(redox.is_redox);
        assert_eq!(redox.reducing_agent.as_deref(), Some("Zn"));
        assert_eq!(redox.oxidising_agent.as_deref(), Some("CuSO₄"));
    }

    #[wasm_bindgen_test]
    fn cu_plus_znso4_is_infeasible() {
        let t = table();
        let cu = vec![element_charged("Cu", 2)];
        let znso4 = vec![element_charged("Zn", 2), ion("SO₄")];
        let r = solve(&t, &cu, &znso4, None, None);
        assert!(!r.feasible);
        assert!(r.redox.is_none());
        assert!(r.messages.iter().any(|m| m.contains("activity series")));
    }

    #[wasm_bindgen_test]
    fn ch4_plus_o2_is_combustion() {
        let t = table();
        let ch4 = vec![element("C"), element("H")];
        let o2 = vec![element("O")];
        let r = solve(&t, &ch4, &o2, None, None);
        assert!(r.feasible, "expected feasible: {:?}", r.error);
        assert_eq!(r.reaction_class, "Combustion");
        let mut formulas: Vec<&str> = r.products.iter().map(|p| p.formula.as_str()).collect();
        formulas.sort();
        assert_eq!(formulas, vec!["CO₂", "H₂O"]);
    }

    #[wasm_bindgen_test]
    fn fe_plus_o2_is_synthesis_not_combustion() {
        let t = table();
        // Fe (group 8) is a transition metal -> explicit charge pick required.
        let fe = vec![element_charged("Fe", 2)];
        let o2 = vec![element("O")];
        let r = solve(&t, &fe, &o2, None, None);
        assert!(r.feasible, "expected feasible: {:?}", r.error);
        assert_eq!(r.reaction_class, "Synthesis");
    }

    #[wasm_bindgen_test]
    fn unclassified_pair_is_infeasible_with_error() {
        let t = table();
        // A lone polyatomic ion (not bare, not a cation/anion pair) paired with
        // a bare element matches none of the four reaction classes.
        let oh = vec![ion("OH")];
        let cl = vec![element("Cl")];
        let r = solve(&t, &oh, &cl, None, None);
        assert!(!r.feasible);
        assert_eq!(r.reaction_class, "None");
        assert!(r.error.is_some());
        assert!(r.redox.is_none());
    }

    #[wasm_bindgen_test]
    fn transition_metal_without_charge_is_infeasible_with_error() {
        let t = table();
        let zn = vec![element("Zn")]; // no charge supplied
        let o2 = vec![element("O")];
        let r = solve(&t, &zn, &o2, None, None);
        assert!(!r.feasible);
        assert!(r.error.is_some());
    }

    #[wasm_bindgen_test]
    fn end_to_end_via_public_entry_point() {
        let zone1 = serde_wasm_bindgen::to_value(&vec![element("Na"), ion("OH")]).unwrap();
        let zone2 = serde_wasm_bindgen::to_value(&vec![element("H"), element("Cl")]).unwrap();
        let r = solve_compound_reaction(zone1, zone2, None, None);
        assert!(r.feasible, "expected feasible: {:?}", r.error);
        assert_eq!(r.reaction_class, "DoubleDisplacement");
    }
}
