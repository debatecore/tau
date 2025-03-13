use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Error, Pool, Postgres};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

use super::utils::get_optional_value_to_be_patched;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    /// The main motion content, e.g. "This House would abolish the UN Security Council."
    pub motion: String,
    /// Infoslide i.e. additional information. It may be required
    /// to understand a complex motion.
    pub adinfo: Option<String>,
}

#[serde_inline_default]
#[derive(Deserialize, ToSchema)]
pub struct MotionPatch {
    motion: Option<String>,
    #[serde_inline_default(None)]
    adinfo: Option<String>,
}

impl Motion {
    pub async fn post(
        motion: Motion,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Motion, OmniError> {
        match query_as!(
            Motion,
            r#"INSERT INTO motions(id, motion, adinfo)
        VALUES ($1, $2, $3) RETURNING id, motion, adinfo"#,
            motion.id,
            motion.motion,
            motion.adinfo
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(motion),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Motion, OmniError> {
        match query_as!(Motion, "SELECT * FROM motions WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(motion) => Ok(motion),
            Err(e) => match e {
                Error::RowNotFound => Err(OmniError::ResourceNotFoundError),
                _ => {
                    error!("Failed to get a motion with id {id}: {e}");
                    Err(e)?
                }
            },
        }
    }

    pub async fn patch(
        self,
        patch: MotionPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Motion, OmniError> {
        let motion = Motion {
            id: self.id,
            motion: patch.motion.unwrap_or(self.motion),
            adinfo: get_optional_value_to_be_patched(patch.adinfo, self.adinfo),
        };
        match query!(
            "UPDATE motions SET motion = $1, adinfo = $2 WHERE id = $3",
            motion.motion,
            motion.adinfo,
            motion.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(motion),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM motions WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error deleting a motion with id {}: {e}", self.id);
                Err(e)
            }
        }
    }

    pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<Motion>, OmniError> {
        match query_as!(Motion, "SELECT * FROM motions",)
            .fetch_all(pool)
            .await
        {
            Ok(motions) => Ok(motions),
            Err(e) => Err(e)?,
        }
    }
}
