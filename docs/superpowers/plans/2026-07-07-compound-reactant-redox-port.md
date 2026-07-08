# Compound-Reactant Engine + Redox Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the iOS ChemCore compound-reactant engine + redox analyzer to Rust `pt-domain`, expose via `pt-wasm`, and consume as a typed API in `chem-interactive`.

**Architecture:** This is a faithful translation of already-tested Swift. Each pt-domain task ports ONE Swift file (algorithm) + its XCTest file (test cases) to one idiomatic Rust module with inline `#[cfg(test)]` tests. Then pt-wasm adds a `solve_compound_reaction` binding that resolves element data from the service table, and chem-interactive rebuilds + consumes the wasm.

**Tech Stack:** Rust (workspace crates pt-domain/pt-wasm), wasm-bindgen + serde + tsify, wasm-pack, TypeScript/Vite (chem-interactive).

## Global Constraints

- **This is a PORT.** The authoritative algorithm + test values are the Swift files under `/Users/putumas.mertayasa.e/Developer/tools/ios-chem-interactive/ChemCore/`. Each task names the exact Swift source + test file to READ and translate. Rust results MUST equal the Swift results (same golden values).
- **Repos / branches (already feature branches — do NOT create new ones):** `sam-periodic-table` @ `feat/reactant-compound`; `chem-interactive` @ `feat/synch-ios`.
- **Additive only.** Do not modify existing pt-domain/pt-wasm logic except: `polyatomic.rs` gains a `composition` field; `engine/mod.rs` + `lib.rs` gain `pub mod` / `pub use` lines for new modules. The existing binary stoichiometry + wasm bindings stay untouched.
- **Rust idioms:** tests are inline `#[cfg(test)] mod tests` in the same file (repo convention — see `polyatomic.rs`). Use `BTreeMap<String,i64>` for compositions/oxidation maps (deterministic). `Option`/`Result` for nullable/failing returns. Reuse `crate::{gcd, lcm, covalent_stoich, iupac_first, determine_bonding, NATURALLY_DIATOMIC, is_naturally_diatomic, ElementClass, AmountResult, LimitingSide, ReactantEntry, QuantityUnit}`.
- **Test commands:** per module `cargo test -p pt-domain <module>`; full native `cargo test --workspace --exclude pt-wasm`; wasm `wasm-pack test --node crates/pt-wasm`; everything `make test`. Run from `sam-periodic-table/`.
- **Formatting/lint:** run `cargo fmt` before each commit; `cargo clippy -p pt-domain` should be clean (no new warnings).
- **Commit convention:** end body with `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

Swift source dir: `SWIFT=/Users/putumas.mertayasa.e/Developer/tools/ios-chem-interactive/ChemCore/Sources/ChemCore` · tests: `STEST=/Users/putumas.mertayasa.e/Developer/tools/ios-chem-interactive/ChemCore/Tests/ChemCoreTests`.

---

## File Structure (new)

`crates/pt-domain/src/engine/`: `fraction.rs`, `formula_text.rs`, `species.rs`, `reactant.rs`, `balancer.rs`, `reaction_class.rs`, `activity_series.rs`, `product_prediction.rs`, `reaction_solver.rs`, `oxidation_state.rs`, `redox.rs`. Modified: `polyatomic.rs`, `engine/mod.rs`, `lib.rs`.
`crates/pt-wasm/src/`: `compound_reaction.rs` (+ `lib.rs` `mod`).
`chem-interactive/src/wasm/`: regenerated `pkg/*` + `reaction.ts`.

Each pt-domain task adds `pub mod <m>;` to `engine/mod.rs` and the new items to `lib.rs` `pub use engine::<m>::{...};`.

---

### Task 1: PolyatomicIon composition

**Files:** Modify `crates/pt-domain/src/engine/polyatomic.rs`
**Reference:** Swift `PolyatomicIon.composition` (in `$SWIFT/State/PolyatomicIon.swift` — the 6 maps).

**Produces:** `PolyatomicIon.composition: &'static [(&'static str, u8)]` on each of the 6 `POLYATOMIC_IONS`.

- [ ] **Step 1: Add a failing test** to the existing `mod tests` in `polyatomic.rs`:

```rust
    #[test]
    fn sulfate_composition() {
        let so4 = POLYATOMIC_IONS.iter().find(|i| i.symbol == "SO₄").unwrap();
        assert_eq!(so4.composition, &[("S", 1), ("O", 4)]);
    }
    #[test]
    fn hydroxide_composition() {
        let oh = POLYATOMIC_IONS.iter().find(|i| i.symbol == "OH").unwrap();
        assert_eq!(oh.composition, &[("O", 1), ("H", 1)]);
    }
    #[test]
    fn ammonium_composition() {
        let nh4 = POLYATOMIC_IONS.iter().find(|i| i.symbol == "NH₄").unwrap();
        assert_eq!(nh4.composition, &[("N", 1), ("H", 4)]);
    }
```

- [ ] **Step 2:** `cargo test -p pt-domain polyatomic` → FAIL (`no field composition`).
- [ ] **Step 3:** Add `pub composition: &'static [(&'static str, u8)],` to the struct and populate each entry: `OH→&[("O",1),("H",1)]`, `NO₃→&[("N",1),("O",3)]`, `SO₄→&[("S",1),("O",4)]`, `CO₃→&[("C",1),("O",3)]`, `PO₄→&[("P",1),("O",4)]`, `NH₄→&[("N",1),("H",4)]`.
- [ ] **Step 4:** `cargo test -p pt-domain polyatomic` → PASS. Then `cargo fmt`.
- [ ] **Step 5:** Commit `feat(pt-domain): add element composition to PolyatomicIon`.

---

### Task 2: Fraction

**Files:** Create `crates/pt-domain/src/engine/fraction.rs`; modify `engine/mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/Fraction.swift` + `$STEST/FractionTests.swift`.

**Consumes:** `crate::gcd`.
**Produces:** `pub struct Fraction { pub num: i64, pub den: i64 }` (always reduced, positive denominator) with `Fraction::new(num, den) -> Fraction`, `impl std::ops::{Add,Mul}`, `pub fn is_zero(&self) -> bool`. Derive `Clone, Copy, PartialEq, Eq, Debug`.

- [ ] **Step 1:** Read the two Swift files. Write `fraction.rs` translating `Fraction` (constructor reduces via `gcd`, moves sign to numerator; `+`/`*` via the constructor). Add inline `#[cfg(test)] mod tests` porting all 5 `FractionTests` cases (reduce 4/8→1/2, sign 1/-2→-1/2, 1/2+1/3=5/6, 2/3·3/4=1/2, is_zero). Add `pub mod fraction;` to `engine/mod.rs` and `pub use engine::fraction::Fraction;` to `lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain fraction` → run; it should PASS once written (write test first if you prefer strict TDD — either way tests must exist and pass).
- [ ] **Step 3:** `cargo fmt` + `cargo clippy -p pt-domain` clean.
- [ ] **Step 4:** Commit `feat(pt-domain): add exact Fraction for the reaction balancer`.

---

### Task 3: FormulaText

**Files:** Create `crates/pt-domain/src/engine/formula_text.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/FormulaText.swift` + `$STEST/FormulaTextTests.swift`.

**Consumes:** `crate::gcd`.
**Produces:** `pub fn formula_subscript(n: i64) -> String` (Unicode subscripts; `n<=1`→`""`); `pub fn crossover_subscripts(cation_charge: i64, anion_charge: i64) -> (i64 /*cation*/, i64 /*anion*/)`; `pub fn binary_formula(first: &str, first_count: i64, second: &str, second_count: i64, second_is_polyatomic: bool) -> String`.

- [ ] **Step 1:** Read Swift files; port the three functions (subscript digit map; gcd-reduced crossover using magnitudes; binary formula with parentheses when the polyatomic part repeats). Port all `FormulaTextTests` cases as inline tests. Wire `mod.rs` + `lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain formula_text` → PASS. `cargo fmt`.
- [ ] **Step 3:** Commit `feat(pt-domain): add formula-text + crossover helpers`.

---

### Task 4: Balancer

**Files:** Create `crates/pt-domain/src/engine/balancer.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/Balancer.swift` + `$STEST/BalancerTests.swift`.

**Consumes:** `Fraction` (Task 2), `crate::gcd`.
**Produces:** `pub fn balance(reactants: &[BTreeMap<String,i64>], products: &[BTreeMap<String,i64>]) -> Option<Vec<i64>>` — smallest positive integer coefficients, reactants then products, or `None` when unbalanceable/degenerate.

- [ ] **Step 1:** Read Swift; port the element×species matrix build (reactants +, products −), Gaussian elimination over `Fraction`, single-free-column guard, LCM-scale to integers, sign-normalise, divide by GCD, reject non-positive/degenerate. Port all `BalancerTests` cases (water→[2,1,2], neutralisation→[1,1,1,1], combustion methane→[1,2,1,2], carbonate→[2,1,2,1,1], unbalanceable→None). Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain balancer` → PASS. `cargo fmt` + clippy clean.
- [ ] **Step 3:** Commit `feat(pt-domain): add general integer reaction balancer`.

