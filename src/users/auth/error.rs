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
    #[error("Can only clear session given in Bearer scheme.")]
    ClearSessionBearerOnly,
    #[error("Provided single-use login token has already been used.")]
    TokenAlreadyUsed,
    #[error("Provided single-use login token has expired.")]
    TokenExpired,
    #[error("Invalid token.")]
    InvalidToken,
}

impl AuthError {
    pub fn status_code(&self) -> StatusCode {
        use AuthError as E;
        match self {
            E::InvalidCredentials
            | E::NoCredentials
            | E::SessionExpired
            | E::TokenAlreadyUsed
            | E::TokenExpired
            | E::InvalidToken => StatusCode::UNAUTHORIZED,
            E::NonAsciiHeaderCharacters
            | E::NoBasicAuthColonSplit
            | E::BadHeaderAuthSchemeData
            | E::UnsupportedHeaderAuthScheme
            | E::ClearSessionBearerOnly => StatusCode::BAD_REQUEST,
        }
    }
}
