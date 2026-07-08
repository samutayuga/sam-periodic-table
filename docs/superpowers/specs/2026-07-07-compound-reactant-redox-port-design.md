# Compound-Reactant Engine + Redox Analyzer Port (Rust / pt-wasm)

**Date:** 2026-07-07
**Status:** Approved design, ready for implementation planning
**Scope:** Port the iOS ChemCore compound-reactant engine + redox analyzer to Rust `pt-domain`, expose via `pt-wasm`, and make it consumable in `chem-interactive` (API only — no React UI).

## Summary

The iOS app (`ios-chem-interactive`, Swift ChemCore) gained a **compound-reactant reaction engine** (each reactant built from 1–2 species → classify → predict products → general balance → limiting reactant + yields) and a **redox analyzer** (oxidation states before/after → redox vs non-redox → oxidising/reducing agents → template narrative). ChemCore was originally ported *from* this Rust `pt-domain` crate; this brings the new logic back the other way.

`pt-domain` already holds the original binary-synthesis stoichiometry (`solve_stoichiometry`, `balance_equation`, `bonding`, `covalent`, `metallic`, `polyatomic`, `valence`, `math`). This port **adds** the compound-reactant engine + redox on top, exposes them through `pt-wasm`, and rebuilds the wasm package that `chem-interactive` consumes.

**Branches (both already feature branches — no new branch):** `sam-periodic-table` on `feat/reactant-compound`, `chem-interactive` on `feat/synch-ios`.

### Decisions locked during brainstorming

