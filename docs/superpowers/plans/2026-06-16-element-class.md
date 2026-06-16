# Element Class Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a computed `ElementClass` (Metal / NonMetal / Metalloid) property across pt-domain, pt-services, and pt-wasm.

**Architecture:** `element_class(z: u8)` lives in `pt-domain/classification.rs` and uses the existing `group()` and `period()` functions. `ElementView` in pt-services delegates to it. `WasmElement` in pt-wasm serialises the result as a `String`, and a `tsify`-derived `ElementClass` enum provides the TypeScript union type.

**Tech Stack:** Rust, wasm-bindgen, tsify, serde

---

## File Map

| File | Change |
|------|--------|
| `crates/pt-domain/src/classification.rs` | Add `ElementClass` enum, `CLASS_METALLOIDS` const, `element_class(z)` fn, unit tests |
| `crates/pt-domain/src/lib.rs` | Re-export `ElementClass` and `element_class` |
| `crates/pt-services/src/view.rs` | Add `element_class()` method, smoke test |
| `crates/pt-wasm/src/element.rs` | Add `ElementClass` tsify enum, `class: String` field, From impl, test assertion |

---

## Task 1: ElementClass in pt-domain

**Files:**
- Modify: `crates/pt-domain/src/classification.rs`
- Modify: `crates/pt-domain/src/lib.rs`

- [ ] **Step 1: Write failing tests**

Add this test block inside the existing `mod tests` at the bottom of `crates/pt-domain/src/classification.rs`:

```rust
    #[test]
    fn element_class_hydrogen_exception() {
        assert_eq!(element_class(1).unwrap(), ElementClass::NonMetal);
    }

    #[test]
    fn element_class_halogens_and_noble_gases() {
        assert_eq!(element_class(17).unwrap(), ElementClass::NonMetal); // Cl
        assert_eq!(element_class(35).unwrap(), ElementClass::NonMetal); // Br
        assert_eq!(element_class(2).unwrap(), ElementClass::NonMetal);  // He
        assert_eq!(element_class(10).unwrap(), ElementClass::NonMetal); // Ne
    }

    #[test]
    fn element_class_lanthanides_and_actinides() {
        assert_eq!(element_class(57).unwrap(), ElementClass::Metal); // La
        assert_eq!(element_class(71).unwrap(), ElementClass::Metal); // Lu
        assert_eq!(element_class(89).unwrap(), ElementClass::Metal); // Ac
        assert_eq!(element_class(92).unwrap(), ElementClass::Metal); // U
        assert_eq!(element_class(103).unwrap(), ElementClass::Metal); // Lr
    }

    #[test]
    fn element_class_metalloids() {
        for &z in &[5u8, 14, 32, 33, 51, 52, 84] {
            assert_eq!(element_class(z).unwrap(), ElementClass::Metalloid, "z={z}");
        }
        // At (85) is group 17 — non-metal, not metalloid
        assert_eq!(element_class(85).unwrap(), ElementClass::NonMetal);
    }

    #[test]
    fn element_class_nonmetals_above_staircase() {
        assert_eq!(element_class(6).unwrap(), ElementClass::NonMetal);  // C  (period 2, group 14)
        assert_eq!(element_class(7).unwrap(), ElementClass::NonMetal);  // N  (period 2, group 15)
        assert_eq!(element_class(8).unwrap(), ElementClass::NonMetal);  // O  (period 2, group 16)
        assert_eq!(element_class(15).unwrap(), ElementClass::NonMetal); // P  (period 3, group 15)
        assert_eq!(element_class(16).unwrap(), ElementClass::NonMetal); // S  (period 3, group 16)
        assert_eq!(element_class(34).unwrap(), ElementClass::NonMetal); // Se (period 4, group 16)
    }

    #[test]
    fn element_class_metals() {
        assert_eq!(element_class(11).unwrap(), ElementClass::Metal); // Na
        assert_eq!(element_class(12).unwrap(), ElementClass::Metal); // Mg
        assert_eq!(element_class(26).unwrap(), ElementClass::Metal); // Fe
        assert_eq!(element_class(79).unwrap(), ElementClass::Metal); // Au
    }

    #[test]
    fn element_class_invalid_z() {
        assert!(element_class(0).is_err());
        assert!(element_class(119).is_err());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p pt-domain element_class 2>&1 | head -30
```

Expected: compile errors — `element_class` and `ElementClass` not defined yet.

