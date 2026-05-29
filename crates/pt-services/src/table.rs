//! The top-level query API.

use crate::error::ServiceError;
use crate::view::ElementView;
use pt_data::ElementRepository;
use std::path::Path;

/// Loaded periodic table providing lookups by several keys.
pub struct PeriodicTable {
    repo: ElementRepository,
}

impl PeriodicTable {
    /// Loads element data from a directory of YAML files (once).
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ServiceError> {
        Ok(Self {
            repo: ElementRepository::load_from_dir(dir)?,
        })
    }

    pub fn by_atomic_number(&self, z: u8) -> Option<ElementView<'_>> {
        self.repo.get_by_atomic_number(z).map(ElementView::new)
    }

    pub fn by_symbol(&self, symbol: &str) -> Option<ElementView<'_>> {
        self.repo.get_by_symbol(symbol).map(ElementView::new)
    }

    pub fn by_name(&self, name: &str) -> Option<ElementView<'_>> {
        self.repo.get_by_name(name).map(ElementView::new)
    }

    /// Nearest element whose stored atomic mass is within `tolerance` of `mass`.
    pub fn by_atomic_mass(&self, mass: f64, tolerance: f64) -> Option<ElementView<'_>> {
        self.repo
            .iter()
            .filter(|e| (e.atomic_mass - mass).abs() <= tolerance)
            .min_by(|a, b| {
                (a.atomic_mass - mass)
                    .abs()
                    .total_cmp(&(b.atomic_mass - mass).abs())
            })
            .map(ElementView::new)
    }

    pub fn all(&self) -> impl Iterator<Item = ElementView<'_>> {
        self.repo.iter().map(ElementView::new)
    }
}
