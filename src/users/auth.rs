use super::User;
use crate::omni_error::OmniError;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use base64::{
    prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD},
    Engine,
};
use sqlx::{Pool, Postgres};
use tower_cookies::{cookie::time::Duration, Cookies};

pub const AUTH_SESSION_COOKIE_NAME: &str = "tausession";
const AUTH_SESSION_LENGTH: Duration = Duration::weeks(1);

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("No authentication credentials provided.")]
    NoCredentials,
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
            InvalidCredentials | NoCredentials => StatusCode::UNAUTHORIZED,
            NonAsciiHeaderCharacters
            | NoBasicAuthColonSplit
            | BadHeaderAuthSchemeData
            | UnsupportedHeaderAuthScheme => StatusCode::BAD_REQUEST,
        }
    }
}

impl User {
    pub async fn authenticate(
        headers: &HeaderMap,
        cookies: Cookies,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let cookie = match cookies.get(AUTH_SESSION_COOKIE_NAME) {
            Some(cookie) => match cookie.value().is_empty() {
                true => None,
                false => Some(cookie.value().to_string()),
            },
            None => None,
        };
        let header = match headers.get(AUTHORIZATION) {
            Some(header) => match header.to_str() {
                Ok(header) => Some(header.to_string()),
                Err(_) => return Err(AuthError::NonAsciiHeaderCharacters)?,
            },
            None => None,
        };

        match (cookie, header) {
            (None, None) => Err(AuthError::NoCredentials)?,
            (_, Some(header)) => {
                let (scheme, data) = match header.split_once(' ') {
                    Some((a, b)) => (a, b),
                    None => return Err(AuthError::BadHeaderAuthSchemeData)?,
                };
                match scheme {
                    "Basic" => User::auth_via_b64_credentials(data, pool).await,
                    "Bearer" => todo!(),
                    _ => Err(AuthError::UnsupportedHeaderAuthScheme)?,
                }
            }
            (Some(cookie), None) => todo!(),
        }
    }
    async fn auth_via_b64_credentials(
        data: &str,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let (usr, pwd) =
            match String::from_utf8(BASE64_STANDARD.decode(data)?)?.split_once(":") {
                Some((usr, pwd)) => (usr.to_string(), pwd.to_string()),
                None => return Err(AuthError::InvalidCredentials)?,
            };
        User::auth_via_credentials(usr.as_str(), pwd.as_str(), pool).await
    }
    pub async fn auth_via_credentials(
        login: &str,
        password: &str,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let hash = match sqlx::query!(
            "SELECT password_hash FROM users WHERE handle = $1",
            login
        )
        .fetch_one(pool)
        .await
        {
            Ok(hash) => hash.password_hash,
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Err(AuthError::InvalidCredentials)?,
                _ => return Err(OmniError::SqlxError(e))?,
            },
        };
        let argon = Argon2::default();
        let hash = match PasswordHash::new(&hash) {
            Ok(hash) => hash,
            Err(e) => return Err(e)?,
        };

        match argon.verify_password(password.as_bytes(), &hash).is_ok() {
            true => User::get_by_handle(login, pool).await,
            false => Err(AuthError::InvalidCredentials)?,
        }
    }
}
