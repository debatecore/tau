use super::{
    cookie::set_session_token_cookie, crypto::hash_token, error::AuthError,
    session::Session, AUTH_SESSION_COOKIE_NAME,
};
use crate::{
    omni_error::OmniError,
    users::{auth::login_tokens::LoginToken, User},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::{header::AUTHORIZATION, HeaderMap};
use base64::{prelude::BASE64_STANDARD, Engine};
use sqlx::{types::chrono::Utc, Pool, Postgres};
use tower_cookies::Cookies;

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
                    "Bearer" => User::auth_via_session(data, cookies, pool).await,
                    _ => Err(AuthError::UnsupportedHeaderAuthScheme)?,
                }
            }
            (Some(cookie), None) => User::auth_via_session(&cookie, cookies, pool).await,
        }
    }
    async fn auth_via_b64_credentials(
        data: &str,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let (usr, pwd) =
            match String::from_utf8(BASE64_STANDARD.decode(data)?)?.split_once(":") {
                Some((usr, pwd)) => (usr.to_string(), pwd.to_string()),
                None => return Err(AuthError::NoBasicAuthColonSplit)?,
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
    pub async fn auth_via_session(
        token: &str,
        cookies: Cookies,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let hashed_token = hash_token(token);
        match sqlx::query!(
            "SELECT id, user_id, expiry FROM sessions WHERE token = $1",
            &hashed_token
        )
        .fetch_one(pool)
        .await
        {
            Ok(session) => {
                if session.expiry < Utc::now() {
                    Err(AuthError::SessionExpired)?
                }

                let user = User::get_by_id(session.user_id, pool).await?;
                Session::get_by_id(&session.id, pool)
                    .await?
                    .prolong_and_update_last_access(pool)
                    .await?;
                set_session_token_cookie(token, cookies);
                Ok(user)
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => Err(AuthError::InvalidCredentials)?,
                _ => Err(OmniError::SqlxError(e))?,
            },
        }
    }

    pub async fn auth_via_link(
        token: &str,
        pool: &Pool<Postgres>,
    ) -> Result<User, OmniError> {
        let hashed_token = hash_token(token);
        let token_record = sqlx::query_as!(
            LoginToken,
            "SELECT * FROM login_tokens WHERE token_hash = $1",
            hashed_token
        )
        .fetch_optional(pool)
        .await?;
        if token_record.is_none() {
            return Err(AuthError::InvalidToken)?;
        }
        let token = token_record.unwrap();
        if token.expired() {
            return Err(AuthError::TokenExpired)?;
        } else if token.used {
            return Err(AuthError::TokenAlreadyUsed)?;
        }
        token.mark_as_used(pool).await?;
        match User::get_by_id(token.user_id, pool).await {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }
}
