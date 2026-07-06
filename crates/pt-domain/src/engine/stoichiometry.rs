//! Equation balancing and limiting-reactant solving for a binary synthesis
//! `coeffA·Aₚ + coeffB·B_q -> coeffProduct·AₓBᵧ`.

use super::math::{gcd, lcm};

/// Whether a reactant amount is given in moles or grams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityUnit {
    Mole,
    Mass,
}

/// A user-entered reactant amount.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReactantEntry {
    pub value: f64,
    pub unit: QuantityUnit,
}

/// A reactant's chemical identity plus an optional entered amount.
#[derive(Debug, Clone, PartialEq)]
pub struct ReactantSpec {
    pub symbol: String,
    pub atomic_mass: f64,
    /// The atom's subscript in the product formula (the `x` or `y` in `AₓBᵧ`).
    pub subscript_in_product: i64,
    pub is_diatomic: bool,
    pub entry: Option<ReactantEntry>,
}

/// Smallest-integer balanced coefficients and the reactant molecularities used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BalancedEquation {
    pub coeff_a: i64,
    pub coeff_b: i64,
    pub coeff_product: i64,
    pub molecularity_a: i64,
    pub molecularity_b: i64,
}

/// Which reactant runs out first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitingSide {
    A,
    B,
    Both,
}

/// A resolved amount in both moles and grams.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmountResult {
    pub moles: f64,
    pub mass: f64,
}

/// The full result of solving a binary synthesis.
#[derive(Debug, Clone, PartialEq)]
pub struct StoichResult {
    pub equation: BalancedEquation,
    pub product_molar_mass: f64,
    pub limiting: LimitingSide,
    pub product_yield: AmountResult,
    pub excess: AmountResult,
    pub diatomic_messages: Vec<String>,
}

/// The seven elements that exist only as diatomic molecules in nature.
pub const NATURALLY_DIATOMIC: [&str; 7] = ["H", "N", "O", "F", "Cl", "Br", "I"];

/// Whether `symbol` is one of the [`NATURALLY_DIATOMIC`] elements.
pub fn is_naturally_diatomic(symbol: &str) -> bool {
    NATURALLY_DIATOMIC.contains(&symbol)
}

/// Atoms per reactant unit: 2 for a diatomic element, 1 otherwise.
pub fn molecularity(is_diatomic: bool) -> i64 {
    if is_diatomic {
        2
    } else {
        1
    }
}

/// Balance `coeffA·Aₚ + coeffB·B_q -> coeffProduct·AₓBᵧ` for the smallest integers,
/// where `x`/`y` are the product subscripts and `p`/`q` the reactant molecularities.
pub fn balance_equation(
    subscript_a: i64,
    molecularity_a: i64,
    subscript_b: i64,
    molecularity_b: i64,
) -> BalancedEquation {
    let (x, p, y, q) = (subscript_a, molecularity_a, subscript_b, molecularity_b);
    let c0 = lcm(p / gcd(p, x), q / gcd(q, y));
    let mut a = c0 * x / p;
    let mut b = c0 * y / q;
    let mut c = c0;
    let g = gcd(gcd(a, b), c);
    a /= g;
    b /= g;
    c /= g;
    BalancedEquation {
        coeff_a: a,
        coeff_b: b,
        coeff_product: c,
        molecularity_a: p,
        molecularity_b: q,
    }
}

