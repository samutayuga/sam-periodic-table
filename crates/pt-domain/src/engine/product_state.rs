//! Heuristic standard-state of a synthesised product compound.

use super::bonding::BondingType;
use crate::element::StateOfMatter;

/// The standard-state physical form of a result compound, for the result badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductState {
    Solid,
    Liquid,
    Gas,
}

impl ProductState {
    pub fn label(self) -> &'static str {
        match self {
            ProductState::Solid => "Solid",
            ProductState::Liquid => "Liquid",
            ProductState::Gas => "Gas",
        }
    }
}

/// Heuristic standard-state (~25 °C, 1 atm) of the product compound. Deliberately
/// approximate, for teaching:
/// - **Ionic** and **metallic** products are extended lattices → solid.
/// - **Covalent** products are discrete molecules, so the state is estimated from
///   the constituent elements' own standard states: any gaseous constituent → gas
///   (CO₂, SO₂, O₂), else any liquid constituent → liquid, else solid. Water is
///   special-cased to liquid since the light-element gas guess would otherwise miss it.
pub fn predict_product_state(
    bonding: BondingType,
    a_symbol: &str,
    a_state: StateOfMatter,
    b_symbol: &str,
    b_state: StateOfMatter,
) -> ProductState {
    match bonding {
        BondingType::Ionic | BondingType::Metallic => ProductState::Solid,
        BondingType::Covalent => {
            // H₂O: light-element gas guess would miss it, so special-case to liquid.
            if matches!((a_symbol, b_symbol), ("H", "O") | ("O", "H")) {
                return ProductState::Liquid;
            }
            let states = [a_state, b_state];
            if states.contains(&StateOfMatter::Gas) {
                ProductState::Gas
            } else if states.contains(&StateOfMatter::Liquid) {
                ProductState::Liquid
            } else {
                ProductState::Solid
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use StateOfMatter::*;

    #[test]
    fn ionic_is_solid() {
        assert_eq!(
            predict_product_state(BondingType::Ionic, "Na", Solid, "Cl", Gas),
            ProductState::Solid
        );
    }

    #[test]
    fn metallic_is_solid() {
        assert_eq!(
            predict_product_state(BondingType::Metallic, "Na", Solid, "Mg", Solid),
            ProductState::Solid
        );
    }

    #[test]
    fn covalent_any_gas_constituent_is_gas() {
        assert_eq!(
            predict_product_state(BondingType::Covalent, "C", Solid, "O", Gas),
            ProductState::Gas
        );
        assert_eq!(
            predict_product_state(BondingType::Covalent, "S", Solid, "O", Gas),
            ProductState::Gas
        );
    }

    #[test]
    fn covalent_both_gas_is_gas() {
        assert_eq!(
            predict_product_state(BondingType::Covalent, "O", Gas, "O", Gas),
            ProductState::Gas
        );
    }

    #[test]
    fn covalent_water_special_cased_liquid() {
        assert_eq!(
            predict_product_state(BondingType::Covalent, "H", Gas, "O", Gas),
            ProductState::Liquid
        );
        assert_eq!(
            predict_product_state(BondingType::Covalent, "O", Gas, "H", Gas),
            ProductState::Liquid
        );
    }

    #[test]
    fn covalent_liquid_constituent_is_liquid() {
        assert_eq!(
            predict_product_state(BondingType::Covalent, "Br", Liquid, "I", Solid),
            ProductState::Liquid
        );
    }

    #[test]
    fn covalent_all_solid_is_solid() {
        assert_eq!(
            predict_product_state(BondingType::Covalent, "Si", Solid, "C", Solid),
            ProductState::Solid
        );
    }
}
