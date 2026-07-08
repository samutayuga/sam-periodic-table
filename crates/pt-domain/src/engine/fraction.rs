//! Exact rational, always stored reduced with a positive denominator.
//! Ports the iOS ChemCore `Fraction` used by the reaction balancer.

use crate::gcd;

/// Exact rational number, always reduced with a positive denominator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fraction {
    pub num: i64,
    pub den: i64,
}

impl Fraction {
    /// Builds a reduced fraction, moving the sign to the numerator.
    ///
    /// # Panics
    /// Panics if `den` is zero.
    pub fn new(num: i64, den: i64) -> Fraction {
        assert!(den != 0, "Fraction denominator must be non-zero");
        let sign = if den < 0 { -1 } else { 1 };
        let n = num * sign;
        let d = den.abs();
        if n == 0 {
            return Fraction { num: 0, den: 1 };
        }
        let g = gcd(n.abs(), d);
        Fraction {
            num: n / g,
            den: d / g,
        }
    }

    /// True when the fraction is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.num == 0
    }
}

impl std::ops::Add for Fraction {
    type Output = Fraction;

    fn add(self, other: Fraction) -> Fraction {
        Fraction::new(
            self.num * other.den + other.num * self.den,
            self.den * other.den,
        )
    }
}

impl std::ops::Mul for Fraction {
    type Output = Fraction;

    fn mul(self, other: Fraction) -> Fraction {
        Fraction::new(self.num * other.num, self.den * other.den)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces_on_init() {
        let f = Fraction::new(4, 8);
        assert_eq!(f.num, 1);
        assert_eq!(f.den, 2);
    }

    #[test]
    fn normalizes_sign_to_numerator() {
        let f = Fraction::new(1, -2);
        assert_eq!(f.num, -1);
        assert_eq!(f.den, 2);
    }

    #[test]
    fn addition() {
        assert_eq!(
            Fraction::new(1, 2) + Fraction::new(1, 3),
            Fraction::new(5, 6)
        );
    }

    #[test]
    fn multiplication() {
        assert_eq!(
            Fraction::new(2, 3) * Fraction::new(3, 4),
            Fraction::new(1, 2)
        );
    }

    #[test]
    fn is_zero() {
        assert!(Fraction::new(0, 5).is_zero());
        assert!(!Fraction::new(1, 5).is_zero());
    }
}
