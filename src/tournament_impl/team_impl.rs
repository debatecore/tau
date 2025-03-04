use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Team {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    /// Full name of the team (e.g. "Debate Team Buster").
    /// Must be unique within a scope of a tournament it's assigned to.
    pub full_name: String,
    pub shortened_name: String,
    pub tournament_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct TeamPatch {
    full_name: Option<String>,
    shortened_name: Option<String>,
}

impl Team {
    pub async fn post(
        team: Team,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Team, OmniError> {
        match query_as!(
            Team,
            r#"INSERT INTO teams(id, full_name, shortened_name, tournament_id)
            VALUES ($1, $2, $3, $4) RETURNING id, full_name, shortened_name, tournament_id"#,
            team.id,
            team.full_name,
            team.shortened_name,
            team.tournament_id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(team),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Team, OmniError> {
        match query_as!(Team, "SELECT * FROM teams WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(team) => Ok(team),
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        new_team: TeamPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Team, OmniError> {
        let patch = Team {
            id: self.id,
            full_name: new_team.full_name.unwrap_or(self.full_name),
            shortened_name: new_team.shortened_name.unwrap_or(self.shortened_name),
            tournament_id: self.tournament_id,
        };
        match query!(
            "UPDATE teams set full_name = $1, shortened_name = $2 WHERE id = $3",
            patch.full_name,
            patch.shortened_name,
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
        match query!("DELETE FROM teams WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}
