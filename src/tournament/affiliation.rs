use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    users::{roles::Role, User},
};

use super::Tournament;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
/// Some Judges might be affiliated with certain teams,
/// which poses a risk of biased rulings.
/// Tournament Organizers can denote such affiliations.
/// A Judge is prevented from ruling debates wherein
/// one of the sides is a team they're affiliated with.
pub struct Affiliation {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub team_id: Uuid,
    pub judge_user_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct AffiliationPatch {
    pub tournament_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub judge_user_id: Option<Uuid>,
}

impl Affiliation {
    pub async fn post(
        affiliation: Affiliation,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Affiliation, OmniError> {
        match query_as!(
            Affiliation,
            r#"INSERT INTO judge_team_assignments(id, judge_user_id, team_id, tournament_id)
            VALUES ($1, $2, $3, $4) RETURNING id, judge_user_id, team_id, tournament_id"#,
            affiliation.id,
            affiliation.judge_user_id,
            affiliation.team_id,
            affiliation.tournament_id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(affiliation),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Affiliation, OmniError> {
        match query_as!(
            Affiliation,
            "SELECT * FROM judge_team_assignments WHERE id = $1",
            id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(affiliation) => Ok(affiliation),
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        patch: Affiliation,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Affiliation, OmniError> {
        match query!(
            "UPDATE judge_team_assignments SET judge_user_id = $1, tournament_id = $2, team_id = $3 WHERE id = $4",
            patch.judge_user_id,
            patch.tournament_id,
            patch.team_id,
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

    pub async fn validate(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        let user = User::get_by_id(self.judge_user_id, pool).await?;
        if !user.has_role(Role::Judge, self.tournament_id, pool).await? {
            return Err(OmniError::NotAJudgeError);
        }

        let _tournament = Tournament::get_by_id(self.tournament_id, pool).await?;

        if self.already_exists(pool).await? {
            return Err(OmniError::ResourceAlreadyExistsError);
        }

        Ok(())
    }

    async fn already_exists(&self, pool: &Pool<Postgres>) -> Result<bool, OmniError> {
        match query_as!(Affiliation,
            "SELECT * FROM judge_team_assignments WHERE judge_user_id = $1 AND tournament_id = $2 AND team_id = $3",
            self.judge_user_id,
            self.tournament_id,
            self.team_id
        ).fetch_optional(pool).await {
            Ok(result) => {
                if result.is_none() {
                    return Ok(false);
                }
                else {
                    return Ok(true);
                }
            },
            Err(e) => Err(e)?,
        }
    }
}