- [ ] **Step 3: Add ElementClass enum, CLASS_METALLOIDS, and element_class() to classification.rs**

Add immediately after the `Category` enum definition (after line `    Actinide,` closing brace), before the `METALLOIDS` const:

```rust
/// Broad three-way element classification used for UI colour-coding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementClass {
    Metal,
    NonMetal,
    Metalloid,
}

/// The 7 metalloids per spec: B Si Ge As Sb Te Po.
/// Intentionally differs from METALLOIDS (used by category()) which has At(85) not Po(84).
const CLASS_METALLOIDS: [u8; 7] = [5, 14, 32, 33, 51, 52, 84];

/// Broad Metal / NonMetal / Metalloid classification derived from atomic number.
pub fn element_class(z: u8) -> Result<ElementClass, DomainError> {
    use ElementClass::*;
    validate_z(z)?;
    let g = group(z)?;
    let p = period(z)?;
    if z == 1                                              { return Ok(NonMetal);  } // H exception
    if g == 17 || g == 18                                  { return Ok(NonMetal);  } // halogens + noble gases
    if (57..=71).contains(&z) || (89..=103).contains(&z)  { return Ok(Metal);     } // lanthanides + actinides
    if CLASS_METALLOIDS.contains(&z)                       { return Ok(Metalloid); }
    if p == 2 && (14..=16).contains(&g)                    { return Ok(NonMetal);  } // C N O
    if p == 3 && (15..=16).contains(&g)                    { return Ok(NonMetal);  } // P S
    if p == 4 && g == 16                                   { return Ok(NonMetal);  } // Se
    Ok(Metal)
}
```

- [ ] **Step 4: Re-export from pt-domain/src/lib.rs**

Update the `classification` re-export line (currently ends at `OxidationStates`):

```rust
pub use classification::{
    block, category, element_class, group, oxidation_states, period,
    Block, Category, ElementClass, OxidationStates,
};
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test -p pt-domain element_class
```

Expected output (all 7 new tests pass):
```
test classification::tests::element_class_hydrogen_exception ... ok
test classification::tests::element_class_halogens_and_noble_gases ... ok
test classification::tests::element_class_lanthanides_and_actinides ... ok
test classification::tests::element_class_metalloids ... ok
test classification::tests::element_class_nonmetals_above_staircase ... ok
test classification::tests::element_class_metals ... ok
test classification::tests::element_class_invalid_z ... ok
```

- [ ] **Step 6: Run full pt-domain test suite to check for regressions**

```bash
cargo test -p pt-domain
```

Expected: all existing tests still pass.

- [ ] **Step 7: Commit**

```bash
git add crates/pt-domain/src/classification.rs crates/pt-domain/src/lib.rs
git commit -m "feat(pt-domain): add ElementClass enum and element_class() function"
```

---

## Task 2: ElementView::element_class() in pt-services

**Files:**
- Modify: `crates/pt-services/src/view.rs`

- [ ] **Step 1: Write failing test**

Add inside the existing `mod tests` in `crates/pt-services/src/view.rs`, within the `exposes_computed_properties` test or as a new test after it:

```rust
    #[test]
    fn exposes_element_class() {
        let e = iron();
        let view = ElementView::new(&e);
        assert_eq!(view.element_class(), pt_domain::ElementClass::Metal);
    }
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p pt-services exposes_element_class 2>&1 | head -20
```

Expected: compile error — `element_class` method not defined on `ElementView`.

- [ ] **Step 3: Add element_class() method to ElementView**

Update the import at the top of `crates/pt-services/src/view.rs`:

```rust
use pt_domain::{
    self as domain, Block, Category, ElementClass, ElectronConfiguration, Element,
    OxidationStates, StateOfMatter,
};
```

Add this method to the `ElementView` impl block, after `oxidation_states()`:

```rust
    pub fn element_class(&self) -> ElementClass {
        domain::element_class(self.element.atomic_number).expect("valid atomic number")
    }
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -p pt-services exposes_element_class
```

Expected:
```
test tests::exposes_element_class ... ok
```

- [ ] **Step 5: Run full pt-services test suite**

```bash
cargo test -p pt-services
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/pt-services/src/view.rs
git commit -m "feat(pt-services): expose element_class() on ElementView"
```

---

## Task 3: class field in pt-wasm WasmElement

**Files:**
- Modify: `crates/pt-wasm/src/element.rs`

- [ ] **Step 1: Write failing test assertion**

