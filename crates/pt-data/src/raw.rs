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
