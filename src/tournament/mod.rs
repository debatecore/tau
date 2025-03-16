use debate::Debate;
use location::Location;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use team::Team;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

pub(crate) mod attendee;
pub(crate) mod debate;
pub(crate) mod location;
pub(crate) mod motion;
pub(crate) mod room;
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
        pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(
            Tournament,
            r#"INSERT INTO tournaments(id, full_name, shortened_name)
        VALUES ($1, $2, $3) RETURNING id, full_name, shortened_name"#,
            tournament.id,
            tournament.full_name,
            tournament.shortened_name
        )
        .fetch_one(pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<Tournament>, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments")
            .fetch_all(pool)
            .await
        {
            Ok(tournaments) => Ok(tournaments),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments WHERE id = $1", id)
            .fetch_one(pool)
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
        pool: &Pool<Postgres>,
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
        .execute(pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM tournaments WHERE id = $1", self.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_debates(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Debate>, OmniError> {
        match query_as!(
            Debate,
            "SELECT * FROM debates WHERE tournament_id = $1",
            &self.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(debates) => Ok(debates),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_teams(&self, pool: &Pool<Postgres>) -> Result<Vec<Team>, OmniError> {
        match query_as!(
            Team,
            "SELECT * FROM teams WHERE tournament_id = $1",
            &self.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(debates) => Ok(debates),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_locations(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Location>, OmniError> {
        match query_as!(
            Location,
            "SELECT * FROM locations WHERE tournament_id = $1",
            self.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(locations) => Ok(locations),
            Err(e) => Err(e)?,
        }
    }
}
