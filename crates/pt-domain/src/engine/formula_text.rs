//! Chemical-formula text helpers: Unicode subscripts, gcd-reduced ionic
//! crossover subscripts, and binary formula assembly. Ports the iOS
//! ChemCore `FormulaText.swift` helpers.

use crate::gcd;

/// Map an ASCII digit to its Unicode subscript equivalent.
fn subscript_digit(c: char) -> char {
    match c {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        other => other,
    }
}

/// Unicode subscript for a count; empty string when n <= 1.
pub fn formula_subscript(n: i64) -> String {
    if n <= 1 {
        return String::new();
    }
    n.to_string().chars().map(subscript_digit).collect()
}

/// gcd-reduced crossover subscripts from ionic charges (magnitudes crossed over).
pub fn crossover_subscripts(cation_charge: i64, anion_charge: i64) -> (i64, i64) {
    let cc = cation_charge.abs();
    let ac = anion_charge.abs();
    let g = gcd(cc, ac).max(1);
    (ac / g, cc / g)
}

/// A leading polyatomic cation (e.g. NH₄) needs parentheses when it repeats.
fn first_is_wrapped(symbol: &str) -> bool {
    symbol.chars().count() > 2
        && symbol
            .chars()
            .any(|c| c.is_numeric() || ('₀'..='₉').contains(&c))
}

/// Assemble a two-part formula. The second part is parenthesised only when it is a
/// polyatomic ion carrying a subscript > 1 (e.g. "(NH₄)₂SO₄", but "NaOH").
pub fn binary_formula(
    first: &str,
    first_count: i64,
    second: &str,
    second_count: i64,
    second_is_polyatomic: bool,
) -> String {
    let first_part = if first_is_wrapped(first) && first_count > 1 {
        format!("({first}){}", formula_subscript(first_count))
    } else {
        format!("{first}{}", formula_subscript(first_count))
    };
    let second_part = if second_is_polyatomic && second_count > 1 {
        format!("({second}){}", formula_subscript(second_count))
    } else {
        format!("{second}{}", formula_subscript(second_count))
    };
    first_part + &second_part
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscript_hides_one() {
        assert_eq!(formula_subscript(1), "");
        assert_eq!(formula_subscript(2), "₂");
        assert_eq!(formula_subscript(12), "₁₂");
    }

    #[test]
    fn crossover_nacl() {
        let (cation_sub, anion_sub) = crossover_subscripts(1, -1);
        assert_eq!(cation_sub, 1);
        assert_eq!(anion_sub, 1);
    }

    #[test]
    fn crossover_mgcl2() {
        let (cation_sub, anion_sub) = crossover_subscripts(2, -1);
        assert_eq!(cation_sub, 1);
        assert_eq!(anion_sub, 2);
    }

    #[test]
    fn crossover_reduces_al2o3_not_needed_but_ca_o() {
        let (cation_sub, anion_sub) = crossover_subscripts(2, -2);
        assert_eq!(cation_sub, 1);
        assert_eq!(anion_sub, 1);
    }

    #[test]
    fn binary_formula_simple() {
        assert_eq!(binary_formula("H", 2, "O", 1, false), "H₂O");
    }

    #[test]
    fn binary_formula_polyatomic_parenthesised() {
        assert_eq!(binary_formula("NH₄", 2, "SO₄", 1, true), "(NH₄)₂SO₄");
    }

    #[test]
    fn binary_formula_polyatomic_single_no_parens() {
        assert_eq!(binary_formula("Na", 1, "OH", 1, true), "NaOH");
    }
}
