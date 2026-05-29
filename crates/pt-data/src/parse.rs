//! Reading and parsing a single element YAML file.

use crate::error::DataError;
use crate::raw::RawElement;
use pt_domain::Element;
use std::path::Path;

/// Reads, parses, validates, and converts one element YAML file.
pub fn parse_element_file(path: &Path) -> Result<Element, DataError> {
    let file = path.display().to_string();
    let contents = std::fs::read_to_string(path).map_err(|e| DataError::Io {
        path: file.clone(),
        source: e,
    })?;
    let raw: RawElement = serde_yaml_ng::from_str(&contents).map_err(|e| DataError::Parse {
        file: file.clone(),
        source: e,
    })?;
    raw.into_element(&file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn parses_a_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("lithium.yaml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "atomic_number: 3\nname: Lithium\nsymbol: Li\natomic_mass: 6.94\nmass_number: 7\nstate: solid\n"
        )
        .unwrap();

        let element = parse_element_file(&path).unwrap();
        assert_eq!(element.name, "Lithium");
        assert_eq!(element.atomic_number, 3);
    }

    #[test]
    fn missing_file_is_io_error() {
        let err = parse_element_file(Path::new("/no/such/file.yaml")).unwrap_err();
        assert!(matches!(err, DataError::Io { .. }));
    }

    #[test]
    fn malformed_yaml_is_parse_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.yaml");
        std::fs::write(&path, "this: : : not valid").unwrap();
        let err = parse_element_file(&path).unwrap_err();
        assert!(matches!(err, DataError::Parse { .. }));
    }
}
