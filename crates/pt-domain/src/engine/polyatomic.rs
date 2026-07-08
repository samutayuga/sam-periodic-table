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
    /// Element composition as (element symbol, atom count).
    pub composition: &'static [(&'static str, u8)],
}

/// The six common polyatomic ions, matching the iOS reference table.
pub const POLYATOMIC_IONS: [PolyatomicIon; 6] = [
    PolyatomicIon {
        symbol: "OH",
        name: "Hydroxide",
        charge: -1,
        formula: "OH⁻",
        composition: &[("O", 1), ("H", 1)],
    },
    PolyatomicIon {
        symbol: "NO₃",
        name: "Nitrate",
        charge: -1,
        formula: "NO₃⁻",
        composition: &[("N", 1), ("O", 3)],
    },
    PolyatomicIon {
        symbol: "SO₄",
        name: "Sulfate",
        charge: -2,
        formula: "SO₄²⁻",
        composition: &[("S", 1), ("O", 4)],
    },
    PolyatomicIon {
        symbol: "CO₃",
        name: "Carbonate",
        charge: -2,
        formula: "CO₃²⁻",
        composition: &[("C", 1), ("O", 3)],
    },
    PolyatomicIon {
        symbol: "PO₄",
        name: "Phosphate",
        charge: -3,
        formula: "PO₄³⁻",
        composition: &[("P", 1), ("O", 4)],
    },
    PolyatomicIon {
        symbol: "NH₄",
        name: "Ammonium",
        charge: 1,
        formula: "NH₄⁺",
        composition: &[("N", 1), ("H", 4)],
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

    #[test]
    fn sulfate_composition() {
        let so4 = POLYATOMIC_IONS.iter().find(|i| i.symbol == "SO₄").unwrap();
        assert_eq!(so4.composition, &[("S", 1), ("O", 4)]);
    }
    #[test]
    fn hydroxide_composition() {
        let oh = POLYATOMIC_IONS.iter().find(|i| i.symbol == "OH").unwrap();
        assert_eq!(oh.composition, &[("O", 1), ("H", 1)]);
    }
    #[test]
    fn ammonium_composition() {
        let nh4 = POLYATOMIC_IONS.iter().find(|i| i.symbol == "NH₄").unwrap();
        assert_eq!(nh4.composition, &[("N", 1), ("H", 4)]);
    }
}
