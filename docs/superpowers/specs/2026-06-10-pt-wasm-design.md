# pt-wasm Design Spec

**Date:** 2026-06-10
**Branch:** feat/expose-as-wasm
**Consumer:** React + Vite browser application
**Distribution:** Local build artifact (`pkg/pt-wasm/`)

---

## Goal

Expose the `pt-services` Rust library as a WebAssembly module consumable from TypeScript/React. The WASM binary must be self-contained — all 118 element YAML files are baked in at compile time so the consumer has no filesystem dependency.

---

## Crate Structure

```
sam-periodic-table/
├── crates/
│   ├── pt-domain/       (unchanged)
│   ├── pt-data/         (add build.rs + bundled feature)
│   ├── pt-services/     (add bundled feature, passes through to pt-data)
│   ├── pt-cli/          (unchanged)
│   └── pt-wasm/         (NEW — wasm-bindgen wrappers)
└── data/elements/       (118 YAML files, unchanged)
```

`pt-wasm` must be added to the workspace `members` list in the root `Cargo.toml`.

Feature flag chain:

```
pt-wasm
  └── pt-services [bundled]
        └── pt-data [bundled]
              └── build.rs reads data/elements/*.yaml
                    └── emits OUT_DIR/generated_elements.rs
```

---

## Data Embedding — `pt-data` `build.rs` Codegen

`build.rs` in `pt-data` runs on the host at compile time:

1. Reads all `*.yaml` / `*.yml` files from `data/elements/` (relative to crate root: `../../data/elements/`)
2. Parses each with `serde_yaml_ng` (host-only, `build-dependencies` only — never enters WASM binary)
3. Emits `$OUT_DIR/generated_elements.rs` containing a `static ELEMENTS: &[RawElement]`

### Generated output shape

```rust
pub static ELEMENTS: &[RawElement] = &[
    RawElement {
        atomic_number: 1,
        name: "Hydrogen",
        symbol: "H",
        atomic_mass: 1.008,
        mass_number: 1,
        melting_point: Some(13.99),
        boiling_point: Some(20.271),
        density: Some(0.00008988),
        electronegativity: Some(2.20),
        state: "gas",
        discovery_year: Some(1766),
        discoverer: Some("Henry Cavendish"),
        isotopes: &[
            RawIsotope { mass_number: 1, relative_mass: 1.007825, abundance: 0.999885 },
            RawIsotope { mass_number: 2, relative_mass: 2.014102, abundance: 0.000115 },
        ],
    },
    // ... 117 more
];
```

`RawElement` and `RawIsotope` use `&'static str` for string fields and `&'static [RawIsotope]` for isotopes — zero heap allocation, pure static data.

`build.rs` emits `cargo:rerun-if-changed` for every YAML file, so Cargo rebuilds on data edits.

### `bundled` feature in `pt-data`

`RawElement` and `RawIsotope` are plain Rust structs defined in `pt-data/src/bundled.rs`. The generated `include!` file references them, so they must be declared before the `include!` call:

```rust
// src/bundled.rs
pub struct RawIsotope {
    pub mass_number: u16,
    pub relative_mass: f64,
    pub abundance: f64,
}

pub struct RawElement {
    pub atomic_number: u8,
    pub name: &'static str,
    pub symbol: &'static str,
    pub atomic_mass: f64,
    pub mass_number: u16,
    pub melting_point: Option<f64>,
    pub boiling_point: Option<f64>,
    pub density: Option<f64>,
    pub electronegativity: Option<f64>,
    pub state: &'static str,
    pub discovery_year: Option<i32>,
    pub discoverer: Option<&'static str>,
    pub isotopes: &'static [RawIsotope],
}

include!(concat!(env!("OUT_DIR"), "/generated_elements.rs"));

pub fn load_bundled() -> Result<crate::ElementRepository, crate::error::DataError> {
    crate::ElementRepository::load_from_static(ELEMENTS)
}
```

```rust
// src/lib.rs
#[cfg(feature = "bundled")]
mod bundled;
#[cfg(feature = "bundled")]
pub use bundled::load_bundled;
```

`ElementRepository::load_from_static` accepts `&[RawElement]` and applies the same indexing logic as `load_from_dir` — same validation, same HashMap indexes.

