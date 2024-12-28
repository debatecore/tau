use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(thiserror::Error, Debug)]
pub enum OmniError {
    #[error("{message}")]
    ExplicitError { status: StatusCode, message: String },
    #[error("AuthError: {0}")]
    AuthError(#[from] crate::users::auth::error::AuthError),

    #[error("PhotoUrlError: {0}")]
    PhotoUrlError(#[from] crate::users::photourl::PhotoUrlError),
    #[error("sqlx::Error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("serde_json::Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("base64::DecodeError: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("std::string::FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    // this doesn't implement Error for some reason
    #[error("argon2::password_hash::Error: {0}")]
    PassHashError(String),
}

impl OmniError {
    pub fn respond(self) -> Response {
        use OmniError::*;
        const ISE: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;
        match self {
            ExplicitError {
                status: s,
                message: m,
            } => (s, m).into_response(),
            AuthError(e) => (e.status_code(), e.to_string()).into_response(),

            PhotoUrlError(_) | SqlxError(_) | SerdeJsonError(_)
            | Base64DecodeError(_) | FromUtf8Error(_) | PassHashError(_) => {
                (ISE, self.clerr()).into_response()
            }
        }
    }

    // clerr shall henceforth stand for client facing error message
    fn clerr(&self) -> String {
        use OmniError::*;
        match self {
            ExplicitError { .. } | AuthError(_) => unreachable!(),
            PhotoUrlError(_) => "PhotoUrl parsing failure.",
            SqlxError(_) => "SQL/SQLx failure.",
            SerdeJsonError(_) => "SerdeJSON failure.",
            Base64DecodeError(_) => "Base64 decoding failure.",
            FromUtf8Error(_) => "UTF8 decoding failure.",
            PassHashError(_) => "Password hash failure.",
        }
        .to_string()
    }
}

impl From<argon2::password_hash::Error> for OmniError {
    fn from(e: argon2::password_hash::Error) -> Self {
        OmniError::PassHashError(e.to_string())
    }
}
