# Periodic Table Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust workspace library that serves chemical-element properties from per-element YAML files, separating stored empirical data from physically-derivable computed properties, plus a CLI for queries.

**Architecture:** Four crates in a Cargo workspace with one-directional dependencies — `pt-cli → pt-services → pt-data → pt-domain`. `pt-domain` holds pure value types and stateless calculations (no I/O, no serde). `pt-data` reads/validates YAML into domain types and builds an indexed repository (loaded once). `pt-services` exposes the query API merging stored + computed properties. `pt-cli` is a thin binary.

**Tech Stack:** Rust (edition 2021), `serde` + `serde_yaml_ng` (YAML), `thiserror` (errors), `clap` (CLI), `serde_json` (CLI JSON output); dev: `tempfile`, `assert_cmd`, `predicates`.

**Reference spec:** `docs/superpowers/specs/2026-05-29-periodic-table-design.md`

---

## File Structure

```
periodic-table/
├── Cargo.toml                              # workspace manifest + shared deps
├── data/elements/*.yaml                    # element data (source of truth)
├── crates/
│   ├── pt-domain/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                      # module decls + re-exports
│   │       ├── error.rs                    # DomainError
│   │       ├── element.rs                  # Element, Isotope, StateOfMatter
│   │       ├── config.rs                   # Subshell, Orbital, ElectronConfiguration, electron_configuration
│   │       ├── classification.rs           # Block, Category, OxidationStates, block/period/group/category/oxidation_states
│   │       └── calc.rs                     # atomic_mass_from_isotopes, isotope_mass_matches, state_at
│   ├── pt-data/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                       # module decls + re-exports
│   │       ├── error.rs                     # DataError
│   │       ├── raw.rs                       # serde DTOs + conversion to domain types
│   │       ├── parse.rs                     # parse_element_file
│   │       └── repository.rs                # ElementRepository, init_global, global
│   ├── pt-services/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                        # re-exports
│   │   │   ├── error.rs                      # ServiceError
│   │   │   ├── view.rs                       # ElementView
│   │   │   └── table.rs                      # PeriodicTable
│   │   └── tests/lookup.rs                   # integration tests
│   └── pt-cli/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs                       # clap CLI + dispatch
│       │   └── output.rs                     # ElementOutput (Serialize) + text printer
│       └── tests/cli.rs                      # integration tests
└── docs/superpowers/{specs,plans}/...
```

Each domain calculation lives in a focused module so it can be read and tested in isolation. Tests live inline (`#[cfg(test)] mod tests`) for unit-level coverage; `tests/` directories hold integration tests for the service and CLI layers.

---

## Task 1: Workspace scaffolding

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/pt-domain/Cargo.toml`, `crates/pt-domain/src/lib.rs`
- Create: `crates/pt-data/Cargo.toml`, `crates/pt-data/src/lib.rs`
- Create: `crates/pt-services/Cargo.toml`, `crates/pt-services/src/lib.rs`
- Create: `crates/pt-cli/Cargo.toml`, `crates/pt-cli/src/main.rs`

- [ ] **Step 1: Create the workspace manifest**

Create `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/pt-domain", "crates/pt-data", "crates/pt-services", "crates/pt-cli"]

[workspace.package]
edition = "2021"
version = "0.1.0"

[workspace.dependencies]
pt-domain = { path = "crates/pt-domain" }
pt-data = { path = "crates/pt-data" }
pt-services = { path = "crates/pt-services" }
serde = { version = "1", features = ["derive"] }
serde_yaml_ng = "0.10"
serde_json = "1"
thiserror = "2"
clap = { version = "4", features = ["derive"] }
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 2: Create each crate manifest**

`crates/pt-domain/Cargo.toml`:

```toml
[package]
name = "pt-domain"
edition.workspace = true
version.workspace = true

[dependencies]
thiserror = { workspace = true }
```

`crates/pt-data/Cargo.toml`:

```toml
[package]
name = "pt-data"
edition.workspace = true
version.workspace = true

[dependencies]
pt-domain = { workspace = true }
serde = { workspace = true }
serde_yaml_ng = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

`crates/pt-services/Cargo.toml`:

```toml
[package]
name = "pt-services"
edition.workspace = true
version.workspace = true

[dependencies]
pt-domain = { workspace = true }
pt-data = { workspace = true }
thiserror = { workspace = true }
```

`crates/pt-cli/Cargo.toml`:

```toml
[package]
name = "pt-cli"
edition.workspace = true
version.workspace = true

[[bin]]
name = "pt"
path = "src/main.rs"

[dependencies]
pt-domain = { workspace = true }
pt-services = { workspace = true }
clap = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml_ng = { workspace = true }

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
```

- [ ] **Step 3: Create placeholder crate roots**

`crates/pt-domain/src/lib.rs`:

```rust
//! Pure value types and stateless calculations for the periodic table.
```

`crates/pt-data/src/lib.rs`:

```rust
//! YAML loading and the in-memory element repository.
```

`crates/pt-services/src/lib.rs`:

```rust
//! Query API exposing stored and computed element properties.
```

`crates/pt-cli/src/main.rs`:

```rust
fn main() {
    println!("pt cli");
}
```

- [ ] **Step 4: Verify the workspace builds**

Run: `cargo build --workspace`
Expected: compiles successfully (warnings about unused code are acceptable at this stage).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/
git commit -m "chore: scaffold periodic-table workspace with four crates"
```

---

## Task 2: pt-domain — stored types and error

**Files:**
- Create: `crates/pt-domain/src/error.rs`
- Create: `crates/pt-domain/src/element.rs`
- Modify: `crates/pt-domain/src/lib.rs`

- [ ] **Step 1: Write the error type**

Create `crates/pt-domain/src/error.rs`:

```rust
//! Domain calculation errors.

/// Error returned by domain calculations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    /// The atomic number is outside the supported range `1..=118`.
    #[error("invalid atomic number: {0} (must be 1..=118)")]
    InvalidAtomicNumber(u16),
}
```

- [ ] **Step 2: Write the stored value types**

Create `crates/pt-domain/src/element.rs`:

```rust
//! Stored element data loaded from YAML.

/// The physical state of an element at standard temperature and pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateOfMatter {
    Solid,
    Liquid,
    Gas,
}

/// A single naturally-occurring (or representative) isotope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Isotope {
    /// Nucleon count (protons + neutrons).
    pub mass_number: u16,
    /// Relative atomic mass of this isotope, in unified atomic mass units (u).
    pub relative_mass: f64,
    /// Fractional natural abundance in `0.0..=1.0`.
    pub abundance: f64,
}

/// All stored properties of a chemical element.
///
/// Empirical fields that may be unmeasured for synthetic elements are `Option`.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub atomic_number: u8,
    pub name: String,
    pub symbol: String,
    /// Standard atomic weight, in u.
    pub atomic_mass: f64,
    /// Mass number of the most abundant (or most stable) isotope.
    pub mass_number: u16,
    /// Melting point in Kelvin.
    pub melting_point: Option<f64>,
    /// Boiling point in Kelvin.
    pub boiling_point: Option<f64>,
    /// Density in g/cm³ (at STP).
    pub density: Option<f64>,
    /// Electronegativity on the Pauling scale.
    pub electronegativity: Option<f64>,
    /// State at STP (298.15 K).
    pub state: StateOfMatter,
    pub discovery_year: Option<i32>,
    pub discoverer: Option<String>,
    pub isotopes: Vec<Isotope>,
}
```

- [ ] **Step 3: Wire modules and re-exports**

Replace `crates/pt-domain/src/lib.rs` with:

```rust
//! Pure value types and stateless calculations for the periodic table.

pub mod element;
pub mod error;

pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
```

- [ ] **Step 4: Add a construction smoke test**

Append to `crates/pt-domain/src/element.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_an_element() {
        let e = Element {
            atomic_number: 1,
            name: "Hydrogen".into(),
            symbol: "H".into(),
            atomic_mass: 1.008,
            mass_number: 1,
            melting_point: Some(13.99),
            boiling_point: Some(20.271),
            density: Some(0.00008988),
            electronegativity: Some(2.20),
            state: StateOfMatter::Gas,
            discovery_year: Some(1766),
            discoverer: Some("Henry Cavendish".into()),
            isotopes: vec![Isotope { mass_number: 1, relative_mass: 1.007825, abundance: 0.999885 }],
        };
        assert_eq!(e.symbol, "H");
        assert_eq!(e.state, StateOfMatter::Gas);
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pt-domain`
Expected: PASS (1 test).

- [ ] **Step 6: Commit**

```bash
git add crates/pt-domain/src/
git commit -m "feat(domain): add Element, Isotope, StateOfMatter, and DomainError"
```

---

## Task 3: pt-domain — electron configuration

**Files:**
- Create: `crates/pt-domain/src/config.rs`
- Modify: `crates/pt-domain/src/lib.rs`

- [ ] **Step 1: Write failing tests for the configuration**

Create `crates/pt-domain/src/config.rs`:

