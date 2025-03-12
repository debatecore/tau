use chrono::{DateTime, Local, Utc};
use uuid::Uuid;

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
