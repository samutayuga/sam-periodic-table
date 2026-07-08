//! Metal and halogen activity series, and single-displacement outranking.
//! Ports the iOS ChemCore `ActivitySeries.swift`.

/// Metal reactivity, most reactive first (standard school activity series, H included).
pub const METAL_ACTIVITY_SERIES: &[&str] = &[
    "K", "Na", "Li", "Ca", "Mg", "Al", "Zn", "Fe", "Ni", "Sn", "Pb", "H", "Cu", "Ag", "Hg", "Au",
];

/// Halogen reactivity, most reactive first.
pub const HALOGEN_ACTIVITY_SERIES: &[&str] = &["F", "Cl", "Br", "I"];

/// `Some(true)` when `free` can displace `bound` from a compound: `free` is
/// higher (more reactive) in a shared series. `Some(false)` when both are in
/// a shared series but `free` does not outrank `bound` (including the same
/// element). `None` when the two share no series.
pub fn displaces(free: &str, bound: &str) -> Option<bool> {
    for series in [METAL_ACTIVITY_SERIES, HALOGEN_ACTIVITY_SERIES] {
        let f = series.iter().position(|&s| s == free);
        let b = series.iter().position(|&s| s == bound);
        if let (Some(f), Some(b)) = (f, b) {
            return Some(f < b);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metal_ordering() {
        let zn = METAL_ACTIVITY_SERIES
            .iter()
            .position(|&s| s == "Zn")
            .unwrap();
        let cu = METAL_ACTIVITY_SERIES
            .iter()
            .position(|&s| s == "Cu")
            .unwrap();
        assert!(zn < cu);
    }

    #[test]
    fn test_zn_displaces_cu() {
        assert_eq!(displaces("Zn", "Cu"), Some(true));
    }

    #[test]
    fn test_cu_does_not_displace_zn() {
        assert_eq!(displaces("Cu", "Zn"), Some(false));
    }

    #[test]
    fn test_halogen_ordering() {
        assert_eq!(displaces("Cl", "Br"), Some(true));
        assert_eq!(displaces("I", "Cl"), Some(false));
    }

    #[test]
    fn test_unrelated_pair_nil() {
        assert_eq!(displaces("Zn", "Cl"), None);
    }

    #[test]
    fn test_same_element_false() {
        assert_eq!(displaces("Zn", "Zn"), Some(false));
    }
}
