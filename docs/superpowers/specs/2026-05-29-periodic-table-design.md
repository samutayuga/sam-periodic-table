# Periodic Table Library — Design Spec

**Date:** 2026-05-29
**Status:** Approved (design), pending implementation plan

## 1. Summary

A Rust library providing access to the properties of the chemical elements of the
periodic table. Element data is maintained as human-editable YAML files (one per
element) under `data/elements/`. The library distinguishes between:

- **Stored properties** — empirical or historical facts loaded from YAML.
- **Computed properties** — derived at runtime by the domain layer from the
  element's atomic number / electron configuration.

The system is organized as a Cargo workspace with three library layers (data,
domain, services) plus a CLI binary.

## 2. Goals & Non-Goals

### Goals
- Maintain an exhaustive, easily-updatable dataset for all 118 elements as YAML.
- Cleanly separate stored data from physically-derivable computations.
- Expose a typed Rust API to look up an element by atomic number, name, symbol, or
  atomic mass, returning stored + computed properties.
- Provide a CLI for terminal queries.
- Per-crate unit tests and integration tests, with documentation on every public
  function and service.

### Non-Goals
- No HTTP/REST API, no async runtime.
- No live scraping of the source URL (`https://www.iamocean.com/periodic-table/`).
  Data is populated best-effort from standard public reference values.
- `oxidation_states` and `category` are heuristic, **not** authoritative.
- No orbital-diagram visualization.

## 3. Property Classification

The original requirement listed ~15 "calculations," but many are not physically
derivable from atomic mass/number/radius (e.g. `discoverer`, `discovery_year`,
`melting_point`). Properties are therefore split into two buckets:

**Stored in YAML** (standard reference data):
`atomic_number`, `name`, `symbol`, `atomic_mass`, `mass_number`,
`melting_point`, `boiling_point`, `density`, `electronegativity`, `state`,
`isotopes` (mass number + relative mass + abundance), `discovery_year`,
`discoverer`.

**Computed by the domain layer** (from atomic number / electron configuration):
`electronic_configuration`, `group`, `period`, `block`, `category`,
`oxidation_states` (heuristic), `atomic_mass_from_isotopes` (validation against
stored value), and `state_at(temperature)` (derived from stored melting/boiling
points — the physically-sound replacement for "state from mass/radius").

## 4. Workspace Layout

```
periodic-table/
├── Cargo.toml                 # workspace manifest
├── data/elements/*.yaml       # 118 element files (source of truth)
├── crates/
│   ├── pt-domain/             # lib: types + pure calculations
│   ├── pt-data/               # lib: YAML loading + repository
│   ├── pt-services/           # lib: query API
│   └── pt-cli/                # bin: terminal interface
└── docs/superpowers/specs/    # this spec
```

Dependency direction (no cycles): `pt-cli → pt-services → pt-data → pt-domain`.
`pt-domain` has no internal dependencies.

## 5. Data Model

### 5.1 YAML schema

Units are fixed and documented: temperatures in **Kelvin**, density in **g/cm³**,
atomic mass in **u**, electronegativity on the **Pauling scale**.

Example `data/elements/hydrogen.yaml`:

```yaml
atomic_number: 1
name: Hydrogen
symbol: H
atomic_mass: 1.008
mass_number: 1               # nucleon count of the most abundant isotope
                             # (most stable known isotope for synthetic elements)
melting_point: 13.99
boiling_point: 20.271
density: 0.00008988
electronegativity: 2.20      # nullable (e.g. noble gases)
state: gas                   # solid | liquid | gas, at STP (298.15 K)
discovery_year: 1766         # nullable (ancient elements)
discoverer: Henry Cavendish  # nullable
isotopes:
  - { mass_number: 1, relative_mass: 1.007825, abundance: 0.999885 }
  - { mass_number: 2, relative_mass: 2.014102, abundance: 0.000115 }
```

### 5.2 Rust types (`pt-domain`)

- `Element` — all stored fields. Empirical fields that may be unmeasured for
  synthetic elements are `Option<T>` (`melting_point`, `boiling_point`, `density`,
  `electronegativity`, `discovery_year`, `discoverer`).
- `Isotope { mass_number: u16, relative_mass: f64, abundance: f64 }`.
- `StateOfMatter { Solid, Liquid, Gas }`.
- Computed value types: `ElectronConfiguration` (ordered subshells with
  occupancy + display helpers), `Block { S, P, D, F }`, `Group(u8)` (optional for
  f-block), `Period(u8)`, `Category` enum (alkali metal, alkaline earth,
  transition metal, lanthanide, actinide, metalloid, halogen, noble gas,
  post-transition metal, nonmetal, …), `OxidationStates(Vec<i8>)`.

## 6. Data Layer (`pt-data`)

- `ElementRepository::load_from_dir(path) -> Result<Self, DataError>` — reads every
  `*.yaml`, deserializes, validates, and builds indexes. **Loaded once**; the caller
  holds the repository. An optional `OnceLock<ElementRepository>` global helper is
  provided for the CLI / single-instance applications.
- Indexes: `by_atomic_number` (primary), case-insensitive `by_symbol`, case-insensitive
  `by_name`.