---

### Task 5: Species

**Files:** Create `crates/pt-domain/src/engine/species.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/Species.swift`.

**Consumes:** `crate::ElementClass`.
**Produces:**
```rust
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
    pub composition: std::collections::BTreeMap<String, i64>,
}
```
with a `Species::new(...)` constructor mirroring the Swift memberwise init.

- [ ] **Step 1:** Write `species.rs` with the struct + constructor. Add a trivial inline test constructing a `Species` and asserting its fields. Wire `mod.rs`+`lib.rs` (`pub use engine::species::Species;`).
- [ ] **Step 2:** `cargo test -p pt-domain species` → PASS. `cargo fmt`.
- [ ] **Step 3:** Commit `feat(pt-domain): add Species value type`.

---

### Task 6: Reactant + make_reactant

**Files:** Create `crates/pt-domain/src/engine/reactant.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/Reactant.swift` + `$STEST/ReactantTests.swift`.

**Consumes:** `Species` (Task 5), `formula_subscript`/`crossover_subscripts`/`binary_formula` (Task 3), `crate::{determine_bonding, covalent_stoich, iupac_first, is_naturally_diatomic}`.
**Produces:**
```rust
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
pub fn make_reactant(species: &[Species]) -> Reactant;
```

- [ ] **Step 1:** Read Swift; port `make_reactant`: 1 species → bare element (diatomic composition + "₂" when `is_naturally_diatomic` and non-polyatomic); 2 species → ionic (either polyatomic OR `determine_bonding == Ionic` OR the opposite-charge acid rule from Swift `Reactant.swift`) using cation/anion crossover, else covalent via `covalent_stoich` + `iupac_first`. Port the `ReactantTests` cases (bare Zn/O₂, NaCl, Na₂SO₄, CH₄ covalent, the opposite-charge HCl-ionic + methane-covalent regression tests). Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain reactant` → PASS. `cargo fmt` + clippy clean.
- [ ] **Step 3:** Commit `feat(pt-domain): add Reactant compound builder`.

---

### Task 7: ReactionClass + classifier

**Files:** Create `crates/pt-domain/src/engine/reaction_class.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/ReactionClass.swift` + `$STEST/ReactionClassTests.swift` (NOTE: use the CURRENT Swift `isFuel`, which requires C or H — bare metal + O₂ is Synthesis).

**Consumes:** `Reactant` (Task 6).
**Produces:** `pub enum ReactionClass { Synthesis, DoubleDisplacement, SingleDisplacement, Combustion, None }` (derive `Debug,Clone,Copy,PartialEq,Eq`); `pub fn classify_reaction(r1: &Reactant, r2: &Reactant) -> ReactionClass`.

- [ ] **Step 1:** Read Swift; port `classify_reaction` with priority combustion→single→double→synthesis→none, where combustion requires O₂ + a C/H fuel (bare metal/element + O₂ → Synthesis). Port `ReactionClassTests` cases INCLUDING the Fe+O₂→Synthesis regression test. Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain reaction_class` → PASS. `cargo fmt`.
- [ ] **Step 3:** Commit `feat(pt-domain): add reaction classifier (metal + O2 -> synthesis)`.

