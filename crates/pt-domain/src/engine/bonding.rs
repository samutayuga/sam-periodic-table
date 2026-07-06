//! Bond-type classification for a two-reactant synthesis.

use crate::classification::ElementClass;

/// The kind of chemical bond formed between two reactants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondingType {
    Ionic,
    Covalent,
    Metallic,
}

impl BondingType {
    /// The spec label used in UI badges.
    pub fn label(self) -> &'static str {
        match self {
            BondingType::Ionic => "Ionic",
            BondingType::Covalent => "Covalent",
            BondingType::Metallic => "Metallic",
        }
    }
}

/// Reaction-arrow glyph for the bridge between the two reactant slots.
///
/// `None` (not yet classified) shows a plus; ionic and metallic syntheses go to
/// completion (`→`); covalent molecular synthesis often reaches equilibrium (`⇌`).
pub fn reaction_glyph(bonding: Option<BondingType>) -> &'static str {
    match bonding {
        None => "+",
        Some(BondingType::Ionic) | Some(BondingType::Metallic) => "→",
        Some(BondingType::Covalent) => "⇌",
    }
}

/// Bond type from the two broad element classes alone.
pub fn determine_bonding(a: ElementClass, b: ElementClass) -> BondingType {
    use ElementClass::*;
    if a == Metal && b == Metal {
        return BondingType::Metallic;
    }
    if matches!(a, Metalloid | NonMetal) && matches!(b, Metalloid | NonMetal) {
        return BondingType::Covalent;
    }
    BondingType::Ionic
}

/// Bond type accounting for polyatomic ions, which always bond ionically.
pub fn bonding_type(
    a_class: ElementClass,
    b_class: ElementClass,
    a_polyatomic: bool,
    b_polyatomic: bool,
) -> BondingType {
    if a_polyatomic || b_polyatomic {
        return BondingType::Ionic;
    }
    determine_bonding(a_class, b_class)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classification::ElementClass::*;

    #[test]
    fn determine_bonding_table() {
        assert_eq!(determine_bonding(Metal, Metal), BondingType::Metallic);
        assert_eq!(determine_bonding(NonMetal, NonMetal), BondingType::Covalent);
        assert_eq!(
            determine_bonding(Metalloid, NonMetal),
            BondingType::Covalent
        );
        assert_eq!(
            determine_bonding(Metalloid, Metalloid),
            BondingType::Covalent
        );
        assert_eq!(determine_bonding(Metal, NonMetal), BondingType::Ionic);
        assert_eq!(determine_bonding(Metal, Metalloid), BondingType::Ionic);
    }

    #[test]
    fn polyatomic_always_ionic() {
        assert_eq!(
            bonding_type(NonMetal, NonMetal, true, false),
            BondingType::Ionic
        );
        assert_eq!(bonding_type(Metal, Metal, false, true), BondingType::Ionic);
        assert_eq!(
            bonding_type(Metal, Metal, false, false),
            BondingType::Metallic
        );
    }

    #[test]
    fn reaction_glyphs() {
        assert_eq!(reaction_glyph(None), "+");
        assert_eq!(reaction_glyph(Some(BondingType::Ionic)), "→");
        assert_eq!(reaction_glyph(Some(BondingType::Metallic)), "→");
        assert_eq!(reaction_glyph(Some(BondingType::Covalent)), "⇌");
    }
}
