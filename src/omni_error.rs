use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

const RESOURCE_ALREADY_EXISTS_MESSAGE: &str = "Resource already exists";
const RESOURCE_NOT_FOUND_MESSAGE: &str = "Resource not found";

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
    #[error("{RESOURCE_ALREADY_EXISTS_MESSAGE}")]
    ResourceAlreadyExistsError,
    #[error("{RESOURCE_NOT_FOUND_MESSAGE}")]
    ResourceNotFoundError,
}

impl IntoResponse for OmniError {
    fn into_response(self) -> Response {
        self.respond()
    }
}

impl OmniError {
    pub fn is_sqlx_unique_violation(&self) -> bool {
        match self {
            OmniError::SqlxError(e) => match e {
                sqlx::Error::Database(e) => {
                    if e.is_unique_violation() {
                        return true;
                    }
                }
                _ => (),
            },
            _ => (),
        };
        return false;
    }

    pub fn respond(self) -> Response {
        use OmniError as E;
        const ISE: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;
        match self {
            E::ExplicitError {
                status: s,
                message: m,
            } => (s, m).into_response(),
            E::AuthError(e) => (e.status_code(), e.to_string()).into_response(),
            E::SqlxError(e) => match e {
                sqlx::Error::RowNotFound => {
                    return (StatusCode::BAD_REQUEST, RESOURCE_NOT_FOUND_MESSAGE)
                        .into_response()
                }
                sqlx::Error::Database(e) => {
                    if e.is_unique_violation() {
                        return (StatusCode::CONFLICT, RESOURCE_ALREADY_EXISTS_MESSAGE)
                            .into_response();
                    } else if e.is_foreign_key_violation() {
                        return (
                            StatusCode::BAD_REQUEST,
                            "Referring to a nonexistent resource",
                        )
                            .into_response();
                    } else {
                        (ISE, "SQLx Error").into_response()
                    }
                }
                _ => (ISE, "SQLx Error").into_response(),
            },

            E::PhotoUrlError(_)
            | E::SerdeJsonError(_)
            | E::Base64DecodeError(_)
            | E::FromUtf8Error(_)
            | E::PassHashError(_) => (ISE, self.clerr()).into_response(),
            E::ResourceAlreadyExistsError => {
                (StatusCode::CONFLICT, self.clerr()).into_response()
            }
            E::ResourceNotFoundError => {
                (StatusCode::BAD_REQUEST, self.clerr()).into_response()
            }
        }
    }

    /// clerr shall henceforth stand for client facing error message
    fn clerr(&self) -> String {
        use OmniError as E;
        match self {
            E::ExplicitError { .. } | E::AuthError(_) => unreachable!(),
            E::PhotoUrlError(_) => "PhotoUrl parsing failure.",
            E::SqlxError(_) => "SQL/SQLx failure.",
            E::SerdeJsonError(_) => "SerdeJSON failure.",
            E::Base64DecodeError(_) => "Base64 decoding failure.",
            E::FromUtf8Error(_) => "UTF8 decoding failure.",
            E::PassHashError(_) => "Password hash failure.",
            E::ResourceAlreadyExistsError => RESOURCE_ALREADY_EXISTS_MESSAGE,
            E::ResourceNotFoundError => RESOURCE_NOT_FOUND_MESSAGE,
        }
        .to_string()
    }
}

impl From<argon2::password_hash::Error> for OmniError {
    fn from(e: argon2::password_hash::Error) -> Self {
        OmniError::PassHashError(e.to_string())
    }
}
