//! CLI integration tests.

use assert_cmd::Command;
use predicates::str::contains;

fn data_dir() -> String {
    format!("{}/../../data/elements", env!("CARGO_MANIFEST_DIR"))
}

fn pt() -> Command {
    let mut cmd = Command::cargo_bin("pt").unwrap();
    cmd.args(["--data-dir", &data_dir()]);
    cmd
}

#[test]
fn get_by_symbol_prints_name_and_config() {
    pt().args(["get", "--symbol", "Fe"])
        .assert()
        .success()
        .stdout(contains("Iron"))
        .stdout(contains("3d6 4s2"));
}

#[test]
fn get_by_number_json_format() {
    pt().args(["--format", "json", "get", "--number", "1"])
        .assert()
        .success()
        .stdout(contains("\"symbol\": \"H\""))
        .stdout(contains("\"group\": 1"));
}

#[test]
fn get_by_mass_finds_nearest() {
    pt().args(["get", "--mass", "55.8"])
        .assert()
        .success()
        .stdout(contains("Iron"));
}

#[test]
fn unknown_element_exits_nonzero() {
    pt().args(["get", "--symbol", "Zz"])
        .assert()
        .failure()
        .stderr(contains("no matching element"));
}

#[test]
fn list_prints_multiple_elements() {
    pt().arg("list")
        .assert()
        .success()
        .stdout(contains("Hydrogen"))
        .stdout(contains("Iron"));
}

#[test]
fn get_by_name_works() {
    pt().args(["get", "--name", "iron"])
        .assert()
        .success()
        .stdout(contains("Iron"));
}

#[test]
fn get_with_no_selector_finds_nothing() {
    pt().arg("get")
        .assert()
        .failure()
        .stderr(contains("no matching element"));
}

#[test]
fn get_yaml_format() {
    pt().args(["--format", "yaml", "get", "--symbol", "O"])
        .assert()
        .success()
        .stdout(contains("symbol: O"));
}

#[test]
fn get_text_renders_null_fields_as_dash() {
    // Helium has no melting point / electronegativity, but does have a discoverer.
    pt().args(["get", "--symbol", "He"])
        .assert()
        .success()
        .stdout(contains("Helium"))
        .stdout(contains("melting point (K): —"))
        .stdout(contains("Pierre Janssen"));
}

#[test]
fn list_json_format() {
    pt().args(["--format", "json", "list"])
        .assert()
        .success()
        .stdout(contains("\"symbol\": \"H\""));
}

#[test]
fn list_yaml_format() {
    pt().args(["--format", "yaml", "list"])
        .assert()
        .success()
        .stdout(contains("symbol: H"));
}

#[test]
fn load_failure_exits_nonzero() {
    Command::cargo_bin("pt")
        .unwrap()
        .args(["--data-dir", "/no/such/directory", "get", "--number", "1"])
        .assert()
        .failure()
        .stderr(contains("error:"));
}
