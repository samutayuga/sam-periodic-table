//! Output representation for the CLI. Holds a flat, serializable snapshot of an
//! element's stored and computed properties (keeps serde out of the domain crate).

use pt_services::ElementView;
use serde::Serialize;

/// A numeric property paired with its physical unit, so the unit travels with
/// the value in serialized output (e.g. `{ "value": 1.008, "unit": "u" }`).
#[derive(Serialize)]
pub struct Measurement {
    pub value: f64,
    pub unit: &'static str,
}

impl Measurement {
    fn new(value: f64, unit: &'static str) -> Self {
        Self { value, unit }
    }
}

#[derive(Serialize)]
pub struct ElementOutput {
    pub atomic_number: u8,
    pub name: String,
    pub symbol: String,
    pub atomic_mass: Measurement,
    pub mass_number: u16,
    pub melting_point: Option<Measurement>,
    pub boiling_point: Option<Measurement>,
    pub density: Option<Measurement>,
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
    pub computed_atomic_mass: Option<Measurement>,
}

impl ElementOutput {
    pub fn from_view(view: &ElementView) -> Self {
        let e = view.element();
        Self {
            atomic_number: e.atomic_number,
            name: e.name.clone(),
            symbol: e.symbol.clone(),
            atomic_mass: Measurement::new(e.atomic_mass, "u"),
            mass_number: e.mass_number,
            melting_point: e.melting_point.map(|v| Measurement::new(v, "K")),
            boiling_point: e.boiling_point.map(|v| Measurement::new(v, "K")),
            density: e.density.map(|v| Measurement::new(v, "g/cm³")),
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
            computed_atomic_mass: view
                .computed_atomic_mass()
                .map(|v| Measurement::new(v, "u")),
        }
    }
}

fn fmt_opt(value: Option<f64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "—".to_string())
}

fn fmt_measurement(m: &Measurement) -> String {
    format!("{} {}", m.value, m.unit)
}

fn fmt_opt_measurement(value: &Option<Measurement>) -> String {
    value
        .as_ref()
        .map(fmt_measurement)
        .unwrap_or_else(|| "—".to_string())
}

/// Renders subshell occupancies as Unicode superscripts for display,
/// e.g. "1s2 2s2 2p6" → "1s² 2s² 2p⁶". The principal quantum number (the
/// leading digit of each term) is left as an ordinary digit.
fn superscript_electrons(config: &str) -> String {
    let mut out = String::with_capacity(config.len());
    let mut after_subshell = false;
    for ch in config.chars() {
        match ch {
            's' | 'p' | 'd' | 'f' => {
                after_subshell = true;
                out.push(ch);
            }
            '0'..='9' if after_subshell => out.push(superscript_digit(ch)),
            ' ' => {
                after_subshell = false;
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn superscript_digit(d: char) -> char {
    match d {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        other => other,
    }
}

pub fn print_text(o: &ElementOutput) {
    println!(
        "{} ({}) — atomic number {}",
        o.name, o.symbol, o.atomic_number
    );
    println!("  atomic mass:       {}", fmt_measurement(&o.atomic_mass));
    println!("  mass number:       {}", o.mass_number);
    println!(
        "  electron config:   {}",
        superscript_electrons(&o.electron_configuration)
    );
    println!("  group / period:    {} / {}", o.group, o.period);
    println!("  block:             {}", o.block);
    println!("  category:          {}", o.category);
    println!("  state (STP):       {}", o.state);
    println!(
        "  melting point:     {}",
        fmt_opt_measurement(&o.melting_point)
    );
    println!(
        "  boiling point:     {}",
        fmt_opt_measurement(&o.boiling_point)
    );
    println!("  density:           {}", fmt_opt_measurement(&o.density));
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

#[cfg(test)]
mod tests {
    use super::superscript_electrons;

    #[test]
    fn superscripts_only_subshell_occupancies() {
        assert_eq!(
            superscript_electrons("1s2 2s2 2p6 3s2 3p6 3d6 4s2"),
            "1s² 2s² 2p⁶ 3s² 3p⁶ 3d⁶ 4s²"
        );
    }

    #[test]
    fn handles_two_digit_occupancies() {
        assert_eq!(superscript_electrons("4d10"), "4d¹⁰");
    }
}
