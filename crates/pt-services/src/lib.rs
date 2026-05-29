//! Query API exposing stored and computed element properties.

mod error;
mod table;
mod view;

pub use error::ServiceError;
pub use table::PeriodicTable;
pub use view::ElementView;
