use pt_services::ElementView;
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmIsotope {
    pub mass_number: u16,
    pub relative_mass: f64,
    pub abundance: f64,
}

/// Three-way element classification exposed to TypeScript as a union type.
/// Never constructed in Rust; exists only for tsify to emit the TypeScript union in pt_wasm.d.ts.
#[allow(dead_code)]
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub enum WasmElementClass {
    Metal,
    NonMetal,
    Metalloid,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmElement {
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
    pub isotopes: Vec<WasmIsotope>,
    pub electron_configuration: String,
    pub group: u8,
    pub period: u8,
    pub block: String,
    pub category: String,
    pub oxidation_states: Vec<i8>,
    pub computed_atomic_mass: Option<f64>,
    pub class: String,
}

impl From<ElementView<'_>> for WasmElement {
    fn from(view: ElementView<'_>) -> Self {
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
            state: format!("{:?}", e.state),
            discovery_year: e.discovery_year,
            discoverer: e.discoverer.clone(),
            isotopes: e
                .isotopes
                .iter()
                .map(|i| WasmIsotope {
                    mass_number: i.mass_number,
                    relative_mass: i.relative_mass,
                    abundance: i.abundance,
                })
                .collect(),
            electron_configuration: view.electron_configuration().to_string(),
            group: view.group(),
            period: view.period(),
            block: format!("{:?}", view.block()),
            category: format!("{:?}", view.category()),
            oxidation_states: view.oxidation_states().0,
            computed_atomic_mass: view.computed_atomic_mass(),
            class: format!("{:?}", view.element_class()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pt_domain::{Element, Isotope, StateOfMatter};

    fn make_iron() -> Element {
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
    fn converts_stored_fields() {
        let iron = make_iron();
        let view = ElementView::new(&iron);
        let w = WasmElement::from(view);
        assert_eq!(w.atomic_number, 26);
        assert_eq!(w.symbol, "Fe");
        assert_eq!(w.name, "Iron");
        assert_eq!(w.state, "Solid");
        assert_eq!(w.isotopes.len(), 1);
        assert_eq!(w.isotopes[0].mass_number, 56);
    }

    #[test]
    fn converts_computed_fields() {
        let iron = make_iron();
        let view = ElementView::new(&iron);
        let w = WasmElement::from(view);
        assert_eq!(w.group, 8);
        assert_eq!(w.period, 4);
        assert_eq!(w.block, "D");
        assert_eq!(w.category, "TransitionMetal");
        assert_eq!(w.oxidation_states, vec![2i8, 3i8]);
        assert!(w.electron_configuration.contains("3d6"));
        assert_eq!(w.class, "Metal");
    }

    #[test]
    fn optional_fields_are_none_when_absent() {
        let iron = make_iron();
        let view = ElementView::new(&iron);
        let w = WasmElement::from(view);
        assert_eq!(w.discovery_year, None);
        assert_eq!(w.discoverer, None);
    }
}