/// Solve the limiting reactant, theoretical yield and excess for a binary synthesis.
pub fn solve_stoichiometry(a: &ReactantSpec, b: &ReactantSpec) -> StoichResult {
    let p = molecularity(a.is_diatomic);
    let q = molecularity(b.is_diatomic);
    let eqn = balance_equation(a.subscript_in_product, p, b.subscript_in_product, q);

    let unit_mass_a = p as f64 * a.atomic_mass; // molar mass of Aₚ (X₂ when diatomic)
    let unit_mass_b = q as f64 * b.atomic_mass;
    let product_molar_mass = a.subscript_in_product as f64 * a.atomic_mass
        + b.subscript_in_product as f64 * b.atomic_mass;

    let moles_unit = |entry: &Option<ReactantEntry>, unit_mass: f64| -> Option<f64> {
        entry.map(|e| match e.unit {
            QuantityUnit::Mole => e.value,
            QuantityUnit::Mass => e.value / unit_mass,
        })
    };
    let mol_a = moles_unit(&a.entry, unit_mass_a);
    let mol_b = moles_unit(&b.entry, unit_mass_b);
    let extent_a = mol_a.map(|m| m / eqn.coeff_a as f64);
    let extent_b = mol_b.map(|m| m / eqn.coeff_b as f64);

    let (xi, limiting) = match (extent_a, extent_b) {
        (None, None) => (1.0, LimitingSide::Both),
        (Some(ea), None) => (ea, LimitingSide::A),
        (None, Some(eb)) => (eb, LimitingSide::B),
        (Some(ea), Some(eb)) => {
            if ea < eb {
                (ea, LimitingSide::A)
            } else if eb < ea {
                (eb, LimitingSide::B)
            } else {
                (ea, LimitingSide::Both)
            }
        }
    };

    let yield_moles = eqn.coeff_product as f64 * xi;
    let product_yield = AmountResult {
        moles: yield_moles,
        mass: yield_moles * product_molar_mass,
    };

    let mut excess = AmountResult {
        moles: 0.0,
        mass: 0.0,
    };
    if limiting == LimitingSide::A {
        if let Some(mb) = mol_b {
            let left = (mb - eqn.coeff_b as f64 * xi).max(0.0);
            excess = AmountResult {
                moles: left,
                mass: left * unit_mass_b,
            };
        }
    } else if limiting == LimitingSide::B {
        if let Some(ma) = mol_a {
            let left = (ma - eqn.coeff_a as f64 * xi).max(0.0);
            excess = AmountResult {
                moles: left,
                mass: left * unit_mass_a,
            };
        }
    }

    let mut diatomic_messages = Vec::new();
    if a.is_diatomic {
        diatomic_messages.push(format!(
            "{0} cannot exist as monoatomic, It only exist in {0}₂",
            a.symbol
        ));
    }
    if b.is_diatomic {
        diatomic_messages.push(format!(
            "{0} cannot exist as monoatomic, It only exist in {0}₂",
            b.symbol
        ));
    }

    StoichResult {
        equation: eqn,
        product_molar_mass,
        limiting,
        product_yield,
        excess,
        diatomic_messages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(
        symbol: &str,
        atomic_mass: f64,
        subscript: i64,
        diatomic: bool,
        entry: Option<ReactantEntry>,
    ) -> ReactantSpec {
        ReactantSpec {
            symbol: symbol.into(),
            atomic_mass,
            subscript_in_product: subscript,
            is_diatomic: diatomic,
            entry,
        }
    }

    fn mole(value: f64) -> Option<ReactantEntry> {
        Some(ReactantEntry {
            value,
            unit: QuantityUnit::Mole,
        })
    }

    fn mass(value: f64) -> Option<ReactantEntry> {
        Some(ReactantEntry {
            value,
            unit: QuantityUnit::Mass,
        })
    }

    fn coeffs(e: &BalancedEquation) -> [i64; 3] {
        [e.coeff_a, e.coeff_b, e.coeff_product]
    }

    #[test]
    fn balance_water() {
        // 2H₂ + O₂ -> 2H₂O
        let e = balance_equation(2, 2, 1, 2);
        assert_eq!(coeffs(&e), [2, 1, 2]);
    }

    #[test]
    fn balance_nacl() {
        // 2Na + Cl₂ -> 2NaCl
        let e = balance_equation(1, 1, 1, 2);
        assert_eq!(coeffs(&e), [2, 1, 2]);
    }

    #[test]
    fn balance_mgcl2() {
        // Mg + Cl₂ -> MgCl₂
        let e = balance_equation(1, 1, 2, 2);
        assert_eq!(coeffs(&e), [1, 1, 1]);
    }

    #[test]
    fn diatomic_set_and_molecularity() {
        assert_eq!(NATURALLY_DIATOMIC, ["H", "N", "O", "F", "Cl", "Br", "I"]);
        assert!(is_naturally_diatomic("Cl"));
        assert!(!is_naturally_diatomic("Na"));
        assert_eq!(molecularity(true), 2);
        assert_eq!(molecularity(false), 1);
    }

    #[test]
    fn yield_water_stoichiometric() {
        let h = spec("H", 1.0, 2, true, mole(2.0));
        let o = spec("O", 16.0, 1, true, mole(1.0));
        let r = solve_stoichiometry(&h, &o);
        assert_eq!(r.limiting, LimitingSide::Both);
        assert!((r.product_yield.moles - 2.0).abs() < 1e-9);
        assert!((r.product_molar_mass - 18.0).abs() < 1e-9);
        assert!((r.product_yield.mass - 36.0).abs() < 1e-9);
        assert!((r.excess.moles - 0.0).abs() < 1e-9);
    }

    #[test]
    fn excess_hydrogen() {
        let h = spec("H", 1.0, 2, true, mole(3.0));
        let o = spec("O", 16.0, 1, true, mole(1.0));
        let r = solve_stoichiometry(&h, &o);
        assert_eq!(r.limiting, LimitingSide::B);
        assert!((r.product_yield.moles - 2.0).abs() < 1e-9);
        assert!((r.excess.moles - 1.0).abs() < 1e-9);
        assert!((r.excess.mass - 2.0).abs() < 1e-9); // 1 mol H₂ × 2 g/mol
    }

    #[test]
    fn mass_unit_conversion() {
        // 32 g O₂ = 1 mol O₂ ; paired with 4 mol H₂ -> O limiting, yield 2 mol H₂O
        let h = spec("H", 1.0, 2, true, mole(4.0));
        let o = spec("O", 16.0, 1, true, mass(32.0));
        let r = solve_stoichiometry(&h, &o);
        assert_eq!(r.limiting, LimitingSide::B);
        assert!((r.product_yield.moles - 2.0).abs() < 1e-9);
    }

    #[test]
    fn blank_b_means_a_limiting() {
        let h = spec("H", 1.0, 2, true, mole(2.0));
        let o = spec("O", 16.0, 1, true, None);
        let r = solve_stoichiometry(&h, &o);
        assert_eq!(r.limiting, LimitingSide::A);
        assert!((r.product_yield.moles - 2.0).abs() < 1e-9);
        assert!((r.excess.moles - 0.0).abs() < 1e-9);
    }

    #[test]
    fn both_blank_one_mol_basis() {
        let h = spec("H", 1.0, 2, true, None);
        let o = spec("O", 16.0, 1, true, None);
        let r = solve_stoichiometry(&h, &o);
        assert_eq!(r.limiting, LimitingSide::Both);
        assert!((r.product_yield.moles - 2.0).abs() < 1e-9); // coeffProduct × ξ(=1)
    }

    #[test]
    fn diatomic_messages_only_for_diatomic() {
        let na = spec("Na", 23.0, 1, false, mole(1.0));
        let cl = spec("Cl", 35.45, 1, true, mole(1.0));
        let r = solve_stoichiometry(&na, &cl);
        assert_eq!(
            r.diatomic_messages,
            vec!["Cl cannot exist as monoatomic, It only exist in Cl₂".to_string()]
        );
    }

    #[test]
    fn excess_never_negative() {
        let h = spec("H", 1.0, 2, true, mole(2.0));
        let o = spec("O", 16.0, 1, true, mole(1.0));
        let r = solve_stoichiometry(&h, &o);
        assert!(r.excess.moles >= 0.0);
        assert!((r.excess.moles - 0.0).abs() < 1e-9);
    }
}
