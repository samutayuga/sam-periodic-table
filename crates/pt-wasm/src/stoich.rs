use pt_domain::{
    solve_stoichiometry, LimitingSide, QuantityUnit, ReactantEntry, ReactantSpec, POLYATOMIC_IONS,
};
use pt_services::PeriodicTable as ServiceTable;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

/// One reactant in a stoichiometry query: an element symbol, its subscript in the
/// product formula, and an optional entered amount (defaults to mole basis).
#[derive(Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
pub struct WasmReactantInput {
    pub symbol: String,
    pub subscript: i32,
    /// Entered amount; `None`/absent treats the reactant as "enough".
    pub amount: Option<f64>,
    /// "mass" for grams, anything else (or absent) for moles.
    pub unit: Option<String>,
}

/// Result of solving a binary synthesis: balanced coefficients, the limiting
/// reactant, theoretical yield and leftover excess, plus any diatomic notices.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmStoichResult {
    pub coeff_a: i32,
    pub coeff_b: i32,
    pub coeff_product: i32,
    pub product_molar_mass: f64,
    /// "A", "B", or "Both".
    pub limiting: String,
    pub yield_moles: f64,
    pub yield_mass: f64,
    pub excess_moles: f64,
    pub excess_mass: f64,
    pub diatomic_messages: Vec<String>,
}

/// A common polyatomic ion exposed to TypeScript.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmPolyatomicIon {
    pub symbol: String,
    pub name: String,
    pub charge: i32,
    pub formula: String,
}

fn entry_of(input: &WasmReactantInput) -> Option<ReactantEntry> {
    input.amount.map(|value| ReactantEntry {
        value,
        unit: if input.unit.as_deref() == Some("mass") {
            QuantityUnit::Mass
        } else {
            QuantityUnit::Mole
        },
    })
}

fn spec_of(table: &ServiceTable, input: &WasmReactantInput) -> Option<ReactantSpec> {
    let view = table.by_symbol(&input.symbol)?;
    Some(ReactantSpec {
        symbol: input.symbol.clone(),
        atomic_mass: view.element().atomic_mass,
        subscript_in_product: input.subscript as i64,
        // Elements are treated as monoatomic reactants — no natural-diatomic (X₂)
        // assumption — so e.g. Li + Cl → LiCl rather than 2Li + Cl₂ → 2LiCl.
        is_diatomic: false,
        entry: entry_of(input),
    })
}

fn limiting_label(side: LimitingSide) -> &'static str {
    match side {
        LimitingSide::A => "A",
        LimitingSide::B => "B",
        LimitingSide::Both => "Both",
    }
}

/// Solve a binary synthesis from two element inputs. Atomic masses and diatomic
/// status are looked up from the bundled table; `None` if either symbol is unknown.
pub(crate) fn solve(
    table: &ServiceTable,
    a: &WasmReactantInput,
    b: &WasmReactantInput,
) -> Option<WasmStoichResult> {
    let spec_a = spec_of(table, a)?;
    let spec_b = spec_of(table, b)?;
    let r = solve_stoichiometry(&spec_a, &spec_b);
    Some(WasmStoichResult {
        coeff_a: r.equation.coeff_a as i32,
        coeff_b: r.equation.coeff_b as i32,
        coeff_product: r.equation.coeff_product as i32,
        product_molar_mass: r.product_molar_mass,
        limiting: limiting_label(r.limiting).to_string(),
        yield_moles: r.product_yield.moles,
        yield_mass: r.product_yield.mass,
        excess_moles: r.excess.moles,
        excess_mass: r.excess.mass,
        diatomic_messages: r.diatomic_messages,
    })
}

/// The six common polyatomic ions.
pub(crate) fn polyatomic_ions() -> Vec<WasmPolyatomicIon> {
    POLYATOMIC_IONS
        .iter()
        .map(|ion| WasmPolyatomicIon {
            symbol: ion.symbol.to_string(),
            name: ion.name.to_string(),
            charge: ion.charge as i32,
            formula: ion.formula.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> ServiceTable {
        ServiceTable::load_bundled().unwrap()
    }

    fn input(
        symbol: &str,
        subscript: i32,
        amount: Option<f64>,
        unit: Option<&str>,
    ) -> WasmReactantInput {
        WasmReactantInput {
            symbol: symbol.into(),
            subscript,
            amount,
            unit: unit.map(str::to_string),
        }
    }

    #[test]
    fn water_stoichiometric_yield() {
        // Monoatomic reactants (no X₂ assumption): 2H + O -> H₂O.
        let h = input("H", 2, Some(2.0), Some("mole"));
        let o = input("O", 1, Some(1.0), Some("mole"));
        let r = solve(&table(), &h, &o).unwrap();
        assert_eq!((r.coeff_a, r.coeff_b, r.coeff_product), (2, 1, 1));
        assert_eq!(r.limiting, "Both");
        assert!((r.yield_moles - 1.0).abs() < 1e-9);
        // H₂O from real atomic masses ~ 18.0 g/mol.
        assert!((r.product_molar_mass - 18.0).abs() < 0.1);
        // No diatomic assumption -> no diatomic notices.
        assert!(r.diatomic_messages.is_empty());
    }

    #[test]
    fn excess_oxygen_from_mass_input() {
        // Monoatomic O: 32 g O = 2 mol O, paired with 3 mol H for H₂O (2H:1O).
        // H limiting (3/2 = 1.5 product), O in excess (2 - 1.5 = 0.5 mol).
        let h = input("H", 2, Some(3.0), Some("mole"));
        let o = input("O", 1, Some(32.0), Some("mass"));
        let r = solve(&table(), &h, &o).unwrap();
        assert_eq!(r.limiting, "A");
        assert!(r.excess_moles > 0.0);
    }

    #[test]
    fn no_diatomic_assumption_for_chlorine() {
        // Cl is not assumed to exist as Cl₂: Li + Cl -> LiCl, no notices.
        let na = input("Na", 1, Some(1.0), None);
        let cl = input("Cl", 1, Some(1.0), None);
        let r = solve(&table(), &na, &cl).unwrap();
        assert_eq!((r.coeff_a, r.coeff_b, r.coeff_product), (1, 1, 1));
        assert!(r.diatomic_messages.is_empty());
    }

    #[test]
    fn unknown_symbol_is_none() {
        let a = input("XX", 1, None, None);
        let b = input("O", 1, None, None);
        assert!(solve(&table(), &a, &b).is_none());
    }

    #[test]
    fn polyatomic_table_exposed() {
        let ions = polyatomic_ions();
        assert_eq!(ions.len(), 6);
        let sulfate = ions.iter().find(|i| i.symbol == "SO₄").unwrap();
        assert_eq!(sulfate.charge, -2);
    }
}
