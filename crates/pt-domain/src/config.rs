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
        orbitals.push(Orbital {
            n,
            subshell,
            electrons: electrons as u8,
        });
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
            if let Some(orbital) = orbitals
                .iter_mut()
                .find(|o| o.n == n && o.subshell == subshell)
            {
                orbital.electrons = electrons;
            } else {
                orbitals.push(Orbital {
                    n,
                    subshell,
                    electrons,
                });
            }
        }
        orbitals.retain(|o| o.electrons > 0);
    }
    Ok(ElectronConfiguration { orbitals })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_atomic_number_errors() {
        assert_eq!(
            electron_configuration(0),
            Err(DomainError::InvalidAtomicNumber(0))
        );
        assert_eq!(
            electron_configuration(119),
            Err(DomainError::InvalidAtomicNumber(119))
        );
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
        assert_eq!(
            electron_configuration(10).unwrap().to_string(),
            "1s2 2s2 2p6"
        );
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
