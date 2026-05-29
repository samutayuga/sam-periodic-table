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
        let entries = std::fs::read_dir(dir).map_err(|e| DataError::Io {
            path: dir.display().to_string(),
            source: e,
        })?;

        let mut elements = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| DataError::Io {
                path: dir.display().to_string(),
                source: e,
            })?;
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

        if elements.is_empty() {
            return Err(DataError::EmptyDataDir(dir.display().to_string()));
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

        Ok(Self {
            elements,
            by_number,
            by_symbol,
            by_name,
        })
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
}
