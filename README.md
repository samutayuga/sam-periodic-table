# Periodic Table

A Rust library (plus a small CLI) that provides programmatic access to the
properties of the 118 chemical elements. Element data is maintained as
human-editable YAML files — one per element — and the library cleanly separates
**stored** empirical/historical data from **computed** properties that are
derived at runtime from first principles.

- **Stored** (loaded from YAML): atomic number, name, symbol, atomic mass, mass
  number, melting/boiling point, density, electronegativity, state, isotopes,
  discovery year, discoverer.
- **Computed** (derived on demand): electron configuration (Aufbau + Pauli +
  Hund, with a ground-state anomaly table), group, period, block, category,
  heuristic oxidation states, isotope-weighted atomic mass, and physical state at
  an arbitrary temperature.

---

## Workspace layout

```
periodic-table/
├── Cargo.toml                 # workspace manifest
├── data/elements/*.yaml       # 118 element files (source of truth)
└── crates/
    ├── pt-domain/             # pure value types + stateless calculations
    ├── pt-data/               # YAML loading + indexed repository
    ├── pt-services/           # query API (PeriodicTable, ElementView)
    └── pt-cli/                # the `pt` binary
```

Dependencies flow in one direction only:

```mermaid
flowchart TD
    CLI["pt-cli<br/>(binary: pt)"] --> SVC["pt-services<br/>PeriodicTable · ElementView"]
    SVC --> DATA["pt-data<br/>ElementRepository · parsing · validation"]
    SVC --> DOM["pt-domain<br/>types · calculations"]
    DATA --> DOM
    DATA -. reads .-> YAML[("data/elements/*.yaml")]
```

- **`pt-domain`** — no I/O, no serialization. Defines `Element`, `Isotope`,
  `StateOfMatter` and the stateless calculation functions.
- **`pt-data`** — owns serde DTOs, parses/validates YAML into domain types, and
  builds an in-memory `ElementRepository` (loaded once) indexed by atomic number,
  symbol, and name.
- **`pt-services`** — the public query API. `PeriodicTable` looks elements up;
  `ElementView` merges an element's stored fields with its computed properties.
- **`pt-cli`** — a thin binary named `pt` over `pt-services`.

---

## Building and testing

```bash
cargo build --workspace      # build everything
cargo test --workspace       # run all unit + integration tests
cargo run -p pt-cli -- --help
```

---

## User guide (library)

The crates are path/workspace members and are not published to crates.io. To use
the library from another crate in this workspace, depend on `pt-services` (it
re-exports everything you need and pulls in `pt-domain`):

```toml
[dependencies]
pt-services = { path = "crates/pt-services" }
pt-domain   = { path = "crates/pt-domain" }   # for the Block/Category/... types
```

Load the table once at startup, then query it:

```rust
use pt_services::PeriodicTable;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads every data/elements/*.yaml, validates, and indexes them.
    let table = PeriodicTable::load("data/elements")?;

    // Look up by atomic number, symbol (case-insensitive), or name.
    let fe = table.by_symbol("fe").expect("iron is present");

    // Stored properties live on the underlying Element.
    let e = fe.element();
    println!("{} ({}) — atomic mass {}", e.name, e.symbol, e.atomic_mass);

    // Computed properties are derived on access.
    println!("electron config : {}", fe.electron_configuration());
    println!("group / period  : {} / {}", fe.group(), fe.period());
    println!("block           : {:?}", fe.block());
    println!("category        : {:?}", fe.category());
    println!("oxidation states: {:?}", fe.oxidation_states().0);

    // Physical state at a chosen temperature (needs melting/boiling points).
    println!("state @ 2000 K  : {:?}", fe.state_at(2000.0)); // Some(Liquid)

    // Atomic mass recomputed from the stored isotope abundances.
    println!("mass (isotopes) : {:?}", fe.computed_atomic_mass());

    // Nearest element within a mass tolerance.
    if let Some(v) = table.by_atomic_mass(55.8, 0.5) {
        println!("nearest to 55.8 : {}", v.element().symbol);
    }

    // Iterate over the whole table.
    let metals = table.all().filter(|v| v.block() == pt_domain::Block::D).count();
    println!("d-block elements: {metals}");
    Ok(())
}
```