- **Integration depth:** expose the API only — pt-domain port + pt-wasm bindings + rebuilt pkg + a thin typed TS wrapper. No React Reaction Lab UI.
- **Tests:** port the ChemCore XCTest cases to Rust so the port is verified against the Swift behavior (the repo's fidelity ethos).
- **Redox naming:** `analyze_redox` renders formulas only (no `name` closure — a JS closure cannot cross the wasm boundary); TS renames if desired.
- **Fe/element + O₂ → Synthesis:** the corrected classification (combustion requires a C/H fuel) is included.

## Architecture

### 1. pt-domain (Rust) — new engine modules

Under `crates/pt-domain/src/engine/`, mirroring the ChemCore `Reaction/` module, pure and idiomatic Rust, each with **inline `#[cfg(test)] mod tests`** ported from the corresponding Swift XCTest cases:

```
engine/
  fraction.rs            // exact rational (reuses math::gcd); add/mul, reduced, positive denom
  formula_text.rs        // formula_subscript, crossover_subscripts, binary_formula
  species.rs             // Species value type
  reactant.rs            // Reactant + make_reactant(&[Species]) -> Reactant
  balancer.rs            // balance(reactants, products) -> Option<Vec<i64>> (Fraction null-space)
  reaction_class.rs      // ReactionClass + classify_reaction (Fe/element + O₂ -> Synthesis)
  activity_series.rs     // METAL_ACTIVITY_SERIES, HALOGEN_ACTIVITY_SERIES, displaces()
  product_prediction.rs  // Product, Prediction, predict_products
  reaction_solver.rs     // BalancedTerm, ReactionResult, ReactionError, solve_reaction
  oxidation_state.rs     // oxidation_state(&BTreeMap<String,i64>) -> Option<BTreeMap<String,i64>>
  redox.rs               // OxidationChange, ElementRedox, RedoxAnalysis, analyze_redox
```

Value types (Rust equivalents of the Swift structs):

```rust
pub struct Species {
    pub symbol: String,
    pub atomic_mass: f64,
    pub charge: Option<i8>,
    pub element_class: ElementClass,
    pub is_polyatomic: bool,
    pub valence_electrons: u8,
    pub group: u8,
    pub period: u8,
    pub composition: BTreeMap<String, i64>,  // element -> atom count
}

pub struct Reactant {
    pub species: Vec<Species>,               // 1 or 2
    pub formula: String,
    pub composition: BTreeMap<String, i64>,
    pub molar_mass: f64,
    pub cation: Option<Species>,
    pub anion: Option<Species>,
    pub is_bare_element: bool,
}

pub struct BalancedTerm { pub coeff: i64, pub formula: String, pub molar_mass: f64, pub composition: BTreeMap<String, i64> }

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
pub enum ReactionError { Unbalanceable, NoProducts, UnknownReactionClass, MissingAtomicMass(String) }

pub enum OxidationChange { Oxidised, Reduced, Unchanged }
pub struct ElementRedox { pub symbol: String, pub before: i64, pub after: i64, pub change: OxidationChange, pub reactant_formula: String, pub product_formula: String }
pub struct RedoxAnalysis {
    pub is_redox: bool,
    pub oxidising_agent: Option<String>,
    pub reducing_agent: Option<String>,
    pub changes: Vec<ElementRedox>,
    pub oxidation_states: BTreeMap<String, BTreeMap<String, i64>>,  // formula -> (element -> state)
    pub indeterminate: Vec<String>,
    pub narrative: Vec<String>,
}

pub fn solve_reaction(r1: &Reactant, r2: &Reactant, e1: Option<ReactantEntry>, e2: Option<ReactantEntry>,
                      atomic_mass: impl Fn(&str) -> Option<f64>) -> Result<ReactionResult, ReactionError>;
pub fn analyze_redox(result: &ReactionResult) -> RedoxAnalysis;
```

- Use `BTreeMap` for compositions/oxidation states (deterministic ordering, matching the Swift dictionaries' logical behavior; ordering-sensitive assertions use it).
- `atomic_mass` stays an injected closure in pt-domain (as in Swift); the wasm layer supplies it from the service table.
- `polyatomic.rs`: add `pub composition: &'static [(&'static str, u8)]` to each of the 6 `POLYATOMIC_IONS` (OH→[(O,1),(H,1)], SO₄→[(S,1),(O,4)], NO₃→[(N,1),(O,3)], CO₃→[(C,1),(O,3)], PO₄→[(P,1),(O,4)], NH₄→[(N,1),(H,4)]).
- Reuse existing `math::{gcd,lcm}`, `covalent_stoich`, `iupac_first`, `determine_bonding`, `NATURALLY_DIATOMIC`, `ElementClass`, `AmountResult`, `LimitingSide`, `ReactantEntry`, `QuantityUnit`.
- `lib.rs` re-exports the new public items alongside the existing ones.

Algorithms are a faithful translation of the reviewed, tested Swift:
- **Oxidation state:** free element → 0; factor a known polyatomic ion (disjoint-remainder guard) → counter-ion charge + central-atom solve; else element rules (F/O/H/halogen/group1/group2) + single-unknown solve-by-difference; ≥2 unknowns → `None`. Documented limitation: same-element/two-environment compounds (e.g. NH₄NO₃) average incorrectly.
- **Balancer:** element × species matrix over `Fraction`, one free column, scale to smallest positive integers, reject non-positive/degenerate.
- **Product prediction:** double displacement (anion swap + H⁺+OH⁻→H₂O, H⁺+CO₃→CO₂+H₂O), single displacement (activity-series feasibility, diatomic freed elements), combustion (hydrocarbon → CO₂+H₂O; bare-element oxide via crossover), synthesis.
- **Redox:** per-element before/after over reactant/product terms; agents; template narrative; ambiguous same-side → `indeterminate`.

### 2. pt-wasm — bindings

Extend `crates/pt-wasm` (new `compound_reaction.rs`, `mod`-ed in `lib.rs`). The wasm layer resolves all element data from `pt_services::PeriodicTable` by symbol; JS passes only symbols + charges + quantities.

Input (Tsify `from_wasm_abi`):
```rust
struct WasmSpecies { symbol: String, is_polyatomic: bool, charge: Option<i8> }
struct WasmQuantity { value: f64, unit: String }   // "mole" | "mass"
```

Entry point:
```rust
#[wasm_bindgen]
pub fn solve_compound_reaction(
    zone1: Vec<WasmSpecies>, zone2: Vec<WasmSpecies>,
    q1: Option<WasmQuantity>, q2: Option<WasmQuantity>,
) -> WasmReactionResult
```

Internally: resolve each `WasmSpecies` to a `pt_domain::Species` (atomic mass, group, period, valence, element_class from the table; polyatomic mass = Σ constituent masses, composition from the `POLYATOMIC_IONS` table); apply the **pair-aware charge rule** (mirrors the Swift `SpeciesMapping`: polyatomic/metal+nonmetal/acid[H+halogen] → ionic charges; other nonmetal+nonmetal → covalent nil; single species → carry its ionic charge) — this rule lives in the wasm resolver because it decides ionic-vs-covalent intent from placed species; `make_reactant` each zone; `solve_reaction` with an atomic-mass closure over the table; `analyze_redox` on a feasible result.

Output (Tsify `into_wasm_abi`):
```rust
struct WasmTerm { coeff: i32, formula: String, molar_mass: f64, composition: Vec<(String, i32)> }
struct WasmElementRedox { symbol: String, before: i32, after: i32, change: String, reactant_formula: String, product_formula: String }
struct WasmRedox { is_redox: bool, oxidising_agent: Option<String>, reducing_agent: Option<String>,
                   changes: Vec<WasmElementRedox>, narrative: Vec<String> }
struct WasmReactionResult {
    feasible: bool,
    reaction_class: String,
    reactants: Vec<WasmTerm>,
    products: Vec<WasmTerm>,
    limiting: String,                 // "A" | "B" | "Both"
    yields: Vec<(f64, f64)>,          // (moles, mass) per product
    excess: (f64, f64),
    messages: Vec<String>,
    error: Option<String>,            // set for unknown-class / unbalanceable etc.
    redox: Option<WasmRedox>,
}
```
Infeasible / unknown-class / unbalanceable surface as `feasible=false` and/or `error`, never a JS throw. The existing `WasmReaction`/`WasmCovalentStoich` binary bindings are untouched.

### 3. chem-interactive — consume (API only)

1. Build the wasm in `sam-periodic-table` with the same target/flags that produced the current `src/wasm/pkg` (verify from the existing `pt_wasm.js` header — `web`/bundler target).
2. Copy the regenerated `pkg/{pt_wasm.js, pt_wasm_bg.wasm, pt_wasm_bg.js, pt_wasm.d.ts}` into `chem-interactive/src/wasm/pkg/`. The new export + Tsify types appear in the generated `.d.ts` automatically.
3. Add a thin typed wrapper (e.g. `src/wasm/reaction.ts`) exposing `solveCompoundReaction(zone1, zone2, q1?, q2?): WasmReactionResult`. No React UI.
4. Smoke check: a small node/vitest test that the wasm loads and `solveCompoundReaction` returns the expected `NaOH + HCl` result.

Additive throughout; existing bindings/usage unchanged.

## Testing

- **pt-domain:** inline `#[cfg(test)]` per module, ported from the ChemCore tests — oxidation states (NaCl, MgO, CO₂, KMnO₄→Mn+7, FeCl₃→Fe+3, NaOH, Na₂SO₄→S+6, CuSO₄→Cu+2, indeterminate→None), Fraction, balancer (water/neutralisation/combustion/carbonate/unbalanceable), classifier (double/single/combustion/synthesis + Fe+O₂→synthesis), activity series (Zn>Cu, halogens), product prediction (diatomic freed Br₂/H₂, crossover MgO/Al₂O₃, neutralisation/carbonate specials, single-displacement feasible/infeasible), solver (limiting/yields/excess golden cases), redox (synthesis/single-displacement/combustion/non-redox/infeasible/indeterminate). `cargo test -p pt-domain`.
- **pt-wasm:** `#[wasm_bindgen_test]` end-to-end golden cases via `solve_compound_reaction` — NaOH+HCl, 2HCl+Na₂CO₃, Zn+CuSO₄ (feasible) + Cu+ZnSO₄ (infeasible), CH₄+O₂ (combustion), Fe+O₂ (synthesis), a not-classified pair. `wasm-pack test --node crates/pt-wasm`.
- **Full workspace:** `make test` (native + wasm) green; `make coverage` unaffected.
- **chem-interactive:** the smoke check; existing tests unaffected.

Golden results must match the Swift/iOS values.

## Out of scope

- React Reaction Lab UI (compound zones, poster, tour) in chem-interactive.
- Reaction classes / compounds beyond those the engine produces.
- Redox oxidation-state exceptions (peroxides, hydrides) — unreachable, as in Swift.
- Naming the substances in the redox narrative (formulas only in Rust; TS may rename).
