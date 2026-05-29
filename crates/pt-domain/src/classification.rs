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
