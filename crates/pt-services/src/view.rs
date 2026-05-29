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
            isotopes: vec![Isotope {
                mass_number: 56,
                relative_mass: 55.934936,
                abundance: 1.0,
            }],
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
        assert_eq!(
            view.electron_configuration().to_string(),
            "1s2 2s2 2p6 3s2 3p6 3d6 4s2"
        );
    }
}