---

### Task 8: Activity series

**Files:** Create `crates/pt-domain/src/engine/activity_series.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/ActivitySeries.swift` + `$STEST/ActivitySeriesTests.swift`.

**Produces:** `pub const METAL_ACTIVITY_SERIES: &[&str]`, `pub const HALOGEN_ACTIVITY_SERIES: &[&str]`, `pub fn displaces(free: &str, bound: &str) -> Option<bool>` (true when `free` outranks `bound` in a shared series; `None` when no shared series; same element → `Some(false)`).

- [ ] **Step 1:** Read Swift; port the two ordered series verbatim + `displaces`. Port all `ActivitySeriesTests` cases. Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain activity_series` → PASS. `cargo fmt`.
- [ ] **Step 3:** Commit `feat(pt-domain): add metal + halogen activity series`.

---

### Task 9: Product prediction

**Files:** Create `crates/pt-domain/src/engine/product_prediction.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/ProductPrediction.swift` + `$STEST/ProductPredictionTests.swift`.

**Consumes:** `Reactant`/`Species` (T5/6), `crossover_subscripts`/`binary_formula`/`formula_subscript` (T3), `displaces` (T8), `crate::is_naturally_diatomic`.
**Produces:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Product { pub formula: String, pub composition: BTreeMap<String, i64> }
#[derive(Debug, Clone, PartialEq)]
pub enum Prediction { Products(Vec<Product>), Infeasible(String) }
pub fn predict_products(class: ReactionClass, r1: &Reactant, r2: &Reactant) -> Prediction;
```