### `bundled` feature in `pt-services`

```rust
// src/table.rs
impl PeriodicTable {
    #[cfg(feature = "bundled")]
    pub fn load_bundled() -> Result<Self, ServiceError> {
        Ok(Self { repo: pt_data::load_bundled()? })
    }
}
```

---

## `pt-wasm` Crate

### `Cargo.toml`

```toml
[package]
name = "pt-wasm"
edition.workspace = true
version.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
pt-services = { workspace = true, features = ["bundled"] }
wasm-bindgen = "0.2"
tsify = { version = "0.4", features = ["js"] }
serde = { workspace = true }
serde-wasm-bindgen = "0.6"
js-sys = "0.3"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

### Owned transfer type: `WasmElement`

`ElementView<'a>` holds a borrow — cannot cross the WASM boundary. `WasmElement` is a flat owned struct built from `ElementView` by copying all stored fields and materialising all computed properties.

```rust
#[derive(serde::Serialize, tsify::Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmIsotope {
    pub mass_number: u16,
    pub relative_mass: f64,
    pub abundance: f64,
}

#[derive(serde::Serialize, tsify::Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmElement {
    // stored
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
    // computed
    pub electron_configuration: String,
    pub group: u8,
    pub period: u8,
    pub block: String,
    pub category: String,
    pub oxidation_states: Vec<i8>,
    pub computed_atomic_mass: Option<f64>,
}
```

`tsify` generates a matching TypeScript interface automatically — no hand-written `.d.ts`.

### `PeriodicTable` WASM wrapper

```rust
#[wasm_bindgen]
pub struct PeriodicTable(pt_services::PeriodicTable);

#[wasm_bindgen]
impl PeriodicTable {
    /// Loads the bundled element data. Synchronous — no filesystem access.
    pub fn load() -> Result<PeriodicTable, JsError> {
        Ok(Self(pt_services::PeriodicTable::load_bundled()?))
    }

    pub fn by_symbol(&self, symbol: &str) -> Option<WasmElement> { ... }
    pub fn by_name(&self, name: &str) -> Option<WasmElement> { ... }
    pub fn by_atomic_number(&self, z: u8) -> Option<WasmElement> { ... }
    pub fn by_atomic_mass(&self, mass: f64, tolerance: f64) -> Option<WasmElement> { ... }
    pub fn all(&self) -> Vec<WasmElement> { ... }

    /// Physical state at the given temperature in Kelvin.
    /// Returns "Solid", "Liquid", "Gas", or null if melting/boiling points are unknown.
    pub fn state_at(&self, symbol: &str, temperature_k: f64) -> Option<String> { ... }
}
```

`PeriodicTable.load()` is synchronous — all data is static. Only `init()` (WASM module bootstrap) is async.

---

## TypeScript Consumer (React + Vite)

### Vite config

```ts
// vite.config.ts
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
});
```

Required npm dev deps: `vite-plugin-wasm`, `vite-plugin-top-level-await`.

### Usage

```typescript
import init, { PeriodicTable } from '../pkg/pt-wasm';

await init();
const pt = PeriodicTable.load();

const iron = pt.by_symbol("Fe");       // WasmElement | undefined
const all  = pt.all();                 // WasmElement[]
const near = pt.by_atomic_mass(55.845, 0.1);
const state = pt.state_at("Fe", 1900); // "Liquid" | null
```

### Build command

```bash
wasm-pack build crates/pt-wasm --target bundler --out-dir ../../pkg/pt-wasm
```

Output:
```
pkg/pt-wasm/
├── pt_wasm_bg.wasm
├── pt_wasm.js        (ESM glue)
├── pt_wasm.d.ts      (generated TypeScript types)
└── package.json
```

---

## Testing

- `wasm-bindgen-test` crate for in-browser / headless WASM tests of `pt-wasm`
- Existing `pt-data` and `pt-services` unit tests cover non-WASM paths unchanged
- `build.rs` codegen validated by the existing `bundled` test: load via `load_bundled()`, assert all 118 elements present and indexed correctly

---

## Out of Scope

- Publishing to npm
- Node.js / non-browser targets
- Async streaming of element data
- Modifying the YAML data format
