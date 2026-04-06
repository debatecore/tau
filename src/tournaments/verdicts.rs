use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{omni_error::OmniError, tournaments::debates::Debate, users::User};

use super::{roles::Role, Tournament};

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
/// Verdict denotes a winner of a debate (i.e. Proposition or Opposition).
/// Every Judge can make a verdict on a debate
/// within a tournament they're assigned to.
/// A debate can have multiple verdicts.
pub struct Verdict {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub debate_id: Uuid,
    pub judge_user_id: Uuid,
    pub proposition_won: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct VerdictPatch {
    pub debate_id: Option<Uuid>,
    pub judge_user_id: Option<Uuid>,
    pub proposition_won: Option<bool>,
}

impl Verdict {
    pub async fn post(
        verdict: Verdict,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Verdict, OmniError> {
        match query_as!(
            Verdict,
            r#"INSERT INTO verdicts(id, judge_user_id, debate_id, proposition_won)
            VALUES ($1, $2, $3, $4) RETURNING id, judge_user_id, debate_id, proposition_won"#,
            verdict.id,
            verdict.judge_user_id,
            verdict.debate_id,
            verdict.proposition_won
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(verdict),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Verdict, OmniError> {
        match query_as!(Verdict, "SELECT * FROM verdicts WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(verdict) => Ok(verdict),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Err(OmniError::ResourceNotFoundError),
                _ => Err(OmniError::InternalServerError),
            },
        }
    }

    pub async fn patch(
        self,
        patch: Verdict,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Verdict, OmniError> {
        match query!(
            "UPDATE verdicts SET judge_user_id = $1, debate_id = $2, proposition_won = $3 WHERE id = $4",
            patch.judge_user_id,
            patch.debate_id,
            patch.proposition_won,
            self.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(patch),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM judge_team_assignments WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }

    pub async fn validate(
        &self,
        tournament_id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<(), OmniError> {
        let user = User::get_by_id(self.judge_user_id, pool).await?;
        if !user.has_role(Role::Judge, tournament_id, pool).await? {
            return Err(OmniError::ResourceNotFoundError);
        }

        match Tournament::get_by_id(tournament_id, pool).await {
            Ok(_) => (),
            Err(e) => match e {
                OmniError::SqlxError(sqlx::Error::RowNotFound) => {
                    return Err(OmniError::ResourceNotFoundError)
                }
                _ => return Err(OmniError::InternalServerError),
            },
        }

        if self.already_exists(pool).await? {
            return Err(OmniError::ResourceAlreadyExistsError);
        }

        Ok(())
    }

    pub async fn infer_tournament_id(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Uuid, OmniError> {
        let debate = Debate::get_by_id(self.debate_id, pool).await?;
        Ok(debate.tournament_id)
    }

    async fn already_exists(&self, pool: &Pool<Postgres>) -> Result<bool, OmniError> {
        match query_as!(
            Verdict,
            "SELECT * FROM verdicts WHERE judge_user_id = $1 AND debate_id = $2 AND proposition_won = $3",
            self.judge_user_id,
            self.debate_id,
            self.proposition_won
        )
        .fetch_optional(pool)
        .await
        {
            Ok(result) => {
                if result.is_none() {
                    return Ok(false);
                } else {
                    return Ok(true);
                }
            }
            Err(e) => Err(e)?,
        }
    }
}
