use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Debate {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub motion_id: Option<Uuid>,
    pub marshall_user_id: Option<Uuid>,
    pub tournament_id: Uuid,
    pub round_id: Uuid,
}

#[serde_inline_default]
#[derive(Deserialize, ToSchema)]
pub struct DebatePatch {
    pub motion_id: Option<Uuid>,
    pub marshall_user_id: Option<Uuid>,
    pub tournament_id: Option<Uuid>,
    pub round_id: Option<Uuid>,
}

impl Debate {
    pub async fn post(
        debate: Debate,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Debate, OmniError> {
        match query_as!(
            Debate,
            r#"INSERT INTO debates(id, motion_id, marshall_user_id, tournament_id, round_id)
            VALUES ($1, $2, $3, $4, $5) RETURNING id, motion_id, marshall_user_id, tournament_id, round_id"#,
            debate.id,
            debate.motion_id,
            debate.marshall_user_id,
            debate.tournament_id,
            debate.round_id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(debate),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Debate, OmniError> {
        match query_as!(Debate, "SELECT * FROM debates WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(debate) => Ok(debate),
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        patch: DebatePatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Debate, OmniError> {
        let debate = Debate {
            id: self.id,
            motion_id: patch.motion_id,
            marshall_user_id: patch.marshall_user_id,
            tournament_id: patch.tournament_id.unwrap_or(self.tournament_id),
            round_id: patch.round_id.unwrap_or(self.round_id),
        };
        match query!(
            "UPDATE debates SET motion_id = $1, marshall_user_id = $2, round_id = $3 WHERE id = $4",
            debate.motion_id,
            debate.marshall_user_id,
            debate.round_id,
            debate.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(debate),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM debates WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}
