use chrono::{DateTime, Utc};
use sqlx::{query, Pool, Postgres};
use tracing::error;
use uuid::Uuid;

use crate::omni_error::OmniError;

pub struct LoginToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expiry: DateTime<Utc>,
    pub used: bool,
}

impl LoginToken {
    pub fn expired(&self) -> bool {
        return &Utc::now() > &self.expiry;
    }
}

impl LoginToken {
    pub async fn mark_as_used(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("UPDATE login_tokens SET used = true WHERE id = $1", self.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error invalidating token {}: {e}", self.id);
                Err(e)?
            }
        }
    }
}
