use axum::http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("No authentication credentials provided.")]
    NoCredentials,
    #[error("Session expired.")]
    SessionExpired,

    #[error("Non-ASCII characters found in AUTHORIZATION header.")]
    NonAsciiHeaderCharacters,
    #[error("B64 Basic Auth header is missing a colon inbetween login and passwd.")]
    NoBasicAuthColonSplit,
    #[error("Could not parse header auth scheme/data.")]
    BadHeaderAuthSchemeData,
    #[error("Unsupported header auth scheme - use Basic or Bearer.")]
    UnsupportedHeaderAuthScheme,
}

impl AuthError {
    pub fn status_code(&self) -> StatusCode {
        use AuthError::*;
        match self {
            InvalidCredentials | NoCredentials | SessionExpired => {
                StatusCode::UNAUTHORIZED
            }
            NonAsciiHeaderCharacters
            | NoBasicAuthColonSplit
            | BadHeaderAuthSchemeData
            | UnsupportedHeaderAuthScheme => StatusCode::BAD_REQUEST,
        }
    }
}