- [ ] **Step 1:** Read Swift; port all branches — double displacement (anion swap, H⁺+OH⁻→H₂O, H⁺+CO₃→CO₂+H₂O), single displacement (metal + halogen paths, diatomic freed elements via `freedElement`, activity-series feasibility → `Infeasible`), combustion (hydrocarbon → CO₂/H₂O; bare-element oxide via `crossover_subscripts`), synthesis. Port all `ProductPredictionTests` cases (diatomic freed Br₂/H₂, crossover MgO/Al₂O₃, neutralisation/carbonate, single-displacement feasible+infeasible). Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain product_prediction` → PASS. `cargo fmt` + clippy clean.
- [ ] **Step 3:** Commit `feat(pt-domain): add product prediction for all reaction classes`.

---

### Task 10: Reaction solver

**Files:** Create `crates/pt-domain/src/engine/reaction_solver.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/ReactionSolver.swift` + `$STEST/ReactionSolverTests.swift`.

**Consumes:** `Reactant` (T6), `classify_reaction` (T7), `predict_products`/`Prediction`/`Product` (T9), `balance` (T4), `crate::{AmountResult, LimitingSide, ReactantEntry, QuantityUnit}`.
**Produces:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct BalancedTerm { pub coeff: i64, pub formula: String, pub molar_mass: f64, pub composition: BTreeMap<String, i64> }
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
#[derive(Debug, Clone, PartialEq)]
pub enum ReactionError { Unbalanceable, NoProducts, UnknownReactionClass, MissingAtomicMass(String) }
pub fn solve_reaction(
    r1: &Reactant, r2: &Reactant,
    entry1: Option<ReactantEntry>, entry2: Option<ReactantEntry>,
    atomic_mass: impl Fn(&str) -> Option<f64>,
) -> Result<ReactionResult, ReactionError>;
```

