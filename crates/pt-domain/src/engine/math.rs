//! Small integer helpers shared by the reaction engine.

/// Greatest common divisor (Euclidean). `gcd(a, 0) == a`.
pub fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Least common multiple. `lcm(a, 0) == 0`.
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        a / gcd(a, b) * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_values() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(2, 2), 2);
        assert_eq!(gcd(3, 1), 1);
        assert_eq!(gcd(5, 0), 5);
        assert_eq!(gcd(6, 4), 2);
    }

    #[test]
    fn lcm_values() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(3, 0), 0);
        assert_eq!(lcm(0, 3), 0);
    }
}
