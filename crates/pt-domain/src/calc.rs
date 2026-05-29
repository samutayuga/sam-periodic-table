//! Calculations over stored element data.

use crate::element::{Element, Isotope, StateOfMatter};

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
