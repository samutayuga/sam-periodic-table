//! Output representation for the CLI. Holds a flat, serializable snapshot of an
//! element's stored and computed properties (keeps serde out of the domain crate).

use pt_services::ElementView;
use serde::Serialize;

#[derive(Serialize)]
pub struct ElementOutput {
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
    pub electron_configuration: String,
    pub group: u8,
    pub period: u8,
    pub block: String,
    pub category: String,
    pub oxidation_states: Vec<i8>,
    pub computed_atomic_mass: Option<f64>,
}

impl ElementOutput {
    pub fn from_view(view: &ElementView) -> Self {
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
            state: format!("{:?}", e.state).to_lowercase(),
            discovery_year: e.discovery_year,
            discoverer: e.discoverer.clone(),
            electron_configuration: view.electron_configuration().to_string(),
            group: view.group(),
            period: view.period(),
            block: format!("{:?}", view.block()).to_lowercase(),
            category: format!("{:?}", view.category()),
            oxidation_states: view.oxidation_states().0,
            computed_atomic_mass: view.computed_atomic_mass(),
        }
    }
}

fn fmt_opt(value: Option<f64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "—".to_string())
}

pub fn print_text(o: &ElementOutput) {
    println!(
        "{} ({}) — atomic number {}",
        o.name, o.symbol, o.atomic_number
    );
    println!("  atomic mass:       {}", o.atomic_mass);
    println!("  mass number:       {}", o.mass_number);
    println!("  electron config:   {}", o.electron_configuration);
    println!("  group / period:    {} / {}", o.group, o.period);
    println!("  block:             {}", o.block);
    println!("  category:          {}", o.category);
    println!("  state (STP):       {}", o.state);
    println!("  melting point (K): {}", fmt_opt(o.melting_point));
    println!("  boiling point (K): {}", fmt_opt(o.boiling_point));
    println!("  density (g/cm³):   {}", fmt_opt(o.density));
    println!("  electronegativity: {}", fmt_opt(o.electronegativity));
    println!(
        "  oxidation states:  {}",
        o.oxidation_states
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let discoverer = o.discoverer.clone().unwrap_or_else(|| "—".to_string());
    let year = o
        .discovery_year
        .map(|y| y.to_string())
        .unwrap_or_else(|| "—".to_string());
    println!("  discovered:        {discoverer} ({year})");
}
