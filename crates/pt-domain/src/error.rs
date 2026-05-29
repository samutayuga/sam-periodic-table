//! Domain calculation errors.

/// Error returned by domain calculations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    /// The atomic number is outside the supported range `1..=118`.
    #[error("invalid atomic number: {0} (must be 1..=118)")]
    InvalidAtomicNumber(u16),
}
