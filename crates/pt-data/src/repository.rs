//! In-memory repository of elements, indexed for lookup. Loaded once.

use crate::error::DataError;
use crate::parse::parse_element_file;
use pt_domain::Element;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

/// All loaded elements plus case-insensitive name/symbol indexes.
#[derive(Debug)]
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
        let to_io_err = |e: std::io::Error| DataError::Io {
            path: dir.display().to_string(),
            source: e,
        };
        let entries = std::fs::read_dir(dir).map_err(&to_io_err)?;

        let mut elements = Vec::new();
        for entry in entries {
            let entry = entry.map_err(&to_io_err)?;
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

        Self::build_index(elements)
    }

    #[cfg(feature = "bundled")]
    pub fn load_from_static(raw: &[crate::bundled::StaticElement]) -> Result<Self, DataError> {
        use pt_domain::{Element, Isotope, StateOfMatter};

        let elements: Result<Vec<Element>, DataError> = raw.iter().map(|r| {
            if !(1..=118).contains(&r.atomic_number) {
                return Err(DataError::Validation {
                    file: r.name.to_string(),
                    message: format!("atomic_number {} out of range 1..=118", r.atomic_number),
                });
            }
            if !r.isotopes.is_empty() {
                let sum: f64 = r.isotopes.iter().map(|i| i.abundance).sum();
                if (sum - 1.0).abs() > 0.01 {
                    return Err(DataError::Validation {
                        file: r.name.to_string(),
                        message: format!("isotope abundances sum to {sum:.4}, expected ~1.0"),
                    });
                }
            }
            let state = match r.state {
                "solid" => StateOfMatter::Solid,
                "liquid" => StateOfMatter::Liquid,
                "gas" => StateOfMatter::Gas,
                s => return Err(DataError::Validation {
                    file: r.name.to_string(),
                    message: format!("unknown state: {s}"),
                }),
            };
            Ok(Element {
                atomic_number: r.atomic_number,
                name: r.name.to_string(),
                symbol: r.symbol.to_string(),
                atomic_mass: r.atomic_mass,
                mass_number: r.mass_number,
                melting_point: r.melting_point,
                boiling_point: r.boiling_point,
                density: r.density,
                electronegativity: r.electronegativity,
                state,
                discovery_year: r.discovery_year,
                discoverer: r.discoverer.map(|s| s.to_string()),
                isotopes: r.isotopes.iter().map(|i| Isotope {
                    mass_number: i.mass_number,
                    relative_mass: i.relative_mass,
                    abundance: i.abundance,
                }).collect(),
            })
        }).collect();

        Self::build_index(elements?)
    }

    fn build_index(mut elements: Vec<Element>) -> Result<Self, DataError> {
        if elements.is_empty() {
            return Err(DataError::EmptyDataDir("(bundled)".to_string()));
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
        self.by_symbol
            .get(&symbol.to_lowercase())
            .map(|&i| &self.elements[i])
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Element> {
        self.by_name
            .get(&name.to_lowercase())
            .map(|&i| &self.elements[i])
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

    #[test]
    fn nonexistent_dir_is_io_error() {
        let err = ElementRepository::load_from_dir("/no/such/directory/here").unwrap_err();
        assert!(matches!(err, DataError::Io { .. }));
    }

    #[test]
    fn skips_non_yaml_files() {
        let dir = tempdir().unwrap();
        write_element(
            &dir,
            "hydrogen.yaml",
            "atomic_number: 1\nname: Hydrogen\nsymbol: H\natomic_mass: 1.008\nmass_number: 1\nstate: gas\n",
        );
        write_element(&dir, "notes.txt", "this file should be ignored\n");
        let repo = ElementRepository::load_from_dir(dir.path()).unwrap();
        assert_eq!(repo.len(), 1);
    }

    #[test]
    fn duplicate_symbol_errors() {
        let dir = tempdir().unwrap();
        write_element(
            &dir,
            "a.yaml",
            "atomic_number: 1\nname: Aaa\nsymbol: X\natomic_mass: 1.0\nmass_number: 1\nstate: gas\n",
        );
        write_element(
            &dir,
            "b.yaml",
            "atomic_number: 2\nname: Bbb\nsymbol: X\natomic_mass: 2.0\nmass_number: 2\nstate: gas\n",
        );
        let err = ElementRepository::load_from_dir(dir.path()).unwrap_err();
        assert!(matches!(err, DataError::DuplicateSymbol(_)));
    }

    #[test]
    fn duplicate_name_errors() {
        let dir = tempdir().unwrap();
        write_element(
            &dir,
            "a.yaml",
            "atomic_number: 1\nname: Same\nsymbol: Aa\natomic_mass: 1.0\nmass_number: 1\nstate: gas\n",
        );
        write_element(
            &dir,
            "b.yaml",
            "atomic_number: 2\nname: Same\nsymbol: Bb\natomic_mass: 2.0\nmass_number: 2\nstate: gas\n",
        );
        let err = ElementRepository::load_from_dir(dir.path()).unwrap_err();
        assert!(matches!(err, DataError::DuplicateName(_)));
    }

    #[cfg(feature = "bundled")]
    #[test]
    fn load_from_static_indexes_correctly() {
        use crate::bundled::{StaticElement, StaticIsotope};

        let elements = [
            StaticElement {
                atomic_number: 1,
                name: "Hydrogen",
                symbol: "H",
                atomic_mass: 1.008,
                mass_number: 1,
                melting_point: Some(13.99),
                boiling_point: Some(20.271),
                density: Some(0.00008988),
                electronegativity: Some(2.20),
                state: "gas",
                discovery_year: Some(1766),
                discoverer: Some("Henry Cavendish"),
                isotopes: &[
                    StaticIsotope { mass_number: 1, relative_mass: 1.007825, abundance: 0.999885 },
                    StaticIsotope { mass_number: 2, relative_mass: 2.014102, abundance: 0.000115 },
                ],
            },
            StaticElement {
                atomic_number: 26,
                name: "Iron",
                symbol: "Fe",
                atomic_mass: 55.845,
                mass_number: 56,
                melting_point: Some(1811.0),
                boiling_point: Some(3134.0),
                density: Some(7.874),
                electronegativity: Some(1.83),
                state: "solid",
                discovery_year: None,
                discoverer: None,
                isotopes: &[
                    StaticIsotope { mass_number: 56, relative_mass: 55.934936, abundance: 1.0 },
                ],
            },
        ];

        let repo = ElementRepository::load_from_static(&elements).unwrap();
        assert_eq!(repo.len(), 2);
        assert_eq!(repo.get_by_atomic_number(26).unwrap().name, "Iron");
        assert_eq!(repo.get_by_symbol("fe").unwrap().name, "Iron");
        assert_eq!(repo.get_by_name("HYDROGEN").unwrap().symbol, "H");
    }

    #[cfg(feature = "bundled")]
    #[test]
    fn load_from_static_rejects_out_of_range_atomic_number() {
        use crate::bundled::StaticElement;
        let elements = [StaticElement {
            atomic_number: 0,
            name: "Bad",
            symbol: "Bd",
            atomic_mass: 1.0,
            mass_number: 1,
            melting_point: None,
            boiling_point: None,
            density: None,
            electronegativity: None,
            state: "solid",
            discovery_year: None,
            discoverer: None,
            isotopes: &[],
        }];
        let err = ElementRepository::load_from_static(&elements).unwrap_err();
        assert!(matches!(err, DataError::Validation { .. }));
    }

    #[cfg(feature = "bundled")]
    #[test]
    fn load_from_static_rejects_unknown_state() {
        use crate::bundled::StaticElement;
        let elements = [StaticElement {
            atomic_number: 1,
            name: "Hydrogen",
            symbol: "H",
            atomic_mass: 1.008,
            mass_number: 1,
            melting_point: None,
            boiling_point: None,
            density: None,
            electronegativity: None,
            state: "plasma",
            discovery_year: None,
            discoverer: None,
            isotopes: &[],
        }];
        let err = ElementRepository::load_from_static(&elements).unwrap_err();
        assert!(matches!(err, DataError::Validation { .. }));
    }

    #[cfg(feature = "bundled")]
    #[test]
    fn load_from_static_rejects_bad_abundance_sum() {
        use crate::bundled::{StaticElement, StaticIsotope};
        let elements = [StaticElement {
            atomic_number: 1,
            name: "Hydrogen",
            symbol: "H",
            atomic_mass: 1.008,
            mass_number: 1,
            melting_point: None,
            boiling_point: None,
            density: None,
            electronegativity: None,
            state: "gas",
            discovery_year: None,
            discoverer: None,
            isotopes: &[
                StaticIsotope { mass_number: 1, relative_mass: 1.007825, abundance: 0.5 },
            ],
        }];
        let err = ElementRepository::load_from_static(&elements).unwrap_err();
        assert!(matches!(err, DataError::Validation { .. }));
    }

    #[test]
    fn malformed_file_aborts_load() {
        let dir = tempdir().unwrap();
        write_element(&dir, "broken.yaml", "this: : : not valid yaml");
        let err = ElementRepository::load_from_dir(dir.path()).unwrap_err();
        assert!(matches!(err, DataError::Parse { .. }));
    }

    #[test]
    fn is_empty_is_false_for_loaded_repo() {
        let dir = two_element_dir();
        let repo = ElementRepository::load_from_dir(dir.path()).unwrap();
        assert!(!repo.is_empty());
    }

    #[test]
    fn global_repository_initializes_once() {
        let dir = two_element_dir();
        let repo = init_global(dir.path()).unwrap();
        assert_eq!(repo.len(), 2);
        // A second call returns the cached repository and ignores the new path.
        let again = init_global("/no/such/directory").unwrap();
        assert_eq!(again.len(), 2);
        assert_eq!(global().map(|r| r.len()), Some(2));
    }
}
