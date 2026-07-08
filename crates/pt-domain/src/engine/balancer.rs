//! General integer chemical-equation balancer.
//!
//! Ports the iOS ChemCore `Balancer.swift`: builds an element x species matrix
//! (reactants positive, products negative), reduces it to row echelon form
//! over exact rationals, requires exactly one free column for a unique
//! solution ratio, then scales to the smallest positive integer coefficients.

use std::collections::{BTreeMap, BTreeSet};

use crate::{gcd, lcm, Fraction};

/// Balance a reaction to the smallest positive integer coefficients, ordered
/// reactants-then-products. Returns `None` when no all-positive solution
/// exists (including degenerate/underdetermined/overdetermined systems).
pub fn balance(
    reactants: &[BTreeMap<String, i64>],
    products: &[BTreeMap<String, i64>],
) -> Option<Vec<i64>> {
    let species: Vec<&BTreeMap<String, i64>> = reactants.iter().chain(products.iter()).collect();
    let n = species.len();
    if n < 2 {
        return None;
    }

    // Distinct elements → matrix rows. Reactants positive, products negative.
    let elements: BTreeSet<&String> = species.iter().flat_map(|s| s.keys()).collect();
    let mut m: Vec<Vec<Fraction>> = elements
        .iter()
        .map(|el| {
            (0..n)
                .map(|j| {
                    let count = species[j].get(*el).copied().unwrap_or(0);
                    let sign: i64 = if j < reactants.len() { 1 } else { -1 };
                    Fraction::new(sign * count, 1)
                })
                .collect()
        })
        .collect();

    // Gaussian elimination to reduced row echelon form.
    let mut pivot_cols: Vec<usize> = Vec::new();
    let mut row = 0usize;
    for col in 0..n {
        let pivot = match (row..m.len()).find(|&r| !m[r][col].is_zero()) {
            Some(p) => p,
            None => continue,
        };
        m.swap(row, pivot);
        let inv = Fraction::new(m[row][col].den, m[row][col].num);
        m[row] = m[row].iter().map(|f| *f * inv).collect();
        let pivot_row = m[row].clone();
        for (r, m_row) in m.iter_mut().enumerate() {
            if r != row && !m_row[col].is_zero() {
                let factor = m_row[col];
                let neg_factor = Fraction::new(-factor.num, factor.den);
                *m_row = m_row
                    .iter()
                    .zip(pivot_row.iter())
                    .map(|(a, b)| *a + neg_factor * *b)
                    .collect();
            }
        }
        pivot_cols.push(col);
        row += 1;
        if row == m.len() {
            break;
        }
    }

    // Exactly one free column → unique ratio. Otherwise reject.
    let free_cols: Vec<usize> = (0..n).filter(|c| !pivot_cols.contains(c)).collect();
    if free_cols.len() != 1 {
        return None;
    }
    let free = free_cols[0];

    // Set free variable = 1; pivots = -matrix[pivotRow][free].
    let mut solution = vec![Fraction::new(0, 1); n];
    solution[free] = Fraction::new(1, 1);
    for (row_index, &col) in pivot_cols.iter().enumerate() {
        let pivot_val = m[row_index][free];
        solution[col] = Fraction::new(-pivot_val.num, pivot_val.den);
    }

    // Scale to integers: multiply by LCM of denominators.
    let den_lcm = solution.iter().fold(1i64, |acc, f| lcm(acc, f.den));
    let mut ints: Vec<i64> = solution.iter().map(|f| f.num * (den_lcm / f.den)).collect();

    // Normalise sign so coefficients are positive, then divide by GCD.
    if ints.iter().any(|&x| x < 0) && ints.iter().all(|&x| x <= 0) {
        ints = ints.iter().map(|&x| -x).collect();
    }
    if !ints.iter().all(|&x| x > 0) {
        return None;
    }
    let g = ints.iter().fold(0i64, |acc, &x| gcd(acc, x));
    if g <= 0 {
        return None;
    }
    Some(ints.iter().map(|&x| x / g).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, i64)]) -> BTreeMap<String, i64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn water_synthesis() {
        // H2 + O2 -> H2O => [2,1,2]
        let reactants = vec![map(&[("H", 2)]), map(&[("O", 2)])];
        let products = vec![map(&[("H", 2), ("O", 1)])];
        assert_eq!(balance(&reactants, &products), Some(vec![2, 1, 2]));
    }

    #[test]
    fn neutralisation() {
        // NaOH + HCl -> NaCl + H2O => [1,1,1,1]
        let reactants = vec![
            map(&[("Na", 1), ("O", 1), ("H", 1)]),
            map(&[("H", 1), ("Cl", 1)]),
        ];
        let products = vec![map(&[("Na", 1), ("Cl", 1)]), map(&[("H", 2), ("O", 1)])];
        assert_eq!(balance(&reactants, &products), Some(vec![1, 1, 1, 1]));
    }

    #[test]
    fn combustion_methane() {
        // CH4 + O2 -> CO2 + H2O => [1,2,1,2]
        let reactants = vec![map(&[("C", 1), ("H", 4)]), map(&[("O", 2)])];
        let products = vec![map(&[("C", 1), ("O", 2)]), map(&[("H", 2), ("O", 1)])];
        assert_eq!(balance(&reactants, &products), Some(vec![1, 2, 1, 2]));
    }

    #[test]
    fn carbonate_acid() {
        // 2HCl + Na2CO3 -> 2NaCl + CO2 + H2O => [2,1,2,1,1]
        let reactants = vec![
            map(&[("H", 1), ("Cl", 1)]),
            map(&[("Na", 2), ("C", 1), ("O", 3)]),
        ];
        let products = vec![
            map(&[("Na", 1), ("Cl", 1)]),
            map(&[("C", 1), ("O", 2)]),
            map(&[("H", 2), ("O", 1)]),
        ];
        assert_eq!(balance(&reactants, &products), Some(vec![2, 1, 2, 1, 1]));
    }

    #[test]
    fn unbalanceable_returns_none() {
        // Element on the left with no home on the right.
        let reactants = vec![map(&[("Na", 1)])];
        let products = vec![map(&[("Cl", 1)])];
        assert_eq!(balance(&reactants, &products), None);
    }
}
