#[derive(thiserror::Error, Debug)]
pub enum OmniError {
    #[error("sqlx::Error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("users::photourl::PhotoUrlError: {0}")]
    PhotoUrlError(#[from] crate::users::photourl::PhotoUrlError),
    #[error("serde_json::Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}
