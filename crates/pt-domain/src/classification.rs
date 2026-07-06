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
    let g = group(z)?;
    if z == 1 {
        return Ok(NonMetal);
    }
    if g == 17 || g == 18 {
        return Ok(NonMetal);
    }
    if (57..=71).contains(&z) || (89..=103).contains(&z) {
        return Ok(Metal);
    }
    if CLASS_METALLOIDS.contains(&z) {
        return Ok(Metalloid);
    }
    let p = period(z)?;
    if p == 2 && (14..=16).contains(&g) {
        return Ok(NonMetal);
    }
    if p == 3 && (15..=16).contains(&g) {
        return Ok(NonMetal);
    }
    if p == 4 && g == 16 {
        return Ok(NonMetal);
    }
    Ok(Metal)
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

/// Common oxidation states (heuristic, group-based — not authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxidationStates(pub Vec<i8>);

/// Best-effort common oxidation states derived from the group.
pub fn oxidation_states(z: u8) -> Result<OxidationStates, DomainError> {
    validate_z(z)?;
    let states = match group(z)? {
        1 => vec![1],
        2 => vec![2],
        3..=12 => vec![2, 3],
        13 => vec![3],
        14 => vec![-4, 4],
        15 => vec![-3, 3, 5],
        16 => vec![-2],
        17 => vec![-1],
        // group 18 (noble gases); the catch-all is unreachable for valid groups.
        _ => vec![0],
    };
    Ok(OxidationStates(states))
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
    fn lanthanide_actinide_boundary() {
        // La (57): block from naive fill is f; period 6; group 3 by convention.
        assert_eq!(block(57).unwrap(), Block::F);
        assert_eq!(period(57).unwrap(), 6);
        assert_eq!(group(57).unwrap(), 3);
        assert_eq!(category(57).unwrap(), Category::Lanthanide);
        // Lr (103): excluded from the anomaly table, so naive fill ends at 6d -> d-block,
        // period 7, group 3 (6d1 + 7s2), categorized as an actinide.
        assert_eq!(block(103).unwrap(), Block::D);
        assert_eq!(period(103).unwrap(), 7);
        assert_eq!(group(103).unwrap(), 3);
        assert_eq!(category(103).unwrap(), Category::Actinide);
    }

    #[test]
    fn invalid_z_errors() {
        assert_eq!(block(0), Err(DomainError::InvalidAtomicNumber(0)));
        assert_eq!(period(200), Err(DomainError::InvalidAtomicNumber(200)));
        assert_eq!(group(0), Err(DomainError::InvalidAtomicNumber(0)));
    }

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

    #[test]
    fn oxidation_states_main_group() {
        assert_eq!(oxidation_states(11).unwrap().0, vec![1]); // Na
        assert_eq!(oxidation_states(12).unwrap().0, vec![2]); // Mg
        assert_eq!(oxidation_states(8).unwrap().0, vec![-2]); // O
        assert_eq!(oxidation_states(9).unwrap().0, vec![-1]); // F
        assert_eq!(oxidation_states(10).unwrap().0, vec![0]); // Ne (group 18, catch-all)
        assert_eq!(oxidation_states(5).unwrap().0, vec![3]); // B
        assert_eq!(oxidation_states(6).unwrap().0, vec![-4, 4]); // C (group 14)
        assert_eq!(oxidation_states(7).unwrap().0, vec![-3, 3, 5]); // N (group 15)
        assert_eq!(oxidation_states(26).unwrap().0, vec![2, 3]); // Fe (transition)
        assert!(oxidation_states(0).is_err());
    }

    #[test]
    fn element_class_hydrogen_exception() {
        assert_eq!(element_class(1).unwrap(), ElementClass::NonMetal);
    }

    #[test]
    fn element_class_halogens_and_noble_gases() {
        assert_eq!(element_class(17).unwrap(), ElementClass::NonMetal); // Cl
        assert_eq!(element_class(35).unwrap(), ElementClass::NonMetal); // Br
        assert_eq!(element_class(2).unwrap(), ElementClass::NonMetal); // He
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
        assert_eq!(element_class(6).unwrap(), ElementClass::NonMetal); // C  (period 2, group 14)
        assert_eq!(element_class(7).unwrap(), ElementClass::NonMetal); // N  (period 2, group 15)
        assert_eq!(element_class(8).unwrap(), ElementClass::NonMetal); // O  (period 2, group 16)
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
}
