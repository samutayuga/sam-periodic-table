# Element Class — Design Spec

**Date:** 2026-06-16
**Status:** Approved

---

## Overview

Add an `ElementClass` property (Metal / NonMetal / Metalloid) to the element type system. `ElementClass` is a coarser grouping than the existing `Category` and is intended primarily for UI colour-coding in the periodic table frontend.

`class` is a **computed** property derived from the atomic number at runtime — no YAML changes required.

---

## Architecture

Dependencies flow unchanged: `pt-domain` ← `pt-services` ← `pt-wasm`.

```
pt-domain/classification.rs
  + ElementClass { Metal, NonMetal, Metalloid }
  + element_class(z: u8) -> Result<ElementClass, DomainError>

pt-services/view.rs
  + ElementView::element_class() -> ElementClass

pt-wasm/element.rs
  + ElementClass enum (tsify + Serialize)  → TypeScript union type
  + WasmElement.class: String
```

`element_class(z)` internally calls the existing `group(z)` and `period(z)` — no new parameters, consistent with all other domain functions.

---

## `element_class` Logic (pt-domain)

```rust
pub enum ElementClass { Metal, NonMetal, Metalloid }

/// Atomic numbers of the 7 metalloids per spec (B Si Ge As Sb Te Po).
/// Intentionally differs from METALLOIDS used by category() which includes At(85) not Po(84).
const CLASS_METALLOIDS: [u8; 7] = [5, 14, 32, 33, 51, 52, 84];

pub fn element_class(z: u8) -> Result<ElementClass, DomainError> {
    validate_z(z)?;
    let g = group(z)?;
    let p = period(z)?;

    if z == 1                                              { return Ok(NonMetal) }  // H exception
    if g == 17 || g == 18                                  { return Ok(NonMetal) }  // halogens + noble gases
    if (57..=71).contains(&z) || (89..=103).contains(&z)  { return Ok(Metal)    }  // lanthanides + actinides
    if CLASS_METALLOIDS.contains(&z)                       { return Ok(Metalloid)}
    if p == 2 && (14..=16).contains(&g)                    { return Ok(NonMetal) }  // C N O
    if p == 3 && (15..=16).contains(&g)                    { return Ok(NonMetal) }  // P S
    if p == 4 && g == 16                                   { return Ok(NonMetal) }  // Se
    Ok(Metal)
}
```

Rule priority matches the spec pseudocode exactly. `CLASS_METALLOIDS` is a separate constant from `METALLOIDS` to make the intentional divergence (Po vs At) explicit and auditable.

---

## pt-services — ElementView

```rust
pub fn element_class(&self) -> ElementClass {
    domain::element_class(self.element.atomic_number).expect("valid atomic number")
}
```

Same `.expect()` pattern used by all other computed properties on `ElementView`.

---

## pt-wasm — TypeScript Exposure

```rust
// pt-wasm/element.rs

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub enum ElementClass { Metal, NonMetal, Metalloid }

// WasmElement gains:
pub class: String,  // serialised via format!("{:?}", view.element_class())
```

`WasmElement::From<ElementView>` sets `class: format!("{:?}", view.element_class())` — the same pattern used for `block` and `category`. TypeScript consumers see:

```typescript
iron.class    // "Metal"
boron.class   // "Metalloid"
oxygen.class  // "NonMetal"
```

`tsify` generates the union type `"Metal" | "NonMetal" | "Metalloid"` in `pt_wasm.d.ts`.

---

## Testing

### pt-domain (`classification.rs`)

| Test case | z | Expected |
|-----------|---|----------|
| Hydrogen exception | 1 | NonMetal |
| Halogen (Cl) | 17 | NonMetal |
| Noble gas (He) | 2 | NonMetal |
| Lanthanide (La) | 57 | Metal |
| Actinide (U) | 92 | Metal |
| Metalloid B | 5 | Metalloid |
| Metalloid Po | 84 | Metalloid |
| At (85) — non-metal, not metalloid | 85 | NonMetal |
| C (period 2, group 14) | 6 | NonMetal |
| N (period 2, group 15) | 7 | NonMetal |
| O (period 2, group 16) | 8 | NonMetal |
| P (period 3, group 15) | 15 | NonMetal |
| S (period 3, group 16) | 16 | NonMetal |
| Se (period 4, group 16) | 34 | NonMetal |
| Fe (transition metal) | 26 | Metal |
| Na (alkali metal) | 11 | Metal |
| Invalid z | 0 | Err |

### pt-services (`view.rs`)

Extend `exposes_computed_properties`: assert `view.element_class() == ElementClass::Metal` for iron.

### pt-wasm (`element.rs`)

Extend `converts_computed_fields`: assert `w.class == "Metal"` for iron.

---

## Caveats

- `ElementClass` and `Category` are independent. A future change to `category()`'s metalloid set does not affect `element_class()`.
- `ElementClass` is heuristic, not authoritative — the same caveat that applies to `category`.
- Superheavy elements (Z > 103, not lanthanides/actinides) fall through to the `Metal` default, which is consistent with IUPAC provisional assignments.
