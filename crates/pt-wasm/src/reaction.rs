use pt_domain::{
    bonding_type, covalent_stoich, metallic_electron_count, parse_valence_electrons,
    predict_product_state, reaction_glyph, BondingType,
};
use pt_services::PeriodicTable as ServiceTable;
use serde::Serialize;
use tsify::Tsify;

/// Covalent structure of a binary molecule: atom counts and bond order.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmCovalentStoich {
    pub n_a: i32,
    pub n_b: i32,
    pub bond_order: i32,
}

/// The classified outcome of combining two elemental reactants.
///
/// `covalent` is populated only for covalent products; `metallic_electrons`
/// (delocalised electron-sea count) only for metallic products.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmReaction {
    /// "Ionic", "Covalent", or "Metallic".
    pub bonding: String,
    /// Reaction-arrow glyph between the reactant slots ("→" or "⇌").
    pub glyph: String,
    /// Heuristic standard-state of the product: "Solid", "Liquid", or "Gas".
    pub product_state: String,
    pub covalent: Option<WasmCovalentStoich>,
    pub metallic_electrons: Option<i32>,
}

/// Highest-shell valence electrons for `symbol`, or `None` if unknown.
pub(crate) fn valence_electrons(table: &ServiceTable, symbol: &str) -> Option<u8> {
    let view = table.by_symbol(symbol)?;
    Some(parse_valence_electrons(
        &view.electron_configuration().to_string(),
        view.group(),
    ))
}

/// Classify the synthesis of two elemental reactants given by symbol.
/// Returns `None` if either symbol is not a known element.
pub(crate) fn react(table: &ServiceTable, a: &str, b: &str) -> Option<WasmReaction> {
    let va = table.by_symbol(a)?;
    let vb = table.by_symbol(b)?;

    // Both inputs are bare elements here (not polyatomic ions).
    let bonding = bonding_type(va.element_class(), vb.element_class(), false, false);
    let glyph = reaction_glyph(Some(bonding));
    let product_state = predict_product_state(
        bonding,
        &va.element().symbol,
        va.element().state,
        &vb.element().symbol,
        vb.element().state,
    );

    let ve_a = parse_valence_electrons(&va.electron_configuration().to_string(), va.group());
    let ve_b = parse_valence_electrons(&vb.electron_configuration().to_string(), vb.group());

    let covalent = if bonding == BondingType::Covalent {
        let cs = covalent_stoich(
            ve_a as i64,
            va.group() as i64,
            va.period() as i64,
            ve_b as i64,
            vb.group() as i64,
            vb.period() as i64,
        );
        Some(WasmCovalentStoich {
            n_a: cs.n_a as i32,
            n_b: cs.n_b as i32,
            bond_order: cs.bond_order as i32,
        })
    } else {
        None
    };

    let metallic_electrons = if bonding == BondingType::Metallic {
        Some(metallic_electron_count(ve_a as i64, ve_b as i64) as i32)
    } else {
        None
    };

    Some(WasmReaction {
        bonding: bonding.label().to_string(),
        glyph: glyph.to_string(),
        product_state: product_state.label().to_string(),
        covalent,
        metallic_electrons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> ServiceTable {
        ServiceTable::load_bundled().unwrap()
    }

    #[test]
    fn valence_of_chlorine_is_seven() {
        assert_eq!(valence_electrons(&table(), "Cl"), Some(7));
    }

    #[test]
    fn valence_of_iron_is_two() {
        assert_eq!(valence_electrons(&table(), "Fe"), Some(2));
    }

    #[test]
    fn valence_unknown_is_none() {
        assert_eq!(valence_electrons(&table(), "XX"), None);
    }

    #[test]
    fn metal_plus_nonmetal_is_ionic() {
        let r = react(&table(), "Na", "Cl").unwrap();
        assert_eq!(r.bonding, "Ionic");
        assert_eq!(r.glyph, "→");
        assert_eq!(r.product_state, "Solid"); // ionic lattice
        assert!(r.covalent.is_none());
        assert!(r.metallic_electrons.is_none());
    }

    #[test]
    fn metal_plus_metal_is_metallic() {
        let r = react(&table(), "Na", "Mg").unwrap();
        assert_eq!(r.bonding, "Metallic");
        assert_eq!(r.product_state, "Solid");
        assert_eq!(r.metallic_electrons, Some(9)); // 3·veNa(1) + 3·veMg(2)
        assert!(r.covalent.is_none());
    }

    #[test]
    fn sulfur_plus_oxygen_is_covalent_so2() {
        // S (group 16, period 3) + O (group 16, period 2): orbital-mismatch -> SO₂.
        let r = react(&table(), "S", "O").unwrap();
        assert_eq!(r.bonding, "Covalent");
        assert_eq!(r.glyph, "⇌");
        assert_eq!(r.product_state, "Gas"); // O is gaseous constituent
        let cov = r.covalent.unwrap();
        assert_eq!((cov.n_a, cov.n_b, cov.bond_order), (1, 2, 2));
    }

    #[test]
    fn hydrogen_plus_oxygen_water_is_liquid() {
        let r = react(&table(), "H", "O").unwrap();
        assert_eq!(r.bonding, "Covalent");
        assert_eq!(r.product_state, "Liquid"); // H₂O special case
    }

    #[test]
    fn react_unknown_symbol_is_none() {
        assert!(react(&table(), "Na", "XX").is_none());
    }
}
