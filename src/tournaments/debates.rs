use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Pool, Postgres, Transaction};
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
    pub marshal_user_id: Option<Uuid>,
    pub tournament_id: Uuid,
    pub round_id: Uuid,
}

#[serde_inline_default]
#[derive(Deserialize, ToSchema)]
pub struct DebatePatch {
    pub motion_id: Option<Uuid>,
    pub marshal_user_id: Option<Uuid>,
    pub tournament_id: Option<Uuid>,
    pub round_id: Option<Uuid>,
}

impl Debate {
    pub async fn post(
        tournament_id: Uuid,
        json: Debate,
        pool: &Pool<Postgres>,
    ) -> Result<Debate, OmniError> {

        let mut transaction = pool.begin().await?;
        let debate = Self::post_with_transaction(&mut transaction, tournament_id, json).await?;
        transaction.commit().await?;
        Ok(debate)
    }

    pub async fn post_with_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        tournament_id: Uuid,
        json: Debate,
    ) -> Result<Debate, OmniError> {

        let debate = query_as!(
            Debate,
            r#"INSERT INTO debates(id, motion_id, marshal_user_id, tournament_id, round_id)
            VALUES ($1, $2, $3, $4, $5) RETURNING id, motion_id, marshal_user_id, tournament_id, round_id"#,
            json.id,
            json.motion_id,
            json.marshal_user_id,
            tournament_id,
            json.round_id
        )
        .fetch_one(&mut **transaction)
        .await?;

        Ok(debate)
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
        pool: &Pool<Postgres>,
    ) -> Result<Debate, OmniError> {

        let mut transaction = pool.begin().await?;
        let debate = self.patch_with_transaction(&mut transaction, patch).await?;
        transaction.commit().await?;
        Ok(debate)
    }

    pub async fn patch_with_transaction(
        self,
        transaction: &mut Transaction<'_, Postgres>,
        patch: DebatePatch,
    ) -> Result<Debate, OmniError> {

        let updated = query_as!(
            Debate,
            r#"UPDATE debates SET motion_id = $1, marshal_user_id = $2, round_id = $3 WHERE id = $4
            RETURNING id, motion_id, marshal_user_id, tournament_id, round_id"#,
            patch.motion_id,
            patch.marshal_user_id,
            patch.round_id,
            self.id
        )
        .fetch_one(&mut **transaction)
        .await?;

        Ok(updated)
    }

    pub async fn delete(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        let mut transaction = pool.begin().await?;
        self.delete_with_transaction(&mut transaction).await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn delete_with_transaction(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), OmniError> {
        query!(
            r#"DELETE FROM debates WHERE id = $1"#,
            self.id
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