Lookups return `Option<ElementView>` — `None` means "not found" (not an error).
`PeriodicTable::load` returns `Result<_, ServiceError>`; loading fails only on I/O
or data-validation problems.

---

## How to use the CLI

The binary is named `pt`. From the repository root the default data directory
(`./data/elements`) is used automatically.

```bash
# Look up a single element (by number, symbol, name, or nearest atomic mass)
cargo run -p pt-cli -- get --number 26
cargo run -p pt-cli -- get --symbol Fe
cargo run -p pt-cli -- get --name iron
cargo run -p pt-cli -- get --mass 55.8

# List every element
cargo run -p pt-cli -- list

# Choose an output format and/or a custom data directory
cargo run -p pt-cli -- --format json get --symbol O
cargo run -p pt-cli -- --format yaml get --number 8
cargo run -p pt-cli -- --data-dir ./data/elements list
```

After `cargo build`, the binary is at `target/debug/pt`:

```bash
target/debug/pt get --symbol Fe
```

### Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--data-dir <PATH>` | `./data/elements` | Directory of element YAML files (global) |
| `--format <FMT>` | `text` | Output format: `text`, `json`, or `yaml` (global) |

### `get` selectors (use one)

`--number <Z>` · `--symbol <SYM>` · `--name <NAME>` · `--mass <U>` (nearest within ±0.5 u)

### Example output (`get --symbol Fe`, text)

```
Iron (Fe) — atomic number 26
  atomic mass:       55.845 u
  mass number:       56
  electron config:   1s² 2s² 2p⁶ 3s² 3p⁶ 3d⁶ 4s²
  group / period:    8 / 4
  block:             d
  category:          TransitionMetal
  state (STP):       solid
  melting point:     1811 K
  boiling point:     3134 K
  density:           7.874 g/cm³
  electronegativity: 1.83
  oxidation states:  2, 3
  discovered:        — (—)
```

Unmeasured/unknown fields render as `—`. A `get` with no match exits non-zero with
`error: no matching element found`.

In `json`/`yaml` output, properties that carry a physical unit are emitted as a
`{ value, unit }` object (e.g. `"atomic_mass": { "value": 55.845, "unit": "u" }`,
units `u`, `K`, `g/cm³`); an unmeasured one collapses to `null`. Dimensionless
fields (electronegativity, group, oxidation states, …) stay bare numbers.

---

## Sequence diagram — a lookup

What happens for `pt get --symbol Fe`:

```mermaid
sequenceDiagram
    actor User
    participant CLI as pt (CLI)
    participant Table as PeriodicTable
    participant Repo as ElementRepository
    participant FS as Filesystem
    participant Domain as pt-domain

    User->>CLI: pt get --symbol Fe
    CLI->>Table: load(data_dir)
    Table->>Repo: load_from_dir(dir)
    Repo->>FS: read *.yaml
    FS-->>Repo: file contents
    Repo->>Repo: parse → validate → index (once)
    Repo-->>Table: repository
    Table-->>CLI: PeriodicTable

    CLI->>Table: by_symbol("Fe")
    Table->>Repo: get_by_symbol("fe")
    Repo-->>Table: &Element
    Table-->>CLI: ElementView

    CLI->>+Domain: group(26), period(26), electron_configuration(26), …
    Domain-->>-CLI: computed values
    CLI-->>User: formatted output (text / json / yaml)
```

---

## Data flow

How a single element's data moves from disk to output:

