use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error, Pool, Postgres};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::setup::AppState;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Team {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    full_name: String,
    shortened_name: String,
    tournament_id: Uuid,
}

#[derive(Deserialize)]
pub struct TeamPatch {
    full_name: Option<String>,
    shortened_name: Option<String>,
}

impl Team {
    async fn post(team: Team, connection_pool: &Pool<Postgres>) -> Result<Team, Error> {
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
            Err(e) => Err(e),
        }
    }

    async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Team, Error> {
        match query_as!(Team, "SELECT * FROM teams WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(team) => Ok(team),
            Err(e) => Err(e),
        }
    }

    async fn patch(
        self,
        new_team: TeamPatch,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Team, Error> {
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
            Err(e) => Err(e),
        }
    }

    async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM teams WHERE id = $1", self.id)
            .execute(connection_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/team", get(get_teams).post(create_team))
        .route(
            "/team/:id",
            get(get_team_by_id)
                .patch(patch_team_by_id)
                .delete(delete_team_by_id),
        )
}

async fn create_team(State(state): State<AppState>, Json(json): Json<Team>) -> Response {
    match Team::post(json, &state.connection_pool).await {
        Ok(team) => Json(team).into_response(),
        Err(e) => {
            error!("Error creating a tournament: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_teams(State(state): State<AppState>) -> Response {
    match query_as!(Team, "SELECT * FROM teams")
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(teams) => Json(teams).into_response(),
        Err(e) => {
            error!("Error getting a list of teams: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_team_by_id(Path(id): Path<Uuid>, State(state): State<AppState>) -> Response {
    match Team::get_by_id(id, &state.connection_pool).await {
        Ok(team) => Json(team).into_response(),
        Err(e) => {
            error!("Error getting a team with id {id}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn patch_team_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(new_team): Json<TeamPatch>,
) -> Response {
    match Team::get_by_id(id, &state.connection_pool).await {
        Ok(team) => match team.patch(new_team, &state.connection_pool).await {
            Ok(team) => Json(team).into_response(),
            Err(e) => {
                error!("Error patching a team with id {id}: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_team_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    match Team::get_by_id(id, &state.connection_pool).await {
        Ok(team) => match team.delete(&state.connection_pool).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => {
                error!("Error deleting a team with id {id}: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
