//! Integration tests for the service API, against the bundled dataset.

use pt_services::PeriodicTable;

fn table() -> PeriodicTable {
    let dir = format!("{}/../../data/elements", env!("CARGO_MANIFEST_DIR"));
    PeriodicTable::load(dir).expect("dataset must load")
}

#[test]
fn lookup_by_atomic_number() {
    let t = table();
    let fe = t.by_atomic_number(26).expect("iron present");
    assert_eq!(fe.element().name, "Iron");
    assert_eq!(fe.group(), 8);
    assert_eq!(fe.period(), 4);
}

#[test]
fn lookup_by_symbol_is_case_insensitive() {
    let t = table();
    assert_eq!(t.by_symbol("fe").unwrap().element().name, "Iron");
    assert_eq!(t.by_symbol("FE").unwrap().element().name, "Iron");
}

#[test]
fn lookup_by_name_is_case_insensitive() {
    let t = table();
    assert_eq!(t.by_name("HYDROGEN").unwrap().element().symbol, "H");
}

#[test]
fn lookup_by_mass_within_tolerance() {
    let t = table();
    let v = t.by_atomic_mass(55.8, 0.5).expect("iron within tolerance");
    assert_eq!(v.element().symbol, "Fe");
}

#[test]
fn lookup_misses_return_none() {
    let t = table();
    assert!(t.by_atomic_number(150).is_none());
    assert!(t.by_symbol("Zz").is_none());
    assert!(t.by_atomic_mass(999.0, 0.1).is_none());
}

#[test]
fn computed_atomic_mass_matches_stored_for_chlorine() {
    let t = table();
    let cl = t.by_symbol("Cl").unwrap();
    let computed = cl.computed_atomic_mass().unwrap();
    assert!((computed - cl.element().atomic_mass).abs() < 0.05, "got {computed}");
}
