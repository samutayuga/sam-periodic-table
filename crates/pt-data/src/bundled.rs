use crate::{DataError, ElementRepository};

pub struct StaticIsotope {
    pub mass_number: u16,
    pub relative_mass: f64,
    pub abundance: f64,
}

pub struct StaticElement {
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
    pub isotopes: &'static [StaticIsotope],
}

include!(concat!(env!("OUT_DIR"), "/generated_elements.rs"));

pub fn load_bundled() -> Result<ElementRepository, DataError> {
    ElementRepository::load_from_static(ELEMENTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_data_loads_all_elements() {
        let repo = load_bundled().unwrap();
        assert_eq!(repo.len(), 118);
    }

    #[test]
    fn bundled_hydrogen_is_first() {
        let repo = load_bundled().unwrap();
        let h = repo.get_by_atomic_number(1).unwrap();
        assert_eq!(h.symbol, "H");
        assert_eq!(h.name, "Hydrogen");
    }

    #[test]
    fn bundled_iron_lookup_by_symbol() {
        let repo = load_bundled().unwrap();
        let fe = repo.get_by_symbol("Fe").unwrap();
        assert_eq!(fe.atomic_number, 26);
    }
}
