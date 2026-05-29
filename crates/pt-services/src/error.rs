//! Service-layer errors.

/// Error returned by the service API.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Data(#[from] pt_data::DataError),
}
