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