In `crates/pt-wasm/src/element.rs`, extend the existing `converts_computed_fields` test to assert the new field:

```rust
    #[test]
    fn converts_computed_fields() {
        let iron = make_iron();
        let view = ElementView::new(&iron);
        let w = WasmElement::from(view);
        assert_eq!(w.group, 8);
        assert_eq!(w.period, 4);
        assert_eq!(w.block, "D");
        assert_eq!(w.category, "TransitionMetal");
        assert_eq!(w.oxidation_states, vec![2i8, 3i8]);
        assert!(w.electron_configuration.contains("3d6"));
        assert_eq!(w.class, "Metal"); // NEW
    }
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p pt-wasm converts_computed_fields 2>&1 | head -20
```

Expected: compile error — `class` field not on `WasmElement`.

- [ ] **Step 3: Add ElementClass enum and class field to WasmElement**

At the top of `crates/pt-wasm/src/element.rs`, update the `use` imports:

```rust
use pt_domain::ElementClass;
use pt_services::ElementView;
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
```

Add the `ElementClass` tsify enum after the `WasmIsotope` struct:

```rust
/// Three-way element classification exposed to TypeScript as a union type.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub enum WasmElementClass {
    Metal,
    NonMetal,
    Metalloid,
}
```

Add `class: String` to `WasmElement` (after `computed_atomic_mass`):

```rust
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmElement {
    pub atomic_number: u8,
    pub name: String,
    pub symbol: String,
    pub atomic_mass: f64,
    pub mass_number: u16,
    pub melting_point: Option<f64>,
    pub boiling_point: Option<f64>,
    pub density: Option<f64>,
    pub electronegativity: Option<f64>,
    pub state: String,
    pub discovery_year: Option<i32>,
    pub discoverer: Option<String>,
    pub isotopes: Vec<WasmIsotope>,
    pub electron_configuration: String,
    pub group: u8,
    pub period: u8,
    pub block: String,
    pub category: String,
    pub oxidation_states: Vec<i8>,
    pub computed_atomic_mass: Option<f64>,
    pub class: String,
}
```

Update `From<ElementView<'_>> for WasmElement` — add the `class` field at the end of the `Self { ... }` block:

```rust
            class: format!("{:?}", view.element_class()),
```

The complete updated `From` impl (for reference, showing only the changed struct literal ending):

```rust
impl From<ElementView<'_>> for WasmElement {
    fn from(view: ElementView<'_>) -> Self {
        let e = view.element();
        Self {
            atomic_number: e.atomic_number,
            name: e.name.clone(),
            symbol: e.symbol.clone(),
            atomic_mass: e.atomic_mass,
            mass_number: e.mass_number,
            melting_point: e.melting_point,
            boiling_point: e.boiling_point,
            density: e.density,
            electronegativity: e.electronegativity,
            state: format!("{:?}", e.state),
            discovery_year: e.discovery_year,
            discoverer: e.discoverer.clone(),
            isotopes: e
                .isotopes
                .iter()
                .map(|i| WasmIsotope {
                    mass_number: i.mass_number,
                    relative_mass: i.relative_mass,
                    abundance: i.abundance,
                })
                .collect(),
            electron_configuration: view.electron_configuration().to_string(),
            group: view.group(),
            period: view.period(),
            block: format!("{:?}", view.block()),
            category: format!("{:?}", view.category()),
            oxidation_states: view.oxidation_states().0,
            computed_atomic_mass: view.computed_atomic_mass(),
            class: format!("{:?}", view.element_class()),
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -p pt-wasm converts_computed_fields
```

Expected:
```
test tests::converts_computed_fields ... ok
```

- [ ] **Step 5: Run full pt-wasm test suite**

```bash
cargo test -p pt-wasm
```

Expected: all tests pass.

- [ ] **Step 6: Run full workspace test suite**

```bash
cargo test --workspace
```

Expected: all tests pass across all crates.

- [ ] **Step 7: Commit**

```bash
git add crates/pt-wasm/src/element.rs
git commit -m "feat(pt-wasm): add class field and WasmElementClass enum to WasmElement"
```

---

## Done

After Task 3 completes, `WasmElement.class` is available to the TypeScript frontend as `"Metal" | "NonMetal" | "Metalloid"`. The `WasmElementClass` enum in `pt_wasm.d.ts` provides the TypeScript union type (auto-generated by tsify when `wasm-pack build` is run).
