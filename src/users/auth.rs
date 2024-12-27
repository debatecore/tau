use axum::http::{header::AUTHORIZATION, HeaderMap};
use sqlx::{Pool, Postgres};

use crate::omni_error::OmniError;

use super::User;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Non-ASCII characters found in AUTHORIZATION header.")]
    NonAsciiHeaderCharacters,
}

impl User {
    pub async fn authenticate(
        headers: &HeaderMap,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let cookie: Option<String> = None;
        let header = match headers.get(AUTHORIZATION) {
            Some(header) => match header.to_str() {
                Ok(header) => Some(header.to_string()),
                Err(_) => return Err(AuthError::NonAsciiHeaderCharacters)?,
            },
            None => None,
        };
        todo!()
    }
}
