#[derive(thiserror::Error, Debug)]
pub enum OmniError {
    #[error("AuthError: {0}")]
    AuthError(#[from] crate::users::auth::AuthError),
    #[error("PhotoUrlError: {0}")]
    PhotoUrlError(#[from] crate::users::photourl::PhotoUrlError),
    #[error("sqlx::Error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("serde_json::Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}