- [ ] **Step 1:** Read Swift; port `solve_reaction`: classify (None→`Err(UnknownReactionClass)`); predict (`Infeasible`→`Ok` with `feasible=false`+message+empty products; `Products([])`→`Err(NoProducts)`); balance (`None`→`Err(Unbalanceable)`); molar masses via `atomic_mass` (missing→`Err(MissingAtomicMass)`); extent ξ + limiting; per-product yields; excess; diatomic messages. Port all `ReactionSolverTests` golden cases (neutralisation, combustion coefficients, single-displacement infeasible, 2 mol NaOH + 1 mol HCl limiting/excess). Provide a test `atomic_mass` closure from a small hard-coded map (as the Swift tests do). Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain reaction_solver` → PASS. `cargo fmt` + clippy clean.
- [ ] **Step 3:** Commit `feat(pt-domain): add end-to-end reaction solver`.

---

### Task 11: Oxidation state

**Files:** Create `crates/pt-domain/src/engine/oxidation_state.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/OxidationState.swift` + `$STEST/OxidationStateTests.swift`.

**Consumes:** `crate::POLYATOMIC_IONS` (with `composition` from Task 1).
**Produces:** `pub fn oxidation_state(composition: &BTreeMap<String,i64>) -> Option<BTreeMap<String,i64>>`.

- [ ] **Step 1:** Read Swift; port: single element → 0; polyatomic factoring against `POLYATOMIC_IONS` (largest-atom-count first, disjoint-remainder guard, counter-ion charge, `statesWithinIon`); else element rules (F/O/H/halogen/group1/group2 symbol sets) + single-unknown solve-by-difference; ≥2 unknowns → `None`. Port all `OxidationStateTests` cases (free element, NaCl, MgO, CO₂→C+4, KMnO₄→Mn+7, FeCl₃→Fe+3, NaOH, Na₂SO₄→S+6, CuSO₄→Cu+2, Na₂CO₃, NH₄Cl→N−3, CuS→None). Keep the doc comment about the NH₄NO₃ limitation. Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain oxidation_state` → PASS. `cargo fmt` + clippy clean.
- [ ] **Step 3:** Commit `feat(pt-domain): add oxidation-state assignment`.

---

### Task 12: Redox analysis

**Files:** Create `crates/pt-domain/src/engine/redox.rs`; modify `mod.rs`, `lib.rs`.
**Reference:** `$SWIFT/Reaction/RedoxAnalysis.swift` + `$STEST/RedoxAnalysisTests.swift`.