```mermaid
flowchart LR
    A[("name.yaml")] -->|serde_yaml_ng| B[RawElement DTO]
    B -->|validate range / abundance + convert| C["domain::Element<br/>(stored fields)"]
    C -->|sort + index by number / symbol / name| D[ElementRepository]
    D -->|borrow on lookup| E[ElementView]
    E -->|stored fields| G[Text / JSON / YAML]
    E -->|compute on demand via pt-domain| F["config · group · period · block ·<br/>category · oxidation states · state_at"]
    F --> G
```

Validation performed by `pt-data` while loading: atomic number in `1..=118`;
unique atomic number, symbol, and name; isotope abundances sum to ≈ 1.0 (±0.01);
required fields present.

---

## How electron configuration is computed

Electron configuration is **not** stored in the YAML — it is derived from the
atomic number `Z` alone, in `pt-domain/src/config.rs`
(`electron_configuration(z)`). The arrangement follows three physical rules plus
a correction table.

### 1. Aufbau principle — fill order (Madelung rule)

Subshells fill from lowest energy upward. Energy is ordered by `n + l` (the
**Madelung rule**), with ties broken by the lower principal number `n`. That
gives a fixed sequence that the code hardcodes as `MADELUNG_ORDER`:

```
1s → 2s → 2p → 3s → 3p → 4s → 3d → 4p → 5s → 4d → 5p →
6s → 4f → 5d → 6p → 7s → 5f → 6d → 7p
```

Note that `4s` comes **before** `3d`: `4s` has `n+l = 4+0 = 4` while `3d` has
`3+2 = 5`, so the lower sum fills first.

### 2. Pauli exclusion — subshell capacity

Each subshell `l` holds at most `2·(2l + 1)` electrons (`Subshell::capacity`):

| subshell | `l` | orbitals `2l+1` | capacity `2(2l+1)` |
|----------|-----|-----------------|--------------------|
| s        | 0   | 1               | 2                  |
| p        | 1   | 3               | 6                  |
| d        | 2   | 5               | 10                 |
| f        | 3   | 7               | 14                 |

`aufbau_fill(z)` walks the Madelung order and drops `min(remaining, capacity)`
electrons into each subshell until all `Z` electrons are placed.

### 3. Hund's rule — counting unpaired electrons

Within one subshell, electrons singly occupy each orbital before any pairing
begins. So for a subshell with `orbitals = 2l+1` slots holding `e` electrons
(`ElectronConfiguration::unpaired_electrons`):

```
unpaired = e                 if e ≤ orbitals   (all still single)
unpaired = 2·orbitals − e    if e >  orbitals   (the surplus pairs up)
```

### 4. Ground-state anomalies — the correction table

The naive Aufbau fill is wrong for 19 real elements where a near-full or
half-full d/f subshell is energetically favorable (e.g. chromium, copper, the
lanthanide/actinide boundary). `anomalies(z)` lists the corrected absolute
occupancies; `electron_configuration` overwrites the affected orbitals after the
naive fill, and any orbital forced to `0` is dropped entirely (this is how
palladium loses its `5s` electrons).

### Worked example — iron (`Z = 26`)

Fill in Madelung order, subtracting each subshell's electrons from the 26 total:

| step | subshell | added | remaining |
|------|----------|-------|-----------|
| 1    | 1s       | 2     | 24        |
| 2    | 2s       | 2     | 22        |
| 3    | 2p       | 6     | 16        |
| 4    | 3s       | 2     | 14        |
| 5    | 3p       | 6     | 8         |
| 6    | 4s       | 2     | 6         |
| 7    | 3d       | 6     | 0 → stop  |

Iron is not in the anomaly table, so the fill stands. The orbitals are *stored*
in fill order (`… 4s² 3d⁶`), but are rendered in standard `(n, l)` order — the
CLI prints each subshell's occupancy as a superscript:

```
1s² 2s² 2p⁶ 3s² 3p⁶ 3d⁶ 4s²
```