```rust
//! Electron configuration via the Aufbau principle (Madelung ordering),
//! subshell capacities (Pauli exclusion), and Hund's rule for unpaired counts.

use crate::error::DomainError;
use std::fmt;

/// Azimuthal (orbital-shape) subshell type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subshell {
    S,
    P,
    D,
    F,
}

impl Subshell {
    /// Azimuthal quantum number `l`.
    pub fn azimuthal(self) -> u8 {
        match self {
            Subshell::S => 0,
            Subshell::P => 1,
            Subshell::D => 2,
            Subshell::F => 3,
        }
    }

    /// Maximum electrons the subshell can hold: `2 * (2l + 1)`.
    pub fn capacity(self) -> u8 {
        match self {
            Subshell::S => 2,
            Subshell::P => 6,
            Subshell::D => 10,
            Subshell::F => 14,
        }
    }

    /// Number of orbitals in the subshell: `2l + 1`.
    pub fn orbital_count(self) -> u8 {
        2 * self.azimuthal() + 1
    }

    /// Spectroscopic label.
    pub fn label(self) -> char {
        match self {
            Subshell::S => 's',
            Subshell::P => 'p',
            Subshell::D => 'd',
            Subshell::F => 'f',
        }
    }
}

/// An occupied subshell within a configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orbital {
    pub n: u8,
    pub subshell: Subshell,
    pub electrons: u8,
}

/// A full electron configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronConfiguration {
    /// Occupied subshells, stored in Aufbau fill order.
    pub orbitals: Vec<Orbital>,
}

impl ElectronConfiguration {
    /// Total number of unpaired electrons (Hund's rule).
    pub fn unpaired_electrons(&self) -> u8 {
        self.orbitals
            .iter()
            .map(|o| {
                let half = o.subshell.orbital_count();
                if o.electrons <= half {
                    o.electrons
                } else {
                    2 * half - o.electrons
                }
            })
            .sum()
    }

    /// Electrons in a specific `(n, subshell)`, or 0 if absent.
    pub fn electrons_in(&self, n: u8, subshell: Subshell) -> u8 {
        self.orbitals
            .iter()
            .find(|o| o.n == n && o.subshell == subshell)
            .map_or(0, |o| o.electrons)
    }
}

impl fmt::Display for ElectronConfiguration {
    /// Renders in standard (n, l) order, e.g. "1s2 2s2 2p6 3s2 3p6 3d6 4s2".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ordered = self.orbitals.clone();
        ordered.sort_by_key(|o| (o.n, o.subshell.azimuthal()));
        let parts: Vec<String> = ordered
            .iter()
            .map(|o| format!("{}{}{}", o.n, o.subshell.label(), o.electrons))
            .collect();
        write!(f, "{}", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_atomic_number_errors() {
        assert_eq!(electron_configuration(0), Err(DomainError::InvalidAtomicNumber(0)));
        assert_eq!(electron_configuration(119), Err(DomainError::InvalidAtomicNumber(119)));
    }

    #[test]
    fn hydrogen_and_helium() {
        assert_eq!(electron_configuration(1).unwrap().to_string(), "1s1");
        assert_eq!(electron_configuration(2).unwrap().to_string(), "1s2");
    }

    #[test]
    fn iron_uses_standard_order() {
        assert_eq!(
            electron_configuration(26).unwrap().to_string(),
            "1s2 2s2 2p6 3s2 3p6 3d6 4s2"
        );
    }

    #[test]
    fn neon_is_filled() {
        assert_eq!(electron_configuration(10).unwrap().to_string(), "1s2 2s2 2p6");
    }

    #[test]
    fn chromium_anomaly() {
        // Naive Aufbau would give 3d4 4s2; reality is 3d5 4s1.
        assert_eq!(
            electron_configuration(24).unwrap().to_string(),
            "1s2 2s2 2p6 3s2 3p6 3d5 4s1"
        );
    }

    #[test]
    fn copper_anomaly() {
        assert_eq!(
            electron_configuration(29).unwrap().to_string(),
            "1s2 2s2 2p6 3s2 3p6 3d10 4s1"
        );
    }

    #[test]
    fn palladium_anomaly_drops_5s() {
        // Reality: [Kr] 4d10 (no 5s electrons).
        assert_eq!(
            electron_configuration(46).unwrap().to_string(),
            "1s2 2s2 2p6 3s2 3p6 3d10 4s2 4p6 4d10"
        );
    }

    #[test]
    fn oxygen_has_two_unpaired() {
        assert_eq!(electron_configuration(8).unwrap().unpaired_electrons(), 2);
    }

    #[test]
    fn oganesson_fills_to_118() {
        let config = electron_configuration(118).unwrap();
        let total: u16 = config.orbitals.iter().map(|o| o.electrons as u16).sum();
        assert_eq!(total, 118);
        assert_eq!(config.electrons_in(7, Subshell::P), 6);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pt-domain config`
Expected: FAIL — `electron_configuration` is not defined.

- [ ] **Step 3: Implement the Aufbau fill, anomaly table, and public function**

Append to `crates/pt-domain/src/config.rs` (above the `#[cfg(test)]` module):

```rust
/// Madelung (n + l) fill order, covering atomic numbers 1..=118.
const MADELUNG_ORDER: [(u8, Subshell); 19] = [
    (1, Subshell::S),
    (2, Subshell::S),
    (2, Subshell::P),
    (3, Subshell::S),
    (3, Subshell::P),
    (4, Subshell::S),
    (3, Subshell::D),
    (4, Subshell::P),
    (5, Subshell::S),
    (4, Subshell::D),
    (5, Subshell::P),
    (6, Subshell::S),
    (4, Subshell::F),
    (5, Subshell::D),
    (6, Subshell::P),
    (7, Subshell::S),
    (5, Subshell::F),
    (6, Subshell::D),
    (7, Subshell::P),
];

fn validate_z(z: u8) -> Result<(), DomainError> {
    if (1..=118).contains(&z) {
        Ok(())
    } else {
        Err(DomainError::InvalidAtomicNumber(z as u16))
    }
}

/// Naive Aufbau fill (before anomaly corrections), in fill order.
pub(crate) fn aufbau_fill(z: u8) -> Vec<Orbital> {
    let mut remaining = z as u16;
    let mut orbitals = Vec::new();
    for &(n, subshell) in MADELUNG_ORDER.iter() {
        if remaining == 0 {
            break;
        }
        let cap = subshell.capacity() as u16;
        let electrons = remaining.min(cap);
        orbitals.push(Orbital { n, subshell, electrons: electrons as u8 });
        remaining -= electrons;
    }
    orbitals
}

/// Known ground-state anomalies: absolute occupancies for the orbitals that
/// deviate from naive Aufbau. Orbitals set to 0 are dropped after applying.
fn anomalies(z: u8) -> Option<&'static [(u8, Subshell, u8)]> {
    use Subshell::{D, F, S};
    match z {
        24 => Some(&[(3, D, 5), (4, S, 1)]),  // Cr
        29 => Some(&[(3, D, 10), (4, S, 1)]), // Cu
        41 => Some(&[(4, D, 4), (5, S, 1)]),  // Nb
        42 => Some(&[(4, D, 5), (5, S, 1)]),  // Mo
        44 => Some(&[(4, D, 7), (5, S, 1)]),  // Ru
        45 => Some(&[(4, D, 8), (5, S, 1)]),  // Rh
        46 => Some(&[(4, D, 10), (5, S, 0)]), // Pd
        47 => Some(&[(4, D, 10), (5, S, 1)]), // Ag
        57 => Some(&[(4, F, 0), (5, D, 1)]),  // La
        58 => Some(&[(4, F, 1), (5, D, 1)]),  // Ce
        64 => Some(&[(4, F, 7), (5, D, 1)]),  // Gd
        78 => Some(&[(5, D, 9), (6, S, 1)]),  // Pt
        79 => Some(&[(5, D, 10), (6, S, 1)]), // Au
        89 => Some(&[(5, F, 0), (6, D, 1)]),  // Ac
        90 => Some(&[(5, F, 0), (6, D, 2)]),  // Th
        91 => Some(&[(5, F, 2), (6, D, 1)]),  // Pa
        92 => Some(&[(5, F, 3), (6, D, 1)]),  // U
        93 => Some(&[(5, F, 4), (6, D, 1)]),  // Np
        96 => Some(&[(5, F, 7), (6, D, 1)]),  // Cm
        _ => None,
    }
}

/// Computes the ground-state electron configuration for atomic number `z`.
pub fn electron_configuration(z: u8) -> Result<ElectronConfiguration, DomainError> {
    validate_z(z)?;
    let mut orbitals = aufbau_fill(z);
    if let Some(overrides) = anomalies(z) {
        for &(n, subshell, electrons) in overrides {
            if let Some(orbital) = orbitals.iter_mut().find(|o| o.n == n && o.subshell == subshell) {
                orbital.electrons = electrons;
            } else {
                orbitals.push(Orbital { n, subshell, electrons });
            }
        }
        orbitals.retain(|o| o.electrons > 0);
    }
    Ok(ElectronConfiguration { orbitals })
}
```

- [ ] **Step 4: Wire the module**

Update `crates/pt-domain/src/lib.rs` to:

```rust
//! Pure value types and stateless calculations for the periodic table.

pub mod config;
pub mod element;
pub mod error;

pub use config::{electron_configuration, ElectronConfiguration, Orbital, Subshell};
pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pt-domain config`
Expected: PASS (all configuration tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pt-domain/src/config.rs crates/pt-domain/src/lib.rs
git commit -m "feat(domain): compute electron configurations with anomaly table"
```

---

## Task 4: pt-domain — block, period, group

**Files:**
- Create: `crates/pt-domain/src/classification.rs`
- Modify: `crates/pt-domain/src/lib.rs`

- [ ] **Step 1: Write failing tests for block/period/group**

Create `crates/pt-domain/src/classification.rs`:

```rust
//! Periodic-table placement and chemistry heuristics derived from the
//! electron configuration.

use crate::config::{aufbau_fill, electron_configuration, Subshell};
use crate::error::DomainError;

/// The block of the periodic table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    S,
    P,
    D,
    F,
}

