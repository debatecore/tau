use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

pub(crate) mod attendee;
pub(crate) mod debate;
pub(crate) mod motion;
pub(crate) mod team;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Tournament {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    // Full name of the tournament. Must be unique.
    full_name: String,
    shortened_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TournamentPatch {
    full_name: Option<String>,
    shortened_name: Option<String>,
}

impl Tournament {
    pub async fn post(
        tournament: Tournament,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(
            Tournament,
            r#"INSERT INTO tournaments(id, full_name, shortened_name)
        VALUES ($1, $2, $3) RETURNING id, full_name, shortened_name"#,
            tournament.id,
            tournament.full_name,
            tournament.shortened_name
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_all(
        connection_pool: &Pool<Postgres>,
    ) -> Result<Vec<Tournament>, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments")
            .fetch_all(connection_pool)
            .await
        {
            Ok(tournaments) => Ok(tournaments),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(tournament) => Ok(tournament),
            Err(e) => {
                error!("Error getting a tournament with id {id}: {e}");
                Err(e)?
            }
        }
    }

    pub async fn patch(
        self,
        patch: TournamentPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        let tournament = Tournament {
            id: self.id,
            full_name: patch.full_name.unwrap_or(self.full_name),
            shortened_name: patch.shortened_name.unwrap_or(self.shortened_name),
        };
        match query!(
            "UPDATE tournaments SET full_name = $1, shortened_name = $2 WHERE id = $3",
            tournament.full_name,
            tournament.shortened_name,
            tournament.id,
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM tournaments WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}
