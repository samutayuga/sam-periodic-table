//! Covalent stoichiometry from the octet rule, with the orbital-mismatch
//! double-bond rule and IUPAC ordering.

use super::math::gcd;

/// The number of atoms of each kind and the bond order in the molecule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CovalentStoich {
    pub n_a: i64,
    pub n_b: i64,
    pub bond_order: i64,
}

fn shell_target(ve: i64) -> i64 {
    if ve <= 2 {
        2
    } else {
        8
    }
}

fn bonds_needed(ve: i64) -> i64 {
    (shell_target(ve) - ve).max(0)
}

/// Pure octet covalent stoichiometry from valence-electron counts.
pub fn calc_stoich(ve_a: i64, ve_b: i64) -> CovalentStoich {
    let b_a = bonds_needed(ve_a);
    let b_b = bonds_needed(ve_b);
    if b_a == 0 || b_b == 0 {
        return CovalentStoich {
            n_a: 1,
            n_b: 1,
            bond_order: 1,
        };
    }
    let g = gcd(b_a, b_b);
    CovalentStoich {
        n_a: b_b / g,
        n_b: b_a / g,
        bond_order: g,
    }
}

/// True when two non-metals of the same group but different periods would, by the
/// octet rule alone, form a 1:1 double bond. Orbital-size mismatch makes that
/// simple double bond inefficient, so the structure resolves to one central + two
/// peripheral atoms. The "1:1 double bond" condition is only satisfiable by
/// valence-6 (group 16) atoms, so groups 14/15/17 are excluded automatically.
pub fn is_orbital_mismatch_double_bond(
    group_a: i64,
    period_a: i64,
    ve_a: i64,
    group_b: i64,
    period_b: i64,
    ve_b: i64,
) -> bool {
    if group_a != group_b || period_a == period_b {
        return false;
    }
    let base = calc_stoich(ve_a, ve_b);
    base.n_a == 1 && base.n_b == 1 && base.bond_order == 2
}

/// Covalent stoichiometry with the orbital-mismatch double-bond rule applied.
/// When the rule fires, the larger atom (higher period) is central (count 1) and
/// the smaller atom is peripheral (count 2), each bond a double bond. Otherwise
/// this is the pure octet [`calc_stoich`].
pub fn covalent_stoich(
    ve_a: i64,
    group_a: i64,
    period_a: i64,
    ve_b: i64,
    group_b: i64,
    period_b: i64,
) -> CovalentStoich {
    if is_orbital_mismatch_double_bond(group_a, period_a, ve_a, group_b, period_b, ve_b) {
        return if period_a > period_b {
            CovalentStoich {
                n_a: 1,
                n_b: 2,
                bond_order: 2,
            }
        } else {
            CovalentStoich {
                n_a: 2,
                n_b: 1,
                bond_order: 2,
            }
        };
    }
    calc_stoich(ve_a, ve_b)
}

/// IUPAC electronegativity-style ordering rank; unknown symbols rank 0.
fn iupac_rank(symbol: &str) -> i64 {
    match symbol {
        "B" => 1,
        "Si" => 2,
        "C" => 3,
        "Sb" => 4,
        "As" => 5,
        "P" => 6,
        "N" => 7,
        "H" => 8,
        "Te" => 9,
        "Se" => 10,
        "S" => 11,
        "O" => 12,
        "I" => 13,
        "Br" => 14,
        "Cl" => 15,
        "F" => 16,
        _ => 0,
    }
}

/// True when symbol A is written first (lower or equal IUPAC index; unknown
/// symbols rank 0, so equal-rank pairs keep A first).
pub fn iupac_first(symbol_a: &str, symbol_b: &str) -> bool {
    iupac_rank(symbol_a) <= iupac_rank(symbol_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(n_a: i64, n_b: i64, bond_order: i64) -> CovalentStoich {
        CovalentStoich {
            n_a,
            n_b,
            bond_order,
        }
    }

    #[test]
    fn hcl_single_bond() {
        assert_eq!(calc_stoich(1, 7), s(1, 1, 1));
    }

    #[test]
    fn water() {
        assert_eq!(calc_stoich(1, 6), s(2, 1, 1));
    }

    #[test]
    fn n2_triple_bond() {
        assert_eq!(calc_stoich(5, 5), s(1, 1, 3));
    }

    #[test]
    fn full_shell_falls_back() {
        assert_eq!(calc_stoich(8, 4), s(1, 1, 1));
    }

    #[test]
    fn iupac_ordering() {
        assert!(iupac_first("C", "O"));
        assert!(!iupac_first("O", "C"));
        assert!(iupac_first("B", "F"));
        assert!(iupac_first("Na", "Cl")); // both default 0 -> a first when equal
    }

    #[test]
    fn so2_central_sulfur() {
        assert_eq!(covalent_stoich(6, 16, 3, 6, 16, 2), s(1, 2, 2));
    }

    #[test]
    fn so2_slot_order_independent() {
        assert_eq!(covalent_stoich(6, 16, 2, 6, 16, 3), s(2, 1, 2));
    }

    #[test]
    fn ses2() {
        assert_eq!(covalent_stoich(6, 16, 4, 6, 16, 3), s(1, 2, 2));
    }

    #[test]
    fn o2_same_period_unchanged() {
        assert_eq!(covalent_stoich(6, 16, 2, 6, 16, 2), s(1, 1, 2));
    }

    #[test]
    fn clf_single_bond_unchanged() {
        assert_eq!(covalent_stoich(7, 17, 3, 7, 17, 2), s(1, 1, 1));
    }

    #[test]
    fn np_triple_bond_unchanged() {
        assert_eq!(covalent_stoich(5, 15, 3, 5, 15, 2), s(1, 1, 3));
    }

    #[test]
    fn different_group_unchanged() {
        assert_eq!(covalent_stoich(4, 14, 2, 6, 16, 2), s(1, 2, 2));
    }

    #[test]
    fn orbital_mismatch_truth_table() {
        assert!(is_orbital_mismatch_double_bond(16, 3, 6, 16, 2, 6)); // S+O
        assert!(!is_orbital_mismatch_double_bond(16, 2, 6, 16, 2, 6)); // O+O same period
        assert!(!is_orbital_mismatch_double_bond(17, 3, 7, 17, 2, 7)); // halogens single
        assert!(!is_orbital_mismatch_double_bond(14, 2, 4, 16, 2, 6)); // different group
    }
}