- Validation rules:
  - `atomic_number` in `1..=118`.
  - Unique `atomic_number`, `symbol`, and `name` across the dataset.
  - Isotope `abundance` values sum to ≈ 1.0 within a documented tolerance (when
    isotopes are present).
  - Required identity fields present (`atomic_number`, `name`, `symbol`,
    `atomic_mass`, `mass_number`).

## 7. Domain Layer (`pt-domain`) — Calculations

All calculations are pure functions defined over a valid atomic number (`1..=118`).
Invalid input returns `DomainError::InvalidAtomicNumber`.

| Calculation | Method | Accuracy |
|---|---|---|
| `electronic_configuration` | Aufbau fill in Madelung (n+l) order; subshell capacities `s2 p6 d10 f14` (Pauli); Hund's rule used to compute unpaired-electron count | Exact, with a small **anomaly table** (~20 elements: Cr, Cu, Nb, Mo, Ru, Rh, Pd, Ag, Pt, Au, La, Ce, Gd, Ac, Th, Pa, U, Np, Cm, Lr) so output matches observed configurations |
| `group` | Derived from valence configuration (s/p/d/f rules; He → group 18) | Exact for main-group and transition metals; f-block assigned group 3 (documented) |
| `period` | Highest principal quantum number `n` in the configuration | Exact |
| `block` | Subshell type of the last-filled (differentiating) electron; He special-cased as s-block | Exact |
| `category` | From group/block plus fixed metalloid/nonmetal sets | Best-effort (partly conventional) |
| `oxidation_states` | Heuristic from group/configuration | **Best-effort, not authoritative** — flagged in docs |
| `atomic_mass_from_isotopes` | Σ(`relative_mass` × `abundance`) | Exact; a validation helper compares the result to the stored `atomic_mass` within tolerance |
| `state_at(temperature_k)` | `solid` if T < melting_point; `liquid` if melting_point ≤ T < boiling_point; `gas` if T ≥ boiling_point | Exact when melting/boiling points are known; returns `None` when they are not |

## 8. Service Layer (`pt-services`)

- `PeriodicTable::load(path) -> Result<Self, ServiceError>` — constructs the
  repository (delegates to `pt-data`).
- Lookups (return `Option<ElementView>`; not-found is `None`, not an error):
  - `by_atomic_number(u8)`
  - `by_symbol(&str)` (case-insensitive)
  - `by_name(&str)` (case-insensitive)
  - `by_atomic_mass(f64, tolerance: f64)` — nearest element whose stored atomic mass
    is within `tolerance` (mass is continuous, so exact match is not assumed).
- `all() -> impl Iterator<Item = ElementView>`.
- `ElementView` merges the stored `Element` with computed properties, computing each
  derived value on access.

## 9. CLI (`pt-cli`)

Built with `clap` (derive):

- `pt get --number 26`
- `pt get --symbol Fe`
- `pt get --name iron`
- `pt get --mass 55.85`
- `pt list`
- Global flags: `--data-dir <path>` (default `./data/elements`),
  `--format text|json|yaml` (default `text`).

Errors map to non-zero exit codes with clear messages.

## 10. Error Handling

`thiserror`-based, one error type per layer:

- `pt-data::DataError` — `Io`, `Parse { file, source }`, `Validation(String)`,
  `DuplicateAtomicNumber(u8)`, `DuplicateSymbol(String)`, `DuplicateName(String)`,
  `EmptyDataDir`.
- `pt-domain::DomainError` — `InvalidAtomicNumber(u16)`.
- `pt-services::ServiceError` — wraps `DataError` (`#[from]`); lookups return
  `Option` rather than erroring on not-found.
- `pt-cli` maps the above to user-facing messages and exit codes.

## 11. Testing Strategy

Per the requirement, each crate carries unit tests, and services have integration
tests.

- **`pt-domain` unit tests:** electron configurations for H, He, C, Ne, Ca, Sc, Fe,
  Cr & Cu (anomalies), La, Lr; group/period/block for representative elements;
  oxidation states for a few groups; `atomic_mass_from_isotopes` for Cl (³⁵Cl/³⁷Cl);
  `state_at` at melting/boiling boundaries.
- **`pt-data` unit tests:** parse valid YAML; reject malformed YAML; reject duplicate
  and out-of-range atomic numbers; abundance-sum check; load a fixture directory.
- **`pt-services` integration tests:** load the real data directory; exercise each
  lookup method, mass-tolerance lookup, and not-found paths.
- **`pt-cli` integration tests:** invoke the binary via `assert_cmd` and assert
  output for each subcommand and format.

## 12. Dependencies

- Runtime: `serde` (derive), `serde_yaml_ng` (maintained YAML crate; upstream
  `serde_yaml` is archived), `thiserror`, `clap` (derive, CLI only).
- Dev: `assert_cmd`, `predicates`.

## 13. Data Provenance & Caveats

- The 118-element dataset is populated best-effort from public reference values
  (IUPAC / standard tables).
- Synthetic superheavy elements will have `null` for unmeasured properties
  (`melting_point`, `boiling_point`, `density`, `electronegativity`).
- Elements known since antiquity will have `null` / "Ancient" for `discoverer` and
  may have `null` `discovery_year`.
- `oxidation_states` and `category` are heuristic and should not be relied upon as
  authoritative chemical references.