**Consumes:** `ReactionResult`/`BalancedTerm` (T10), `oxidation_state` (T11).
**Produces:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxidationChange { Oxidised, Reduced, Unchanged }
#[derive(Debug, Clone, PartialEq)]
pub struct ElementRedox { pub symbol: String, pub before: i64, pub after: i64, pub change: OxidationChange, pub reactant_formula: String, pub product_formula: String }
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
pub fn analyze_redox(result: &ReactionResult) -> RedoxAnalysis;
```
NOTE: no `name` closure — narrative uses formulas. Signed rendering `+n` / `0` / `−n` (Unicode minus U+2212). Include the ambiguous-same-side → `indeterminate` fix (with dedup).

- [ ] **Step 1:** Read Swift; port `analyze_redox`: guard feasible+products (else empty); per-compound `oxidation_state` (None→`indeterminate`); per-element before/after across reactant/product terms (ambiguous same-side → record formulas in `indeterminate`, skip); `is_redox`; agents; narrative sentences + agent-pair summary; non-redox line; dedup `indeterminate`. Port all `RedoxAnalysisTests` cases (synthesis Na/Cl₂, single-displacement Zn/CuSO₄ agents, combustion CH₄/O₂, neutralisation non-redox, infeasible empty, and the ambiguous-→indeterminate regression). Build test `ReactionResult`s directly (as the Swift tests do). Wire `mod.rs`+`lib.rs`.
- [ ] **Step 2:** `cargo test -p pt-domain redox` → PASS.
- [ ] **Step 3:** Run the FULL native suite: `cargo test --workspace --exclude pt-wasm` → all green (existing + new). `cargo fmt` + clippy clean.
- [ ] **Step 4:** Commit `feat(pt-domain): add redox analysis over solved reactions`.

---

### Task 13: pt-wasm compound-reaction binding

**Files:** Create `crates/pt-wasm/src/compound_reaction.rs`; modify `crates/pt-wasm/src/lib.rs`.
**Reference:** existing `crates/pt-wasm/src/reaction.rs` (Tsify + `ServiceTable::by_symbol` pattern) + the Swift `SpeciesMapping.swift` (`$SWIFT_APP/State/SpeciesMapping.swift`, where `$SWIFT_APP=/Users/putumas.mertayasa.e/Developer/tools/ios-chem-interactive/ChemInteractive`) for the pair-aware charge rule.

**Consumes:** all pt-domain items above; `pt_services::PeriodicTable`.
**Produces:** `#[wasm_bindgen] pub fn solve_compound_reaction(zone1: Vec<WasmSpecies>, zone2: Vec<WasmSpecies>, q1: Option<WasmQuantity>, q2: Option<WasmQuantity>) -> WasmReactionResult` and the Tsify types `WasmSpecies`/`WasmQuantity` (in) and `WasmTerm`/`WasmElementRedox`/`WasmRedox`/`WasmReactionResult` (out), per the spec's Section 2 shapes.

- [ ] **Step 1: Write the resolver + binding.** Read `reaction.rs` for the Tsify/table pattern. For each `WasmSpecies`: element → `table.by_symbol` gives atomic_mass (`element().atomic_mass`), group, period, `element_class`, valence (`parse_valence_electrons`), composition `{symbol:1}`; polyatomic → look up `POLYATOMIC_IONS`, mass = Σ constituent atomic masses, composition from its `composition`, charge = ion charge. Apply the **pair-aware charge rule** (port `SpeciesMapping.buildReactant`/`ionicCharge`/`isIonicPair`/`isAcidPair`): polyatomic OR metal+nonmetal OR H+halogen → ionic charges; other nonmetal+nonmetal → covalent (charge `None`); single species → its ionic charge. `make_reactant` each zone; `solve_reaction` with an `atomic_mass` closure over the table; on `Ok(feasible)` also `analyze_redox`. Map to the Wasm output types; on `Err`, set `feasible=false` + `error`.

- [ ] **Step 2: Add `#[wasm_bindgen_test]` golden cases** (in `compound_reaction.rs`): NaOH+HCl→{NaCl,H₂O} non-redox; 2HCl+Na₂CO₃→3 products; Zn+CuSO₄ redox (reducing Zn, oxidising CuSO₄) with charge picks; Cu+ZnSO₄ → `feasible=false`; CH₄+O₂ → combustion; Fe+O₂ → `reaction_class=="Synthesis"`; a not-classified pair → `feasible=false`/`error`. Build `ServiceTable::load_bundled()` as in the existing tests. Add `mod compound_reaction;` to `lib.rs`.

- [ ] **Step 3: Run.** `wasm-pack test --node crates/pt-wasm` → PASS. Then `make test` (native + wasm) → all green. `cargo fmt`.

- [ ] **Step 4: Commit** `feat(pt-wasm): add solve_compound_reaction binding + redox`.

---

### Task 14: chem-interactive — rebuild pkg + typed wrapper

