//! The common polyatomic ions offered as compound reactants.

/// A polyatomic ion: a charged group of covalently bonded atoms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyatomicIon {
    /// Bare atom-group symbol, e.g. `"SO₄"`.
    pub symbol: &'static str,
    pub name: &'static str,
    pub charge: i8,
    /// Display formula including the charge, e.g. `"SO₄²⁻"`.
    pub formula: &'static str,
}

/// The six common polyatomic ions, matching the iOS reference table.
pub const POLYATOMIC_IONS: [PolyatomicIon; 6] = [
    PolyatomicIon {
        symbol: "OH",
        name: "Hydroxide",
        charge: -1,
        formula: "OH⁻",
    },
    PolyatomicIon {
        symbol: "NO₃",
        name: "Nitrate",
        charge: -1,
        formula: "NO₃⁻",
    },
    PolyatomicIon {
        symbol: "SO₄",
        name: "Sulfate",
        charge: -2,
        formula: "SO₄²⁻",
    },
    PolyatomicIon {
        symbol: "CO₃",
        name: "Carbonate",
        charge: -2,
        formula: "CO₃²⁻",
    },
    PolyatomicIon {
        symbol: "PO₄",
        name: "Phosphate",
        charge: -3,
        formula: "PO₄³⁻",
    },
    PolyatomicIon {
        symbol: "NH₄",
        name: "Ammonium",
        charge: 1,
        formula: "NH₄⁺",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_six_ions() {
        assert_eq!(POLYATOMIC_IONS.len(), 6);
    }

    #[test]
    fn sulfate_entry() {
        let sulfate = POLYATOMIC_IONS
            .iter()
            .find(|i| i.symbol == "SO₄")
            .expect("sulfate present");
        assert_eq!(sulfate.name, "Sulfate");
        assert_eq!(sulfate.charge, -2);
        assert_eq!(sulfate.formula, "SO₄²⁻");
    }

    #[test]
    fn ammonium_is_the_only_cation() {
        let cations: Vec<_> = POLYATOMIC_IONS.iter().filter(|i| i.charge > 0).collect();
        assert_eq!(cations.len(), 1);
        assert_eq!(cations[0].symbol, "NH₄");
    }
}