impl From<Subshell> for Block {
    fn from(s: Subshell) -> Self {
        match s {
            Subshell::S => Block::S,
            Subshell::P => Block::P,
            Subshell::D => Block::D,
            Subshell::F => Block::F,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks() {
        assert_eq!(block(2).unwrap(), Block::S); // He: naive last orbital is 1s
        assert_eq!(block(11).unwrap(), Block::S); // Na
        assert_eq!(block(26).unwrap(), Block::D); // Fe
        assert_eq!(block(9).unwrap(), Block::P); // F
        assert_eq!(block(60).unwrap(), Block::F); // Nd
    }

    #[test]
    fn periods() {
        assert_eq!(period(1).unwrap(), 1); // H
        assert_eq!(period(11).unwrap(), 3); // Na
        assert_eq!(period(26).unwrap(), 4); // Fe
        assert_eq!(period(46).unwrap(), 5); // Pd (period from naive fill keeps 5s)
        assert_eq!(period(60).unwrap(), 6); // Nd
    }

    #[test]
    fn groups_main_block() {
        assert_eq!(group(1).unwrap(), 1); // H
        assert_eq!(group(2).unwrap(), 18); // He
        assert_eq!(group(3).unwrap(), 1); // Li
        assert_eq!(group(4).unwrap(), 2); // Be
        assert_eq!(group(8).unwrap(), 16); // O
        assert_eq!(group(9).unwrap(), 17); // F
        assert_eq!(group(10).unwrap(), 18); // Ne
        assert_eq!(group(5).unwrap(), 13); // B
    }

    #[test]
    fn groups_transition_block() {
        assert_eq!(group(21).unwrap(), 3); // Sc
        assert_eq!(group(26).unwrap(), 8); // Fe
        assert_eq!(group(30).unwrap(), 12); // Zn
        assert_eq!(group(24).unwrap(), 6); // Cr (anomaly)
        assert_eq!(group(29).unwrap(), 11); // Cu (anomaly)
        assert_eq!(group(46).unwrap(), 10); // Pd (anomaly, 5s dropped)
    }

    #[test]
    fn groups_f_block_uses_convention() {
        assert_eq!(group(60).unwrap(), 3); // Nd
        assert_eq!(group(92).unwrap(), 3); // U
    }

    #[test]
    fn invalid_z_errors() {
        assert_eq!(block(0), Err(DomainError::InvalidAtomicNumber(0)));
        assert_eq!(period(200), Err(DomainError::InvalidAtomicNumber(200)));
        assert_eq!(group(0), Err(DomainError::InvalidAtomicNumber(0)));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pt-domain classification`
Expected: FAIL — `block`, `period`, `group` not defined.

- [ ] **Step 3: Implement block, period, group**

Insert into `crates/pt-domain/src/classification.rs`, before the `#[cfg(test)]` module:

```rust
fn validate_z(z: u8) -> Result<(), DomainError> {
    if (1..=118).contains(&z) {
        Ok(())
    } else {
        Err(DomainError::InvalidAtomicNumber(z as u16))
    }
}

/// The block, from the subshell of the differentiating electron (naive Aufbau).
pub fn block(z: u8) -> Result<Block, DomainError> {
    validate_z(z)?;
    let fill = aufbau_fill(z);
    let last = fill.last().expect("z >= 1 yields at least one orbital");
    Ok(Block::from(last.subshell))
}

/// The period (row), from the highest principal quantum number in the naive fill.
pub fn period(z: u8) -> Result<u8, DomainError> {
    validate_z(z)?;
    let fill = aufbau_fill(z);
    Ok(fill.iter().map(|o| o.n).max().expect("non-empty fill"))
}

/// The group (column) 1..=18. f-block elements are assigned group 3 by convention.
pub fn group(z: u8) -> Result<u8, DomainError> {
    validate_z(z)?;
    if z == 1 {
        return Ok(1); // Hydrogen
    }
    if z == 2 {
        return Ok(18); // Helium
    }
    let config = electron_configuration(z)?;
    let p = period(z)?;
    let g = match block(z)? {
        Block::S => config.electrons_in(p, Subshell::S),
        Block::P => 12 + config.electrons_in(p, Subshell::P),
        Block::D => config.electrons_in(p - 1, Subshell::D) + config.electrons_in(p, Subshell::S),
        Block::F => 3,
    };
    Ok(g)
}
```

- [ ] **Step 4: Wire the module**

Update `crates/pt-domain/src/lib.rs` to add the module and re-exports:

```rust
//! Pure value types and stateless calculations for the periodic table.

pub mod classification;
pub mod config;
pub mod element;
pub mod error;

pub use classification::{block, group, period, Block};
pub use config::{electron_configuration, ElectronConfiguration, Orbital, Subshell};
pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pt-domain classification`
Expected: PASS (all block/period/group tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pt-domain/src/classification.rs crates/pt-domain/src/lib.rs
git commit -m "feat(domain): derive block, period, and group from configuration"
```

---

## Task 5: pt-domain — category

**Files:**
- Modify: `crates/pt-domain/src/classification.rs`
- Modify: `crates/pt-domain/src/lib.rs`

- [ ] **Step 1: Write failing tests for category**

Add to the `#[cfg(test)] mod tests` in `crates/pt-domain/src/classification.rs`:

```rust
    #[test]
    fn categories() {
        assert_eq!(category(1).unwrap(), Category::ReactiveNonmetal); // H
        assert_eq!(category(2).unwrap(), Category::NobleGas); // He
        assert_eq!(category(11).unwrap(), Category::AlkaliMetal); // Na
        assert_eq!(category(12).unwrap(), Category::AlkalineEarthMetal); // Mg
        assert_eq!(category(26).unwrap(), Category::TransitionMetal); // Fe
        assert_eq!(category(5).unwrap(), Category::Metalloid); // B
        assert_eq!(category(17).unwrap(), Category::Halogen); // Cl
        assert_eq!(category(10).unwrap(), Category::NobleGas); // Ne
        assert_eq!(category(13).unwrap(), Category::PostTransitionMetal); // Al
        assert_eq!(category(8).unwrap(), Category::ReactiveNonmetal); // O
        assert_eq!(category(60).unwrap(), Category::Lanthanide); // Nd
        assert_eq!(category(92).unwrap(), Category::Actinide); // U
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pt-domain classification::tests::categories`
Expected: FAIL — `Category` / `category` not defined.

- [ ] **Step 3: Implement Category and the heuristic**

Add to `crates/pt-domain/src/classification.rs`, after the `Block` definitions:

```rust
/// A heuristic element category. Not authoritative — see the spec caveats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    AlkaliMetal,
    AlkalineEarthMetal,
    TransitionMetal,
    PostTransitionMetal,
    Metalloid,
    ReactiveNonmetal,
    NobleGas,
    Halogen,
    Lanthanide,
    Actinide,
}

const METALLOIDS: [u8; 7] = [5, 14, 32, 33, 51, 52, 85]; // B Si Ge As Sb Te At
const POST_TRANSITION: [u8; 12] = [13, 31, 49, 50, 81, 82, 83, 84, 113, 114, 115, 116];

/// Best-effort element category derived from group, block, and atomic number.
pub fn category(z: u8) -> Result<Category, DomainError> {
    validate_z(z)?;
    let g = group(z)?;
    let b = block(z)?;
    let c = if (57..=71).contains(&z) {
        Category::Lanthanide
    } else if (89..=103).contains(&z) {
        Category::Actinide
    } else if g == 18 {
        Category::NobleGas
    } else if g == 1 && z != 1 {
        Category::AlkaliMetal
    } else if g == 2 {
        Category::AlkalineEarthMetal
    } else if g == 17 {
        Category::Halogen
    } else if b == Block::D {
        Category::TransitionMetal
    } else if METALLOIDS.contains(&z) {
        Category::Metalloid
    } else if POST_TRANSITION.contains(&z) {
        Category::PostTransitionMetal
    } else {
        Category::ReactiveNonmetal
    };
    Ok(c)
}
```

- [ ] **Step 4: Wire the re-export**

Update the `classification` re-export line in `crates/pt-domain/src/lib.rs` to:

```rust
pub use classification::{block, category, group, period, Block, Category};
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pt-domain classification`
Expected: PASS (categories plus the earlier block/period/group tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pt-domain/src/classification.rs crates/pt-domain/src/lib.rs
git commit -m "feat(domain): add heuristic element category"
```

---

## Task 6: pt-domain — oxidation states, isotope mass, state-at-temperature

**Files:**
- Modify: `crates/pt-domain/src/classification.rs` (oxidation states)
- Create: `crates/pt-domain/src/calc.rs`
- Modify: `crates/pt-domain/src/lib.rs`

- [ ] **Step 1: Write failing tests for oxidation states**

Add to the `#[cfg(test)] mod tests` in `crates/pt-domain/src/classification.rs`:

```rust
    #[test]
    fn oxidation_states_main_group() {
        assert_eq!(oxidation_states(11).unwrap().0, vec![1]); // Na
        assert_eq!(oxidation_states(12).unwrap().0, vec![2]); // Mg
        assert_eq!(oxidation_states(8).unwrap().0, vec![-2]); // O
        assert_eq!(oxidation_states(9).unwrap().0, vec![-1]); // F
        assert_eq!(oxidation_states(10).unwrap().0, vec![0]); // Ne
        assert_eq!(oxidation_states(5).unwrap().0, vec![3]); // B
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pt-domain classification::tests::oxidation_states_main_group`
Expected: FAIL — `OxidationStates` / `oxidation_states` not defined.

- [ ] **Step 3: Implement oxidation states**

Add to `crates/pt-domain/src/classification.rs`, after the `Category` definitions:

```rust
/// Common oxidation states (heuristic, group-based — not authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxidationStates(pub Vec<i8>);

/// Best-effort common oxidation states derived from the group.
pub fn oxidation_states(z: u8) -> Result<OxidationStates, DomainError> {
    validate_z(z)?;
    let states = match group(z)? {
        1 => vec![1],
        2 => vec![2],
        13 => vec![3],
        14 => vec![-4, 4],
        15 => vec![-3, 3, 5],
        16 => vec![-2],
        17 => vec![-1],
        18 => vec![0],
        3..=12 => vec![2, 3],
        _ => vec![],
    };
    Ok(OxidationStates(states))
}
```

- [ ] **Step 4: Write failing tests for calc.rs**

Create `crates/pt-domain/src/calc.rs`:

```rust
//! Calculations over stored element data.

use crate::element::{Element, Isotope, StateOfMatter};

#[cfg(test)]
mod tests {
    use super::*;

    fn chlorine_isotopes() -> Vec<Isotope> {
        vec![
            Isotope { mass_number: 35, relative_mass: 34.968853, abundance: 0.7576 },
            Isotope { mass_number: 37, relative_mass: 36.965903, abundance: 0.2424 },
        ]
    }

    #[test]
    fn weighted_mass_of_chlorine() {
        let mass = atomic_mass_from_isotopes(&chlorine_isotopes()).unwrap();
        assert!((mass - 35.45).abs() < 0.01, "got {mass}");
    }

    #[test]
    fn no_isotopes_yields_none() {
        assert_eq!(atomic_mass_from_isotopes(&[]), None);
    }

    #[test]
    fn isotope_validation_matches_stored() {
        let element = sample(35.45, chlorine_isotopes());
        assert!(isotope_mass_matches(&element, 0.01));
    }

    #[test]
    fn state_transitions_with_temperature() {
        // Iron: mp 1811 K, bp 3134 K.
        let iron = sample_with_points(Some(1811.0), Some(3134.0));
        assert_eq!(state_at(&iron, 300.0), Some(StateOfMatter::Solid));
        assert_eq!(state_at(&iron, 2000.0), Some(StateOfMatter::Liquid));
        assert_eq!(state_at(&iron, 4000.0), Some(StateOfMatter::Gas));
    }

    #[test]
    fn state_at_is_none_without_points() {
        let unknown = sample_with_points(None, None);
        assert_eq!(state_at(&unknown, 300.0), None);
    }

    fn sample(atomic_mass: f64, isotopes: Vec<Isotope>) -> Element {
        Element {
            atomic_number: 17,
            name: "Test".into(),
            symbol: "Ts".into(),
            atomic_mass,
            mass_number: 35,
            melting_point: None,
            boiling_point: None,
            density: None,
            electronegativity: None,
            state: StateOfMatter::Gas,
            discovery_year: None,
            discoverer: None,
            isotopes,
        }
    }

    fn sample_with_points(mp: Option<f64>, bp: Option<f64>) -> Element {
        let mut e = sample(55.845, vec![]);
        e.melting_point = mp;
        e.boiling_point = bp;
        e
    }
}
```

- [ ] **Step 5: Run tests to verify they fail**

Run: `cargo test -p pt-domain calc`
Expected: FAIL — functions not defined.

- [ ] **Step 6: Implement the calc functions**

Insert into `crates/pt-domain/src/calc.rs`, before the `#[cfg(test)]` module:

```rust
/// Abundance-weighted mean of the isotope relative masses, or `None` if there
/// are no isotopes or the total abundance is zero.
pub fn atomic_mass_from_isotopes(isotopes: &[Isotope]) -> Option<f64> {
    if isotopes.is_empty() {
        return None;
    }
    let total_abundance: f64 = isotopes.iter().map(|i| i.abundance).sum();
    if total_abundance == 0.0 {
        return None;
    }
    let weighted: f64 = isotopes.iter().map(|i| i.relative_mass * i.abundance).sum();
    Some(weighted / total_abundance)
}

/// Whether the isotope-derived mass matches the stored atomic mass within `tolerance`.
pub fn isotope_mass_matches(element: &Element, tolerance: f64) -> bool {
    match atomic_mass_from_isotopes(&element.isotopes) {
        Some(mass) => (mass - element.atomic_mass).abs() <= tolerance,
        None => false,
    }
}

/// The physical state at `temperature_k`, from the stored melting/boiling points.
/// Returns `None` when either point is unknown.
pub fn state_at(element: &Element, temperature_k: f64) -> Option<StateOfMatter> {
    let mp = element.melting_point?;
    let bp = element.boiling_point?;
    Some(if temperature_k < mp {
        StateOfMatter::Solid
    } else if temperature_k < bp {
        StateOfMatter::Liquid
    } else {
        StateOfMatter::Gas
    })
}
```

- [ ] **Step 7: Wire modules and re-exports**

Update `crates/pt-domain/src/lib.rs` to its final form:

```rust
//! Pure value types and stateless calculations for the periodic table.

pub mod calc;
pub mod classification;
pub mod config;
pub mod element;
pub mod error;

pub use calc::{atomic_mass_from_isotopes, isotope_mass_matches, state_at};
pub use classification::{
    block, category, group, oxidation_states, period, Block, Category, OxidationStates,
};
pub use config::{electron_configuration, ElectronConfiguration, Orbital, Subshell};
pub use element::{Element, Isotope, StateOfMatter};
pub use error::DomainError;
```

- [ ] **Step 8: Run the full domain test suite**

Run: `cargo test -p pt-domain`
Expected: PASS (all domain tests).

- [ ] **Step 9: Commit**

```bash
git add crates/pt-domain/src/
git commit -m "feat(domain): add oxidation states, isotope mass, and state-at-temperature"
```

---

## Task 7: pt-data — error type and YAML conversion

**Files:**
- Create: `crates/pt-data/src/error.rs`
- Create: `crates/pt-data/src/raw.rs`
- Modify: `crates/pt-data/src/lib.rs`

- [ ] **Step 1: Write the error type**

Create `crates/pt-data/src/error.rs`:

```rust
//! Data-loading errors.

/// Error returned while loading or validating element data.
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse {file}: {source}")]
    Parse {
        file: String,
        source: serde_yaml_ng::Error,
    },
    #[error("validation error in {file}: {message}")]
    Validation { file: String, message: String },
    #[error("duplicate atomic number {0}")]
    DuplicateAtomicNumber(u8),
    #[error("duplicate symbol {0}")]
    DuplicateSymbol(String),
    #[error("duplicate name {0}")]
    DuplicateName(String),
    #[error("no element files found in {0}")]
    EmptyDataDir(String),
}
```

- [ ] **Step 2: Write failing tests for the DTO conversion**

Create `crates/pt-data/src/raw.rs`:

```rust
//! Serde DTOs for YAML and conversion into domain types. Keeps serialization
//! concerns out of `pt-domain`.

use crate::error::DataError;
use pt_domain::{Element, Isotope, StateOfMatter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RawState {
    Solid,
    Liquid,
    Gas,
}

impl From<RawState> for StateOfMatter {
    fn from(s: RawState) -> Self {
        match s {
            RawState::Solid => StateOfMatter::Solid,
            RawState::Liquid => StateOfMatter::Liquid,
            RawState::Gas => StateOfMatter::Gas,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawIsotope {
    pub mass_number: u16,
    pub relative_mass: f64,
    pub abundance: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawElement {
    pub atomic_number: u8,
    pub name: String,
    pub symbol: String,
    pub atomic_mass: f64,
    pub mass_number: u16,
    #[serde(default)]
    pub melting_point: Option<f64>,
    #[serde(default)]
    pub boiling_point: Option<f64>,
    #[serde(default)]
    pub density: Option<f64>,
    #[serde(default)]
    pub electronegativity: Option<f64>,
    pub state: RawState,
    #[serde(default)]
    pub discovery_year: Option<i32>,
    #[serde(default)]
    pub discoverer: Option<String>,
    #[serde(default)]
    pub isotopes: Vec<RawIsotope>,
}

impl RawElement {
    /// Validates and converts into a domain `Element`. `file` is used for error context.
    pub(crate) fn into_element(self, file: &str) -> Result<Element, DataError> {
        if !(1..=118).contains(&self.atomic_number) {
            return Err(DataError::Validation {
                file: file.to_string(),
                message: format!("atomic_number {} out of range 1..=118", self.atomic_number),
            });
        }
        if !self.isotopes.is_empty() {
            let sum: f64 = self.isotopes.iter().map(|i| i.abundance).sum();
            if (sum - 1.0).abs() > 0.01 {
                return Err(DataError::Validation {
                    file: file.to_string(),
                    message: format!("isotope abundances sum to {sum:.4}, expected ~1.0"),
                });
            }
        }
        Ok(Element {
            atomic_number: self.atomic_number,
            name: self.name,
            symbol: self.symbol,
            atomic_mass: self.atomic_mass,
            mass_number: self.mass_number,
            melting_point: self.melting_point,
            boiling_point: self.boiling_point,
            density: self.density,
            electronegativity: self.electronegativity,
            state: self.state.into(),
            discovery_year: self.discovery_year,
            discoverer: self.discoverer,
            isotopes: self
                .isotopes
                .into_iter()
                .map(|i| Isotope {
                    mass_number: i.mass_number,
                    relative_mass: i.relative_mass,
                    abundance: i.abundance,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELIUM_YAML: &str = r#"
atomic_number: 2
name: Helium
symbol: He
atomic_mass: 4.0026
mass_number: 4
boiling_point: 4.222
state: gas
discovery_year: 1868
isotopes:
  - { mass_number: 3, relative_mass: 3.016029, abundance: 0.00000134 }
  - { mass_number: 4, relative_mass: 4.002603, abundance: 0.99999866 }
"#;

    #[test]
    fn parses_helium_with_optional_fields_absent() {
        let raw: RawElement = serde_yaml_ng::from_str(HELIUM_YAML).unwrap();
        let element = raw.into_element("helium.yaml").unwrap();
        assert_eq!(element.symbol, "He");
        assert_eq!(element.state, StateOfMatter::Gas);
        assert_eq!(element.melting_point, None);
        assert_eq!(element.electronegativity, None);
        assert_eq!(element.isotopes.len(), 2);
    }

    #[test]
    fn rejects_out_of_range_atomic_number() {
        let yaml = "atomic_number: 0\nname: X\nsymbol: X\natomic_mass: 1.0\nmass_number: 1\nstate: solid\n";
        let raw: RawElement = serde_yaml_ng::from_str(yaml).unwrap();
        let err = raw.into_element("x.yaml").unwrap_err();
        assert!(matches!(err, DataError::Validation { .. }));
    }

    #[test]
    fn rejects_bad_abundance_sum() {
        let yaml = "atomic_number: 1\nname: H\nsymbol: H\natomic_mass: 1.0\nmass_number: 1\nstate: gas\nisotopes:\n  - { mass_number: 1, relative_mass: 1.0, abundance: 0.5 }\n";
        let raw: RawElement = serde_yaml_ng::from_str(yaml).unwrap();
        let err = raw.into_element("h.yaml").unwrap_err();
        assert!(matches!(err, DataError::Validation { .. }));
    }
}
```

- [ ] **Step 3: Wire the modules**

Replace `crates/pt-data/src/lib.rs` with:

```rust
//! YAML loading and the in-memory element repository.

mod error;
mod raw;

pub use error::DataError;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pt-data raw`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pt-data/src/error.rs crates/pt-data/src/raw.rs crates/pt-data/src/lib.rs
git commit -m "feat(data): add DataError and validating YAML-to-domain conversion"
```

---

## Task 8: pt-data — parse a single element file

**Files:**
- Create: `crates/pt-data/src/parse.rs`
- Modify: `crates/pt-data/src/lib.rs`

- [ ] **Step 1: Write failing tests for parse_element_file**

Create `crates/pt-data/src/parse.rs`:

```rust
//! Reading and parsing a single element YAML file.

use crate::error::DataError;
use crate::raw::RawElement;
use pt_domain::Element;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn parses_a_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("lithium.yaml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "atomic_number: 3\nname: Lithium\nsymbol: Li\natomic_mass: 6.94\nmass_number: 7\nstate: solid\n"
        )
        .unwrap();

        let element = parse_element_file(&path).unwrap();
        assert_eq!(element.name, "Lithium");
        assert_eq!(element.atomic_number, 3);
    }

    #[test]
    fn missing_file_is_io_error() {
        let err = parse_element_file(Path::new("/no/such/file.yaml")).unwrap_err();
        assert!(matches!(err, DataError::Io { .. }));
    }

    #[test]
    fn malformed_yaml_is_parse_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.yaml");
        std::fs::write(&path, "this: : : not valid").unwrap();
        let err = parse_element_file(&path).unwrap_err();
        assert!(matches!(err, DataError::Parse { .. }));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pt-data parse`
Expected: FAIL — `parse_element_file` not defined.

- [ ] **Step 3: Implement parse_element_file**

Insert into `crates/pt-data/src/parse.rs`, before the `#[cfg(test)]` module:

```rust
/// Reads, parses, validates, and converts one element YAML file.
pub fn parse_element_file(path: &Path) -> Result<Element, DataError> {
    let file = path.display().to_string();
    let contents = std::fs::read_to_string(path).map_err(|e| DataError::Io {
        path: file.clone(),
        source: e,
    })?;
    let raw: RawElement = serde_yaml_ng::from_str(&contents).map_err(|e| DataError::Parse {
        file: file.clone(),
        source: e,
    })?;
    raw.into_element(&file)
}
```

- [ ] **Step 4: Wire the module**

Update `crates/pt-data/src/lib.rs` to:

```rust
//! YAML loading and the in-memory element repository.

mod error;
mod parse;
mod raw;

pub use error::DataError;
pub use parse::parse_element_file;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pt-data parse`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pt-data/src/parse.rs crates/pt-data/src/lib.rs
git commit -m "feat(data): parse a single element YAML file"
```

---

## Task 9: pt-data — element repository

**Files:**
- Create: `crates/pt-data/src/repository.rs`
- Modify: `crates/pt-data/src/lib.rs`

- [ ] **Step 1: Write failing tests for the repository**

Create `crates/pt-data/src/repository.rs`:

```rust
//! In-memory repository of elements, indexed for lookup. Loaded once.

use crate::error::DataError;
use crate::parse::parse_element_file;
use pt_domain::Element;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};

    fn write_element(dir: &TempDir, file: &str, body: &str) {
        let mut f = std::fs::File::create(dir.path().join(file)).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    }

    fn two_element_dir() -> TempDir {
        let dir = tempdir().unwrap();
        write_element(
            &dir,
            "hydrogen.yaml",
            "atomic_number: 1\nname: Hydrogen\nsymbol: H\natomic_mass: 1.008\nmass_number: 1\nstate: gas\n",
        );
        write_element(
            &dir,
            "iron.yaml",
            "atomic_number: 26\nname: Iron\nsymbol: Fe\natomic_mass: 55.845\nmass_number: 56\nstate: solid\n",
        );
        dir
    }

    #[test]
    fn loads_and_indexes() {
        let dir = two_element_dir();
        let repo = ElementRepository::load_from_dir(dir.path()).unwrap();
        assert_eq!(repo.len(), 2);
        assert_eq!(repo.get_by_atomic_number(26).unwrap().name, "Iron");
        assert_eq!(repo.get_by_symbol("fe").unwrap().name, "Iron"); // case-insensitive
        assert_eq!(repo.get_by_name("HYDROGEN").unwrap().symbol, "H");
        assert!(repo.get_by_atomic_number(99).is_none());
    }

    #[test]
    fn empty_dir_errors() {
        let dir = tempdir().unwrap();
        let err = ElementRepository::load_from_dir(dir.path()).unwrap_err();
        assert!(matches!(err, DataError::EmptyDataDir(_)));
    }

    #[test]
    fn duplicate_atomic_number_errors() {
        let dir = tempdir().unwrap();
        write_element(
            &dir,
            "a.yaml",
            "atomic_number: 1\nname: Hydrogen\nsymbol: H\natomic_mass: 1.0\nmass_number: 1\nstate: gas\n",
        );
        write_element(
            &dir,
            "b.yaml",
            "atomic_number: 1\nname: Protium\nsymbol: P\natomic_mass: 1.0\nmass_number: 1\nstate: gas\n",
        );
        let err = ElementRepository::load_from_dir(dir.path()).unwrap_err();
        assert!(matches!(err, DataError::DuplicateAtomicNumber(1)));
    }

    #[test]
    fn iter_is_sorted_by_atomic_number() {
        let dir = two_element_dir();
        let repo = ElementRepository::load_from_dir(dir.path()).unwrap();
        let numbers: Vec<u8> = repo.iter().map(|e| e.atomic_number).collect();
        assert_eq!(numbers, vec![1, 26]);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pt-data repository`
Expected: FAIL — `ElementRepository` not defined.

- [ ] **Step 3: Implement the repository and global helpers**

Insert into `crates/pt-data/src/repository.rs`, before the `#[cfg(test)]` module:

```rust
/// All loaded elements plus case-insensitive name/symbol indexes.
pub struct ElementRepository {
    elements: Vec<Element>,
    by_number: HashMap<u8, usize>,
    by_symbol: HashMap<String, usize>,
    by_name: HashMap<String, usize>,
}

impl ElementRepository {
    /// Loads every `*.yaml`/`*.yml` file in `dir`, validates, and indexes them.
    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, DataError> {
        let dir = dir.as_ref();
        let entries = std::fs::read_dir(dir).map_err(|e| DataError::Io {
            path: dir.display().to_string(),
            source: e,
        })?;

        let mut elements = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| DataError::Io {
                path: dir.display().to_string(),
                source: e,
            })?;
            let path = entry.path();
            let is_yaml = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext == "yaml" || ext == "yml")
                .unwrap_or(false);
            if !is_yaml {
                continue;
            }
            elements.push(parse_element_file(&path)?);
        }

        if elements.is_empty() {
            return Err(DataError::EmptyDataDir(dir.display().to_string()));
        }

        elements.sort_by_key(|e| e.atomic_number);

        let mut by_number = HashMap::new();
        let mut by_symbol = HashMap::new();
        let mut by_name = HashMap::new();
        for (idx, e) in elements.iter().enumerate() {
            if by_number.insert(e.atomic_number, idx).is_some() {
                return Err(DataError::DuplicateAtomicNumber(e.atomic_number));
            }
            if by_symbol.insert(e.symbol.to_lowercase(), idx).is_some() {
                return Err(DataError::DuplicateSymbol(e.symbol.clone()));
            }
            if by_name.insert(e.name.to_lowercase(), idx).is_some() {
                return Err(DataError::DuplicateName(e.name.clone()));
            }
        }

        Ok(Self { elements, by_number, by_symbol, by_name })
    }

    pub fn get_by_atomic_number(&self, z: u8) -> Option<&Element> {
        self.by_number.get(&z).map(|&i| &self.elements[i])
    }

    pub fn get_by_symbol(&self, symbol: &str) -> Option<&Element> {
        self.by_symbol.get(&symbol.to_lowercase()).map(|&i| &self.elements[i])
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Element> {
        self.by_name.get(&name.to_lowercase()).map(|&i| &self.elements[i])
    }

    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

static GLOBAL: OnceLock<ElementRepository> = OnceLock::new();

/// Loads the repository into a process-global slot the first time it is called.
/// Subsequent calls return the already-loaded repository and ignore `dir`.
pub fn init_global(dir: impl AsRef<Path>) -> Result<&'static ElementRepository, DataError> {
    if let Some(repo) = GLOBAL.get() {
        return Ok(repo);
    }
    let repo = ElementRepository::load_from_dir(dir)?;
    let _ = GLOBAL.set(repo);
    Ok(GLOBAL.get().expect("just initialized"))
}

/// Returns the global repository if `init_global` has been called.
pub fn global() -> Option<&'static ElementRepository> {
    GLOBAL.get()
}
```

- [ ] **Step 4: Wire the module**

Update `crates/pt-data/src/lib.rs` to:

```rust
//! YAML loading and the in-memory element repository.

mod error;
mod parse;
mod raw;
mod repository;

pub use error::DataError;
pub use parse::parse_element_file;
pub use repository::{global, init_global, ElementRepository};
```

- [ ] **Step 5: Run the full data test suite**

Run: `cargo test -p pt-data`
Expected: PASS (all data tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pt-data/src/repository.rs crates/pt-data/src/lib.rs
git commit -m "feat(data): add indexed ElementRepository with global helper"
```

---

## Task 10: Element data files

**Files:**
- Create: `data/elements/*.yaml` (14 fully-specified elements below, then the remainder)
- Create: `crates/pt-data/tests/dataset.rs` (completeness gate)

- [ ] **Step 1: Create the data directory and the test-referenced elements**

Create these 14 files verbatim (they are referenced by tests in this and later tasks). Temperatures in Kelvin, density in g/cm³, electronegativity on the Pauling scale.

`data/elements/hydrogen.yaml`:

```yaml
atomic_number: 1
name: Hydrogen
symbol: H
atomic_mass: 1.008
mass_number: 1
melting_point: 13.99
boiling_point: 20.271
density: 0.00008988
electronegativity: 2.20
state: gas
discovery_year: 1766
discoverer: Henry Cavendish
isotopes:
  - { mass_number: 1, relative_mass: 1.007825, abundance: 0.999885 }
  - { mass_number: 2, relative_mass: 2.014102, abundance: 0.000115 }
```

`data/elements/helium.yaml`:

```yaml
atomic_number: 2
name: Helium
symbol: He
atomic_mass: 4.002602
mass_number: 4
boiling_point: 4.222
density: 0.0001785
state: gas
discovery_year: 1868
discoverer: Pierre Janssen
isotopes:
  - { mass_number: 3, relative_mass: 3.016029, abundance: 0.00000134 }
  - { mass_number: 4, relative_mass: 4.002603, abundance: 0.99999866 }
```

`data/elements/lithium.yaml`:

```yaml
atomic_number: 3
name: Lithium
symbol: Li
atomic_mass: 6.94
mass_number: 7
melting_point: 453.65
boiling_point: 1603.0
density: 0.534
electronegativity: 0.98
state: solid
discovery_year: 1817
discoverer: Johan August Arfwedson
isotopes:
  - { mass_number: 6, relative_mass: 6.015123, abundance: 0.0759 }
  - { mass_number: 7, relative_mass: 7.016003, abundance: 0.9241 }
```

`data/elements/carbon.yaml`:

```yaml
atomic_number: 6
name: Carbon
symbol: C
atomic_mass: 12.011
mass_number: 12
melting_point: 3823.0
boiling_point: 4098.0
density: 2.267
electronegativity: 2.55
state: solid
isotopes:
  - { mass_number: 12, relative_mass: 12.0, abundance: 0.9893 }
  - { mass_number: 13, relative_mass: 13.003355, abundance: 0.0107 }
```

`data/elements/oxygen.yaml`:

```yaml
atomic_number: 8
name: Oxygen
symbol: O
atomic_mass: 15.999
mass_number: 16
melting_point: 54.36
boiling_point: 90.188
density: 0.001429
electronegativity: 3.44
state: gas
discovery_year: 1771
discoverer: Carl Wilhelm Scheele
isotopes:
  - { mass_number: 16, relative_mass: 15.994915, abundance: 0.99757 }
  - { mass_number: 17, relative_mass: 16.999132, abundance: 0.00038 }
  - { mass_number: 18, relative_mass: 17.99916, abundance: 0.00205 }
```

`data/elements/fluorine.yaml`:

```yaml
atomic_number: 9
name: Fluorine
symbol: F
atomic_mass: 18.998403
mass_number: 19
melting_point: 53.48
boiling_point: 85.03
density: 0.001696
electronegativity: 3.98
state: gas
discovery_year: 1886
discoverer: Henri Moissan
isotopes:
  - { mass_number: 19, relative_mass: 18.998403, abundance: 1.0 }
```

`data/elements/neon.yaml`:

```yaml
atomic_number: 10
name: Neon
symbol: Ne
atomic_mass: 20.1797
mass_number: 20
melting_point: 24.56
boiling_point: 27.07
density: 0.0009002
state: gas
discovery_year: 1898
discoverer: William Ramsay
isotopes:
  - { mass_number: 20, relative_mass: 19.99244, abundance: 0.9048 }
  - { mass_number: 21, relative_mass: 20.993847, abundance: 0.0027 }
  - { mass_number: 22, relative_mass: 21.991385, abundance: 0.0925 }
```

`data/elements/sodium.yaml`:

```yaml
atomic_number: 11
name: Sodium
symbol: Na
atomic_mass: 22.98977
mass_number: 23
melting_point: 370.94
boiling_point: 1156.0
density: 0.971
electronegativity: 0.93
state: solid
discovery_year: 1807
discoverer: Humphry Davy
isotopes:
  - { mass_number: 23, relative_mass: 22.989769, abundance: 1.0 }
```

`data/elements/magnesium.yaml`:

```yaml
atomic_number: 12
name: Magnesium
symbol: Mg
atomic_mass: 24.305
mass_number: 24
melting_point: 923.0
boiling_point: 1363.0
density: 1.738
electronegativity: 1.31
state: solid
discovery_year: 1755
discoverer: Joseph Black
isotopes:
  - { mass_number: 24, relative_mass: 23.985042, abundance: 0.7899 }
  - { mass_number: 25, relative_mass: 24.985837, abundance: 0.1000 }
  - { mass_number: 26, relative_mass: 25.982593, abundance: 0.1101 }
```

`data/elements/chlorine.yaml`:

```yaml
atomic_number: 17
name: Chlorine
symbol: Cl
atomic_mass: 35.45
mass_number: 35
melting_point: 171.6
boiling_point: 239.11
density: 0.003214
electronegativity: 3.16
state: gas
discovery_year: 1774
discoverer: Carl Wilhelm Scheele
isotopes:
  - { mass_number: 35, relative_mass: 34.968853, abundance: 0.7576 }
  - { mass_number: 37, relative_mass: 36.965903, abundance: 0.2424 }
```

`data/elements/iron.yaml`:

```yaml
atomic_number: 26
name: Iron
symbol: Fe
atomic_mass: 55.845
mass_number: 56
melting_point: 1811.0
boiling_point: 3134.0
density: 7.874
electronegativity: 1.83
state: solid
isotopes:
  - { mass_number: 54, relative_mass: 53.939609, abundance: 0.05845 }
  - { mass_number: 56, relative_mass: 55.934936, abundance: 0.91754 }
  - { mass_number: 57, relative_mass: 56.935393, abundance: 0.02119 }
  - { mass_number: 58, relative_mass: 57.933274, abundance: 0.00282 }
```

`data/elements/copper.yaml`:

```yaml
atomic_number: 29
name: Copper
symbol: Cu
atomic_mass: 63.546
mass_number: 63
melting_point: 1357.77
boiling_point: 2835.0
density: 8.96
electronegativity: 1.90
state: solid
isotopes:
  - { mass_number: 63, relative_mass: 62.929598, abundance: 0.6915 }
  - { mass_number: 65, relative_mass: 64.92779, abundance: 0.3085 }
```

`data/elements/neodymium.yaml`:

```yaml
atomic_number: 60
name: Neodymium
symbol: Nd
atomic_mass: 144.242
mass_number: 144
melting_point: 1297.0
boiling_point: 3347.0
density: 7.01
electronegativity: 1.14
state: solid
discovery_year: 1885
discoverer: Carl Auer von Welsbach
isotopes:
  - { mass_number: 142, relative_mass: 141.907729, abundance: 0.27152 }
  - { mass_number: 143, relative_mass: 142.90982, abundance: 0.12174 }
  - { mass_number: 144, relative_mass: 143.910093, abundance: 0.23798 }
  - { mass_number: 145, relative_mass: 144.912579, abundance: 0.08293 }
  - { mass_number: 146, relative_mass: 145.913123, abundance: 0.17189 }
  - { mass_number: 148, relative_mass: 147.916899, abundance: 0.05756 }
  - { mass_number: 150, relative_mass: 149.920902, abundance: 0.05638 }
```

`data/elements/uranium.yaml`:

```yaml
atomic_number: 92
name: Uranium
symbol: U
atomic_mass: 238.02891
mass_number: 238
melting_point: 1405.3
boiling_point: 4404.0
density: 19.1
electronegativity: 1.38
state: solid
discovery_year: 1789
discoverer: Martin Heinrich Klaproth
isotopes:
  - { mass_number: 234, relative_mass: 234.040952, abundance: 0.000054 }
  - { mass_number: 235, relative_mass: 235.043930, abundance: 0.007204 }
  - { mass_number: 238, relative_mass: 238.050788, abundance: 0.992742 }
```

- [ ] **Step 2: Verify the partial dataset loads**

Run:
```bash
cargo test -p pt-data
```
Expected: PASS (existing data tests still pass; they use temp dirs, not `data/`).

Then sanity-check the real directory loads by running the binary built later — for now, confirm the files parse with a quick throwaway check is unnecessary; the completeness test in Step 3 will cover it.

- [ ] **Step 3: Write the completeness gate test**

Create `crates/pt-data/tests/dataset.rs`:

```rust
//! Acceptance gate for the bundled dataset in `data/elements`.

use pt_data::ElementRepository;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/elements")
}

#[test]
fn dataset_loads_without_error() {
    // Must load and validate (uniqueness, ranges, abundance sums).
    ElementRepository::load_from_dir(data_dir()).expect("data/elements must load");
}

#[test]
#[ignore = "enable once all 118 elements are populated"]
fn dataset_covers_all_118_elements() {
    let repo = ElementRepository::load_from_dir(data_dir()).unwrap();
    assert_eq!(repo.len(), 118, "expected 118 element files");
    for z in 1..=118u8 {
        assert!(
            repo.get_by_atomic_number(z).is_some(),
            "missing element with atomic number {z}"
        );
    }
}
```

- [ ] **Step 4: Run the gate (partial dataset)**

Run: `cargo test -p pt-data --test dataset`
Expected: `dataset_loads_without_error` PASSES; `dataset_covers_all_118_elements` is reported as ignored.

- [ ] **Step 5: Populate the remaining elements**

Add one YAML file per remaining atomic number (4, 5, 7, 13–16, 18–25, 27, 28, 30–59, 61–91, 93–118) following the exact schema above, using standard reference values. Rules:
- Required keys: `atomic_number`, `name`, `symbol`, `atomic_mass`, `mass_number`, `state`.
- Omit `melting_point`/`boiling_point`/`density`/`electronegativity`/`discovery_year`/`discoverer` when no reliable value exists (synthetic superheavy / ancient elements). Omitting a key yields `null`/`None`.
- Include `isotopes` whose `abundance` values sum to ~1.0 (within 0.01) for elements with stable isotopes. For elements with no stable isotopes, either omit `isotopes` or include the most stable isotope with `abundance: 1.0`.
- For synthetic elements with no measured state, use the conventionally-predicted state (`solid` for most superheavy metals).

Remove the `#[ignore]` attribute from `dataset_covers_all_118_elements` once all files exist.

- [ ] **Step 6: Run the full completeness gate**

Run: `cargo test -p pt-data --test dataset`
Expected: both tests PASS (118 elements present and loading cleanly).

- [ ] **Step 7: Commit**

```bash
git add data/elements/ crates/pt-data/tests/dataset.rs
git commit -m "feat(data): add element dataset and completeness gate"
```

---

## Task 11: pt-services — error and element view

**Files:**
- Create: `crates/pt-services/src/error.rs`
- Create: `crates/pt-services/src/view.rs`
- Modify: `crates/pt-services/src/lib.rs`

- [ ] **Step 1: Write the service error**

Create `crates/pt-services/src/error.rs`:

```rust
//! Service-layer errors.

/// Error returned by the service API.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Data(#[from] pt_data::DataError),
}
```

- [ ] **Step 2: Write the element view (with a unit test)**

Create `crates/pt-services/src/view.rs`:

```rust
//! A borrowed view over a stored element that also exposes computed properties.

use pt_domain::{
    self as domain, Block, Category, ElectronConfiguration, Element, OxidationStates, StateOfMatter,
};

/// Combines an element's stored data with its computed properties.
pub struct ElementView<'a> {
    element: &'a Element,
}

impl<'a> ElementView<'a> {
    pub fn new(element: &'a Element) -> Self {
        Self { element }
    }

    /// The underlying stored element.
    pub fn element(&self) -> &Element {
        self.element
    }

    pub fn electron_configuration(&self) -> ElectronConfiguration {
        // Loaded elements always have a valid atomic number (validated by pt-data).
        domain::electron_configuration(self.element.atomic_number)
            .expect("element atomic number is valid by construction")
    }

    pub fn group(&self) -> u8 {
        domain::group(self.element.atomic_number).expect("valid atomic number")
    }

    pub fn period(&self) -> u8 {
        domain::period(self.element.atomic_number).expect("valid atomic number")
    }

    pub fn block(&self) -> Block {
        domain::block(self.element.atomic_number).expect("valid atomic number")
    }

    pub fn category(&self) -> Category {
        domain::category(self.element.atomic_number).expect("valid atomic number")
    }

    pub fn oxidation_states(&self) -> OxidationStates {
        domain::oxidation_states(self.element.atomic_number).expect("valid atomic number")
    }

    /// Atomic mass recomputed from the stored isotopes, if any.
    pub fn computed_atomic_mass(&self) -> Option<f64> {
        domain::atomic_mass_from_isotopes(&self.element.isotopes)
    }

    /// Physical state at the given temperature (K), if melting/boiling points are known.
    pub fn state_at(&self, temperature_k: f64) -> Option<StateOfMatter> {
        domain::state_at(self.element, temperature_k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_domain::Isotope;

    fn iron() -> Element {
        Element {
            atomic_number: 26,
            name: "Iron".into(),
            symbol: "Fe".into(),
            atomic_mass: 55.845,
            mass_number: 56,
            melting_point: Some(1811.0),
            boiling_point: Some(3134.0),
            density: Some(7.874),
            electronegativity: Some(1.83),
            state: StateOfMatter::Solid,
            discovery_year: None,
            discoverer: None,
            isotopes: vec![Isotope { mass_number: 56, relative_mass: 55.934936, abundance: 1.0 }],
        }
    }

    #[test]
    fn exposes_computed_properties() {
        let e = iron();
        let view = ElementView::new(&e);
        assert_eq!(view.group(), 8);
        assert_eq!(view.period(), 4);
        assert_eq!(view.block(), Block::D);
        assert_eq!(view.category(), Category::TransitionMetal);
        assert_eq!(view.state_at(300.0), Some(StateOfMatter::Solid));
        assert_eq!(view.electron_configuration().to_string(), "1s2 2s2 2p6 3s2 3p6 3d6 4s2");
    }
}
```

- [ ] **Step 3: Wire the modules**

Replace `crates/pt-services/src/lib.rs` with:

```rust
//! Query API exposing stored and computed element properties.

mod error;
mod view;

pub use error::ServiceError;
pub use view::ElementView;
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pt-services view`
Expected: PASS (1 test).

- [ ] **Step 5: Commit**

```bash
git add crates/pt-services/src/error.rs crates/pt-services/src/view.rs crates/pt-services/src/lib.rs
git commit -m "feat(services): add ServiceError and ElementView"
```

---

## Task 12: pt-services — PeriodicTable and integration tests

**Files:**
- Create: `crates/pt-services/src/table.rs`
- Modify: `crates/pt-services/src/lib.rs`
- Create: `crates/pt-services/tests/lookup.rs`

- [ ] **Step 1: Write the PeriodicTable**

Create `crates/pt-services/src/table.rs`:

```rust
//! The top-level query API.

use crate::error::ServiceError;
use crate::view::ElementView;
use pt_data::ElementRepository;
use std::path::Path;

/// Loaded periodic table providing lookups by several keys.
pub struct PeriodicTable {
    repo: ElementRepository,
}

impl PeriodicTable {
    /// Loads element data from a directory of YAML files (once).
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ServiceError> {
        Ok(Self {
            repo: ElementRepository::load_from_dir(dir)?,
        })
    }

    pub fn by_atomic_number(&self, z: u8) -> Option<ElementView<'_>> {
        self.repo.get_by_atomic_number(z).map(ElementView::new)
    }

    pub fn by_symbol(&self, symbol: &str) -> Option<ElementView<'_>> {
        self.repo.get_by_symbol(symbol).map(ElementView::new)
    }

    pub fn by_name(&self, name: &str) -> Option<ElementView<'_>> {
        self.repo.get_by_name(name).map(ElementView::new)
    }

    /// Nearest element whose stored atomic mass is within `tolerance` of `mass`.
    pub fn by_atomic_mass(&self, mass: f64, tolerance: f64) -> Option<ElementView<'_>> {
        self.repo
            .iter()
            .filter(|e| (e.atomic_mass - mass).abs() <= tolerance)
            .min_by(|a, b| {
                (a.atomic_mass - mass)
                    .abs()
                    .total_cmp(&(b.atomic_mass - mass).abs())
            })
            .map(ElementView::new)
    }

    pub fn all(&self) -> impl Iterator<Item = ElementView<'_>> {
        self.repo.iter().map(ElementView::new)
    }
}
```

- [ ] **Step 2: Wire the module**

Replace `crates/pt-services/src/lib.rs` with:

```rust
//! Query API exposing stored and computed element properties.

mod error;
mod table;
mod view;

pub use error::ServiceError;
pub use table::PeriodicTable;
pub use view::ElementView;
```

- [ ] **Step 3: Write integration tests against the real dataset**

Create `crates/pt-services/tests/lookup.rs`:

```rust
//! Integration tests for the service API, against the bundled dataset.

use pt_services::PeriodicTable;

fn table() -> PeriodicTable {
    let dir = format!("{}/../../data/elements", env!("CARGO_MANIFEST_DIR"));
    PeriodicTable::load(dir).expect("dataset must load")
}

#[test]
fn lookup_by_atomic_number() {
    let t = table();
    let fe = t.by_atomic_number(26).expect("iron present");
    assert_eq!(fe.element().name, "Iron");
    assert_eq!(fe.group(), 8);
    assert_eq!(fe.period(), 4);
}

#[test]
fn lookup_by_symbol_is_case_insensitive() {
    let t = table();
    assert_eq!(t.by_symbol("fe").unwrap().element().name, "Iron");
    assert_eq!(t.by_symbol("FE").unwrap().element().name, "Iron");
}

#[test]
fn lookup_by_name_is_case_insensitive() {
    let t = table();
    assert_eq!(t.by_name("HYDROGEN").unwrap().element().symbol, "H");
}

#[test]
fn lookup_by_mass_within_tolerance() {
    let t = table();
    let v = t.by_atomic_mass(55.8, 0.5).expect("iron within tolerance");
    assert_eq!(v.element().symbol, "Fe");
}

#[test]
fn lookup_misses_return_none() {
    let t = table();
    assert!(t.by_atomic_number(150).is_none());
    assert!(t.by_symbol("Zz").is_none());
    assert!(t.by_atomic_mass(999.0, 0.1).is_none());
}

#[test]
fn computed_atomic_mass_matches_stored_for_chlorine() {
    let t = table();
    let cl = t.by_symbol("Cl").unwrap();
    let computed = cl.computed_atomic_mass().unwrap();
    assert!((computed - cl.element().atomic_mass).abs() < 0.05, "got {computed}");
}
```

- [ ] **Step 4: Run the service tests**

Run: `cargo test -p pt-services`
Expected: PASS (unit test + all integration tests). These rely on the elements created in Task 10 Step 1.

- [ ] **Step 5: Commit**

```bash
git add crates/pt-services/src/table.rs crates/pt-services/src/lib.rs crates/pt-services/tests/lookup.rs
git commit -m "feat(services): add PeriodicTable lookups with integration tests"
```

---

## Task 13: pt-cli — command-line interface

**Files:**
- Create: `crates/pt-cli/src/output.rs`
- Modify: `crates/pt-cli/src/main.rs`
- Create: `crates/pt-cli/tests/cli.rs`

- [ ] **Step 1: Write the serializable output DTO and text printer**

Create `crates/pt-cli/src/output.rs`:

```rust
//! Output representation for the CLI. Holds a flat, serializable snapshot of an
//! element's stored and computed properties (keeps serde out of the domain crate).

use pt_services::ElementView;
use serde::Serialize;

#[derive(Serialize)]
pub struct ElementOutput {
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
    pub electron_configuration: String,
    pub group: u8,
    pub period: u8,
    pub block: String,
    pub category: String,
    pub oxidation_states: Vec<i8>,
    pub computed_atomic_mass: Option<f64>,
}

impl ElementOutput {
    pub fn from_view(view: &ElementView) -> Self {
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
            state: format!("{:?}", e.state).to_lowercase(),
            discovery_year: e.discovery_year,
            discoverer: e.discoverer.clone(),
            electron_configuration: view.electron_configuration().to_string(),
            group: view.group(),
            period: view.period(),
            block: format!("{:?}", view.block()).to_lowercase(),
            category: format!("{:?}", view.category()),
            oxidation_states: view.oxidation_states().0,
            computed_atomic_mass: view.computed_atomic_mass(),
        }
    }
}

fn fmt_opt(value: Option<f64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "—".to_string())
}

pub fn print_text(o: &ElementOutput) {
    println!("{} ({}) — atomic number {}", o.name, o.symbol, o.atomic_number);
    println!("  atomic mass:       {}", o.atomic_mass);
    println!("  mass number:       {}", o.mass_number);
    println!("  electron config:   {}", o.electron_configuration);
    println!("  group / period:    {} / {}", o.group, o.period);
    println!("  block:             {}", o.block);
    println!("  category:          {}", o.category);
    println!("  state (STP):       {}", o.state);
    println!("  melting point (K): {}", fmt_opt(o.melting_point));
    println!("  boiling point (K): {}", fmt_opt(o.boiling_point));
    println!("  density (g/cm³):   {}", fmt_opt(o.density));
    println!("  electronegativity: {}", fmt_opt(o.electronegativity));
    println!(
        "  oxidation states:  {}",
        o.oxidation_states
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let discoverer = o.discoverer.clone().unwrap_or_else(|| "—".to_string());
    let year = o.discovery_year.map(|y| y.to_string()).unwrap_or_else(|| "—".to_string());
    println!("  discovered:        {discoverer} ({year})");
}
```

- [ ] **Step 2: Write the CLI entry point**

Replace `crates/pt-cli/src/main.rs` with:

```rust
//! Command-line interface for querying the periodic table.

mod output;

use clap::{Args, Parser, Subcommand, ValueEnum};
use output::{print_text, ElementOutput};
use pt_services::{ElementView, PeriodicTable};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "pt", about = "Query the periodic table")]
struct Cli {
    /// Directory containing element YAML files.
    #[arg(long, default_value = "./data/elements", global = true)]
    data_dir: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text, global = true)]
    format: Format,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Look up a single element.
    Get(GetArgs),
    /// List all elements.
    List,
}

#[derive(Args)]
struct GetArgs {
    #[arg(long)]
    number: Option<u8>,
    #[arg(long)]
    symbol: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    mass: Option<f64>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
    Yaml,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let table = PeriodicTable::load(&cli.data_dir)?;
    match cli.command {
        Command::Get(args) => match lookup(&table, &args) {
            Some(view) => {
                emit(&ElementOutput::from_view(&view), cli.format)?;
                Ok(ExitCode::SUCCESS)
            }
            None => {
                eprintln!("error: no matching element found");
                Ok(ExitCode::FAILURE)
            }
        },
        Command::List => {
            let mut outputs: Vec<ElementOutput> =
                table.all().map(|v| ElementOutput::from_view(&v)).collect();
            outputs.sort_by_key(|o| o.atomic_number);
            match cli.format {
                Format::Text => {
                    for o in &outputs {
                        println!("{:>3}  {:<3} {}", o.atomic_number, o.symbol, o.name);
                    }
                }
                Format::Json => println!("{}", serde_json::to_string_pretty(&outputs)?),
                Format::Yaml => print!("{}", serde_yaml_ng::to_string(&outputs)?),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn lookup<'a>(table: &'a PeriodicTable, args: &GetArgs) -> Option<ElementView<'a>> {
    if let Some(z) = args.number {
        return table.by_atomic_number(z);
    }
    if let Some(symbol) = &args.symbol {
        return table.by_symbol(symbol);
    }
    if let Some(name) = &args.name {
        return table.by_name(name);
    }
    if let Some(mass) = args.mass {
        return table.by_atomic_mass(mass, 0.5);
    }
    None
}

fn emit(o: &ElementOutput, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        Format::Text => print_text(o),
        Format::Json => println!("{}", serde_json::to_string_pretty(o)?),
        Format::Yaml => print!("{}", serde_yaml_ng::to_string(o)?),
    }
    Ok(())
}
```

- [ ] **Step 3: Write integration tests**

Create `crates/pt-cli/tests/cli.rs`:

```rust
//! CLI integration tests.

use assert_cmd::Command;
use predicates::str::contains;

fn data_dir() -> String {
    format!("{}/../../data/elements", env!("CARGO_MANIFEST_DIR"))
}

fn pt() -> Command {
    let mut cmd = Command::cargo_bin("pt").unwrap();
    cmd.args(["--data-dir", &data_dir()]);
    cmd
}

#[test]
fn get_by_symbol_prints_name_and_config() {
    pt().args(["get", "--symbol", "Fe"])
        .assert()
        .success()
        .stdout(contains("Iron"))
        .stdout(contains("3d6 4s2"));
}

#[test]
fn get_by_number_json_format() {
    pt().args(["--format", "json", "get", "--number", "1"])
        .assert()
        .success()
        .stdout(contains("\"symbol\": \"H\""))
        .stdout(contains("\"group\": 1"));
}

#[test]
fn get_by_mass_finds_nearest() {
    pt().args(["get", "--mass", "55.8"])
        .assert()
        .success()
        .stdout(contains("Iron"));
}

#[test]
fn unknown_element_exits_nonzero() {
    pt().args(["get", "--symbol", "Zz"])
        .assert()
        .failure()
        .stderr(contains("no matching element"));
}

#[test]
fn list_prints_multiple_elements() {
    pt().arg("list")
        .assert()
        .success()
        .stdout(contains("Hydrogen"))
        .stdout(contains("Iron"));
}
```

- [ ] **Step 4: Run the CLI tests**

Run: `cargo test -p pt-cli`
Expected: PASS (all CLI integration tests). `assert_cmd` builds the `pt` binary automatically.

- [ ] **Step 5: Smoke-test the binary manually**

Run: `cargo run -p pt-cli -- get --symbol Fe`
Expected: prints the Iron summary including electron configuration `... 3d6 4s2`, group 8, period 4.

- [ ] **Step 6: Commit**

```bash
git add crates/pt-cli/src/output.rs crates/pt-cli/src/main.rs crates/pt-cli/tests/cli.rs
git commit -m "feat(cli): add get/list commands with text/json/yaml output"
```

---

## Task 14: Final verification

**Files:** none (verification only)

- [ ] **Step 1: Run the entire workspace test suite**

Run: `cargo test --workspace`
Expected: all unit and integration tests PASS. (If the full 118-element dataset is not yet populated, the `dataset_covers_all_118_elements` test will be `ignored` until Task 10 Step 5 is complete and the `#[ignore]` is removed.)

- [ ] **Step 2: Check formatting and lints**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets`
Expected: no formatting diffs; clippy reports no errors (warnings acceptable but prefer to address them).

- [ ] **Step 3: Confirm docs build**

Run: `cargo doc --workspace --no-deps`
Expected: documentation builds without errors.

- [ ] **Step 4: Commit any formatting fixes**

```bash
git add -A
git commit -m "chore: apply rustfmt and clippy fixes" || echo "nothing to commit"
```

---

## Self-Review Notes

- **Spec coverage:** workspace + 3 layers + CLI (Tasks 1, 11–13); YAML data model & units (Tasks 7, 10); load-once repository with indexes & validation (Task 9); computed properties — electron config/anomalies (Task 3), block/period/group (Task 4), category (Task 5), oxidation states + isotope mass + state-at (Task 6); service lookups by number/name/symbol/mass (Task 12); CLI with `--data-dir`/`--format` (Task 13); per-layer error types (Tasks 2, 7, 11); unit + integration tests throughout; 118-element dataset with completeness gate (Task 10).
- **Heuristics:** `category` and `oxidation_states` are explicitly best-effort per the spec; tests cover only unambiguous cases.
- **Known simplifications encoded deliberately:** `period` and `block` use the naive Aufbau fill (so anomaly electron-shuffling like Pd's dropped 5s does not change the row/block); f-block elements are assigned group 3 by convention; Lr is intentionally excluded from the anomaly table so its group derives correctly.