**Files (in `chem-interactive`):** regenerate `src/wasm/pkg/*`; create `src/wasm/reaction.ts`; add a smoke test.
**Reference:** the header of the current `chem-interactive/src/wasm/pkg/pt_wasm.js` (to match the wasm-pack `--target`).

**Consumes:** the `solve_compound_reaction` export + generated `.d.ts` types from Task 13.

- [ ] **Step 1: Determine the build target.** Read `chem-interactive/src/wasm/pkg/pt_wasm.js` line 2 (`import * as wasm from "./pt_wasm_bg.wasm"`) — this is the `--target web` (or bundler) layout. Match it.
- [ ] **Step 2: Build the wasm** in `sam-periodic-table`: `wasm-pack build crates/pt-wasm --target web --out-dir pkg` (adjust `--target` to match Step 1). Expect `pkg/{pt_wasm.js, pt_wasm_bg.wasm, pt_wasm_bg.js, pt_wasm.d.ts}` and that `pt_wasm.d.ts` now contains `solve_compound_reaction` + the Wasm* types.
- [ ] **Step 3: Copy** into chem-interactive: `cp sam-periodic-table/pkg/pt_wasm* chem-interactive/src/wasm/pkg/` (only the four generated files; do not delete other existing pkg files).
- [ ] **Step 4: Add the typed wrapper** `chem-interactive/src/wasm/reaction.ts`:

```ts
import init, { solve_compound_reaction, type WasmSpecies, type WasmQuantity, type WasmReactionResult } from './pkg/pt_wasm'

let ready: Promise<unknown> | null = null
function ensureInit() { return (ready ??= init()) }

export async function solveCompoundReaction(
  zone1: WasmSpecies[], zone2: WasmSpecies[],
  q1?: WasmQuantity, q2?: WasmQuantity,
): Promise<WasmReactionResult> {
  await ensureInit()
  return solve_compound_reaction(zone1, zone2, q1, q2)
}
```
(Adjust the import names to the exact generated `.d.ts`; if the existing wasm module already centralises `init`, reuse that instead of a second `init()`.)

- [ ] **Step 5: Smoke test.** Add a vitest (or the project's test runner) case asserting `solveCompoundReaction([{symbol:'Na',is_polyatomic:false},{symbol:'OH',is_polyatomic:true}], [{symbol:'H',is_polyatomic:false},{symbol:'Cl',is_polyatomic:false}])` resolves to `feasible===true`, `reaction_class==='Double displacement'`, and products including `NaCl` and `H₂O`. Run the project's test command (check `chem-interactive/package.json` scripts, e.g. `npm test`).
- [ ] **Step 6:** Build the app to confirm types resolve: `npm run build` (in chem-interactive) → success.
- [ ] **Step 7: Commit** in chem-interactive: `feat(wasm): consume solve_compound_reaction from pt-wasm`.

---

## Self-Review Notes (addressed)

- **Spec coverage:** every pt-domain module (Tasks 1–12), pt-wasm binding + wasm tests (Task 13), chem-interactive rebuild+wrapper+smoke (Task 14). `PolyatomicIon.composition` (T1), Fe→synthesis (T7), redox formulas-only + indeterminate fix (T12), pair-aware charge rule in the wasm resolver (T13).
- **Port fidelity:** each task names the exact Swift source + test file; Rust golden values must equal Swift's.
- **Type consistency:** the `Produces` blocks define each module's Rust API; later tasks consume those exact names/signatures (Species, Reactant, ReactionClass, Product/Prediction, BalancedTerm/ReactionResult/ReactionError, RedoxAnalysis, balance, solve_reaction, analyze_redox, oxidation_state).
- **Additive:** existing pt-domain/pt-wasm + binary bindings untouched; only `mod.rs`/`lib.rs`/`polyatomic.rs` extended.
- **Branches:** both repos stay on their current feature branches (no new branch), per the spec.
