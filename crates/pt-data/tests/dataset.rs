//! Acceptance gate for the bundled dataset in `data/elements`.

use pt_data::ElementRepository;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/elements")
}

#[test]
fn dataset_loads_without_error() {
    // Must load and validate (uniqueness, ranges, abundance sums).
    ElementRepository::load_from_dir(data_dir()).expect("data/elements must load");
}

#[test]
fn dataset_covers_all_118_elements() {
    let repo = ElementRepository::load_from_dir(data_dir()).unwrap();
    assert_eq!(repo.len(), 118, "expected 118 element files");
    for z in 1..=118u8 {
        assert!(
            repo.get_by_atomic_number(z).is_some(),
            "missing element with atomic number {z}"
        );
    }
}
