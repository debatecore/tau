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
    #[error("base64::DecodeError: {0}")]
    Base64DecodeERROR(#[from] base64::DecodeError),
    #[error("std::string::FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    // this doesn't implement Error for some reason
    #[error("argon2::password_hash::Error: {0}")]
    ArgonPassHashError(String),
}

impl From<argon2::password_hash::Error> for OmniError {
    fn from(e: argon2::password_hash::Error) -> Self {
        OmniError::ArgonPassHashError(e.to_string())
    }
}
