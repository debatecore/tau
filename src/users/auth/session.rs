use super::AUTH_SESSION_LENGTH;
use crate::omni_error::OmniError;
use sqlx::{
    types::chrono::{DateTime, Utc},
    Pool, Postgres,
};
use uuid::Uuid;

pub struct Session {
    id: Uuid,
    user_id: Uuid,
    issued: DateTime<Utc>,
    expiry: DateTime<Utc>,
    last_access: Option<DateTime<Utc>>,
}

impl Session {
    pub async fn get_by_id(
        id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Session, OmniError> {
        match sqlx::query_as!(
            Session,
            "SELECT id, user_id, issued, expiry, last_access FROM sessions WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await
        {
            Ok(session) => Ok(session),
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
    /// Writes current timestamp to session's last_access field in db
    pub async fn update_last_access(
        self,
        pool: &Pool<Postgres>,
    ) -> Result<Session, OmniError> {
        let now = Some(Utc::now());
        match sqlx::query!(
            "UPDATE sessions SET last_access = $2 WHERE id = $1",
            self.id,
            now
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(Session {
                last_access: now,
                ..self
            }),
            Err(e) => Err(e)?,
        }
    }
    /// Prolongs expiry datetime of the session by a week
    pub async fn prolong(self, pool: &Pool<Postgres>) -> Result<Session, OmniError> {
        let expiry = Utc::now() + AUTH_SESSION_LENGTH;
        match sqlx::query!(
            "UPDATE sessions SET expiry = $2 WHERE id = $1",
            self.id,
            expiry
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(Session { expiry, ..self }),
            Err(e) => Err(e)?,
        }
    }
}
