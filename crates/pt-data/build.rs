use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RawState {
    Solid,
    Liquid,
    Gas,
}

#[derive(Debug, Deserialize)]
struct BuildIsotope {
    mass_number: u16,
    relative_mass: f64,
    abundance: f64,
}

#[derive(Debug, Deserialize)]
struct BuildElement {
    atomic_number: u8,
    name: String,
    symbol: String,
    atomic_mass: f64,
    mass_number: u16,
    #[serde(default)]
    melting_point: Option<f64>,
    #[serde(default)]
    boiling_point: Option<f64>,
    #[serde(default)]
    density: Option<f64>,
    #[serde(default)]
    electronegativity: Option<f64>,
    state: RawState,
    #[serde(default)]
    discovery_year: Option<i32>,
    #[serde(default)]
    discoverer: Option<String>,
    #[serde(default)]
    isotopes: Vec<BuildIsotope>,
}

fn opt_f64(v: Option<f64>) -> String {
    match v {
        Some(f) => format!("Some({f:?})"),
        None => "None".to_string(),
    }
}

fn opt_i32(v: Option<i32>) -> String {
    match v {
        Some(i) => format!("Some({i})"),
        None => "None".to_string(),
    }
}

fn opt_str(v: Option<&str>) -> String {
    match v {
        Some(s) => format!("Some({s:?})"),
        None => "None".to_string(),
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let data_dir = PathBuf::from(&manifest_dir).join("../../data/elements");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("generated_elements.rs");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", data_dir.display());

    let mut elements: Vec<BuildElement> = Vec::new();

    for entry in fs::read_dir(&data_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", data_dir.display()))
    {
        let path = entry.unwrap_or_else(|e| panic!("cannot read entry in {}: {e}", data_dir.display())).path();
        let is_yaml = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext == "yaml" || ext == "yml")
            .unwrap_or(false);
        if !is_yaml {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let element: BuildElement = serde_yaml_ng::from_str(&content)
            .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()));
        elements.push(element);
    }

    let mut out = fs::File::create(&out_path).unwrap();

    writeln!(out, "pub static ELEMENTS: &[StaticElement] = &[").unwrap();
    for e in &elements {
        let state_str = match e.state {
            RawState::Solid => "\"solid\"",
            RawState::Liquid => "\"liquid\"",
            RawState::Gas => "\"gas\"",
        };
        writeln!(out, "    StaticElement {{").unwrap();
        writeln!(out, "        atomic_number: {},", e.atomic_number).unwrap();
        writeln!(out, "        name: {:?},", e.name).unwrap();
        writeln!(out, "        symbol: {:?},", e.symbol).unwrap();
        writeln!(out, "        atomic_mass: {:?},", e.atomic_mass).unwrap();
        writeln!(out, "        mass_number: {},", e.mass_number).unwrap();
        writeln!(out, "        melting_point: {},", opt_f64(e.melting_point)).unwrap();
        writeln!(out, "        boiling_point: {},", opt_f64(e.boiling_point)).unwrap();
        writeln!(out, "        density: {},", opt_f64(e.density)).unwrap();
        writeln!(out, "        electronegativity: {},", opt_f64(e.electronegativity)).unwrap();
        writeln!(out, "        state: {state_str},").unwrap();
        writeln!(out, "        discovery_year: {},", opt_i32(e.discovery_year)).unwrap();
        writeln!(out, "        discoverer: {},", opt_str(e.discoverer.as_deref())).unwrap();
        writeln!(out, "        isotopes: &[").unwrap();
        for iso in &e.isotopes {
            writeln!(
                out,
                "            StaticIsotope {{ mass_number: {}, relative_mass: {:?}, abundance: {:?} }},",
                iso.mass_number, iso.relative_mass, iso.abundance
            )
            .unwrap();
        }
        writeln!(out, "        ],").unwrap();
        writeln!(out, "    }},").unwrap();
    }
    writeln!(out, "];").unwrap();
}
