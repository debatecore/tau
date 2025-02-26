use super::{crypto::hash_token, error::AuthError, AUTH_SESSION_LENGTH};
use crate::{omni_error::OmniError, users::auth::crypto::generate_token};
use serde::Serialize;
use sqlx::{
    types::chrono::{DateTime, Utc},
    Pool, Postgres,
};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Session {
    id: Uuid,
    user_id: Uuid,
    issued: DateTime<Utc>,
    expiry: DateTime<Utc>,
    last_access: Option<DateTime<Utc>>,
}

impl Session {
    pub async fn get_by_id(
        id: &Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Session, OmniError> {
        match sqlx::query_as!(
            Session,
            "SELECT id, user_id, issued, expiry, last_access FROM sessions WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
        {
            Ok(session) => match session {
                Some(s) => Ok(s),
                None => Err(AuthError::SessionExpired)?,
            },
            Err(e) => Err(e)?,
        }
    }
    pub async fn get_by_token(
        token: &str,
        pool: &Pool<Postgres>,
    ) -> Result<Session, OmniError> {
        let hashed_token = hash_token(token);
        match sqlx::query_as!(
            Session,
            "SELECT id, user_id, issued, expiry, last_access FROM sessions WHERE token = $1",
            hashed_token
        ).fetch_optional(pool).await {
            Ok(session) => match session {
                Some(s) => Ok(s),
                None => Err(AuthError::SessionExpired)?,
            },
            Err(e) => Err(e)?
        }
    }
    pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<Session>, OmniError> {
        match sqlx::query_as!(
            Session,
            "SELECT id, user_id, issued, expiry, last_access FROM sessions"
        )
        .fetch_all(pool)
        .await
        {
            Ok(sessions) => Ok(sessions),
            Err(e) => Err(e)?,
        }
    }
    pub async fn create(
        user_id: &Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<(Session, String), OmniError> {
        let id = Uuid::now_v7();
        let token = generate_token();
        let hashed_token = hash_token(&token);
        match sqlx::query_as!(
            Session,
            r#"
            INSERT INTO sessions(id, token, user_id) VALUES ($1, $2, $3)
            RETURNING id, user_id, issued, expiry, last_access
        "#,
            &id,
            &hashed_token,
            user_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(session) => Ok((session, token)),
            Err(e) => Err(e)?,
        }
    }
    pub async fn destroy(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match sqlx::query!("DELETE FROM sessions WHERE id = $1", self.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
    /// Prolongs session expiry by a week and updates last_access at once
    pub async fn prolong_and_update_last_access(
        self,
        pool: &Pool<Postgres>,
    ) -> Result<Session, OmniError> {
        let now = Some(Utc::now());
        let expiry = Utc::now() + AUTH_SESSION_LENGTH;
        match sqlx::query!(
            "UPDATE sessions SET expiry = $2, last_access = $3 WHERE id = $1",
            self.id,
            expiry,
            now
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(Session {
                expiry,
                last_access: now,
                ..self
            }),
            Err(e) => Err(e)?,
        }
    }
}