Its `3d⁶` subshell (6 electrons in 5 orbitals) has `2·5 − 6 = 4` unpaired
electrons.

### Worked example — chromium (`Z = 24`), an anomaly

Naive Aufbau would yield `… 3d⁴ 4s²`. The anomaly table overrides this to
`3d⁵, 4s¹` (a half-filled `3d` is favored), producing:

```
1s² 2s² 2p⁶ 3s² 3p⁶ 3d⁵ 4s¹
```

```mermaid
flowchart LR
    Z["atomic number Z"] --> AUF["aufbau_fill(z)<br/>Madelung order + Pauli capacity"]
    AUF --> ANOM{"anomalies(z)?"}
    ANOM -->|no| OUT["ElectronConfiguration"]
    ANOM -->|yes| FIX["overwrite occupancies<br/>drop any → 0"]
    FIX --> OUT
    OUT -->|sorted by n, l; CLI superscripts occupancy| STR["1s² 2s² 2p⁶ …"]
```

---

## Property reference

| Property | Source | Unit | Notes |
|----------|--------|------|-------|
| atomic_number, name, symbol | stored | — | identity / lookup keys |
| atomic_mass | stored | u (unified atomic mass unit) | standard atomic weight |
| mass_number | stored | — | nucleon count of the representative isotope |
| melting_point, boiling_point | stored | K (kelvin) | `null` when unmeasured |
| density | stored | g/cm³ | `null` when unmeasured |
| electronegativity | stored | — (Pauling scale) | `null` for some elements |
| state | stored | — | `solid` / `liquid` / `gas` at STP |
| isotopes | stored | relative_mass: u; abundance: mole fraction | per isotope: mass number, relative mass, abundance |
| discovery_year, discoverer | stored | — (calendar year) | `null` for ancient/synthetic elements |
| electron_configuration | **computed** | — | Aufbau (Madelung order) + Pauli capacities + ~19-element anomaly table |
| group, period, block | **computed** | — | from the configuration (f-block → group 3 by convention) |
| category | **computed** | — | heuristic (alkali metal, noble gas, metalloid, …) |
| oxidation_states | **computed** | — | heuristic, group-based signed charges |
| computed_atomic_mass | **computed** | u | abundance-weighted isotope mean |
| state_at(T) | **computed** | input T in K | from stored melting/boiling points |

### Caveats

- `category` and `oxidation_states` are best-effort heuristics, **not**
  authoritative chemical references.
- `block`/`period` are derived from the *naive* Aufbau fill, so ground-state
  anomalies (e.g. palladium dropping its 5s electrons) do not shift an element's
  row or block.
- The 118-element dataset uses best-effort standard reference values; synthetic
  superheavy elements omit unmeasured properties, and some elements use a single
  representative isotope.

---

## YAML schema

Each `data/elements/<name>.yaml` looks like:

```yaml
atomic_number: 1
name: Hydrogen
symbol: H
atomic_mass: 1.008
mass_number: 1
melting_point: 13.99          # Kelvin (optional)
boiling_point: 20.271         # Kelvin (optional)
density: 0.00008988           # g/cm³ (optional)
electronegativity: 2.20       # Pauling scale (optional)
state: gas                    # solid | liquid | gas
discovery_year: 1766          # optional
discoverer: Henry Cavendish   # optional
isotopes:                     # optional; abundances should sum to ≈ 1.0
  - { mass_number: 1, relative_mass: 1.007825, abundance: 0.999885 }
  - { mass_number: 2, relative_mass: 2.014102, abundance: 0.000115 }
```

To add or correct an element, edit (or create) its YAML file and reload — no code
changes required.

---

## Documentation

- Design spec: `docs/superpowers/specs/2026-05-29-periodic-table-design.md`
- Implementation plan: `docs/superpowers/plans/2026-05-29-periodic-table.md`
- API docs: `cargo doc --workspace --no-deps --open`
