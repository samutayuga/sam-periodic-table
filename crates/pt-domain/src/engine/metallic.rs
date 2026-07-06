//! Electron-sea model for metallic bonding.

/// Default delocalised-electron pool size for the electron-sea visualisation.
pub const DEFAULT_POOL_SIZE: i64 = 12;

/// Delocalised electron count for the electron-sea model, capped at `pool_size`.
pub fn metallic_electron_count_capped(ve_a: i64, ve_b: i64, pool_size: i64) -> i64 {
    (3 * ve_a + 3 * ve_b).min(pool_size)
}

/// Delocalised electron count using the [`DEFAULT_POOL_SIZE`] cap.
pub fn metallic_electron_count(ve_a: i64, ve_b: i64) -> i64 {
    metallic_electron_count_capped(ve_a, ve_b, DEFAULT_POOL_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn electron_count() {
        assert_eq!(metallic_electron_count(1, 1), 6); // 3 + 3
        assert_eq!(metallic_electron_count(2, 2), 12); // 6 + 6
        assert_eq!(metallic_electron_count(3, 3), 12); // 18 capped to 12
        assert_eq!(metallic_electron_count(1, 0), 3);
    }

    #[test]
    fn custom_pool_size() {
        assert_eq!(metallic_electron_count_capped(3, 3, 18), 18);
    }
}
