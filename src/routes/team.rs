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

use super::utils::handle_failed_to_get_resource_by_id;

const DUPLICATE_NAME_ERROR: &str = r#"
    Team with this name already exists within the
    scope of the tournament, to which the team is assigned."#;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Team {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    /// Full name of the team (e.g. "Debate Team Buster").
    /// Must be unique within a scope of a tournament it's assigned to.
    full_name: String,
    shortened_name: String,
    tournament_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct TeamPatch {
    full_name: Option<String>,
    shortened_name: Option<String>,
}

impl Team {
    async fn post(team: Team, connection_pool: &Pool<Postgres>) -> Result<Team, Error> {
        match team_with_name_exists_in_tournament(
            &team.full_name,
            &team.tournament_id,
            connection_pool,
        )
        .await
        {
            Ok(exists) => {
                if exists {
                    // TO-DO: change the error to actually represent what's going on
                    // (team name already exists in this tournament)
                    return Err(sqlx::Error::PoolClosed);
                }
            }
            Err(e) => {
                return {
                    error!("Error creating a team: {e}");
                    Err(e)
                }
            }
        }
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

/// Create a new team
#[utoipa::path(post, request_body=Team, path = "/team",
    responses((
        status=200, description = "Team created successfully",
        body=Team,
        example=json!(get_team_example())
    ))
)]
async fn create_team(State(state): State<AppState>, Json(json): Json<Team>) -> Response {
    let pool = &state.connection_pool;
    match Tournament::get_by_id(json.tournament_id, &state.connection_pool).await {
        Ok(_) => (),
        Err(e) => return handle_failed_to_get_resource_by_id(e),
    };

    match Team::post(json, pool).await {
        Ok(team) => Json(team).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(get, path = "/team", 
    responses((
    status=200, description = "Ok",
    body=Vec<Motion>,
    example=json!(get_teams_list_example())
)))]
/// Get a list of all teams
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

/// Get details of an existing team
#[utoipa::path(get, path = "/team/{id}", 
    responses((status=200, description = "Ok", body=Team,
    example=json!(get_team_example())
    )),
    params(("id", description = "Team id"))
)]
async fn get_team_by_id(Path(id): Path<Uuid>, State(state): State<AppState>) -> Response {
    match Team::get_by_id(id, &state.connection_pool).await {
        Ok(team) => Json(team).into_response(),
        Err(e) => {
            error!("Error getting a team with id {id}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Patch an existing team
#[utoipa::path(patch, path = "/team/{id}", 
    request_body=Team,
    params(("id", description = "Team id")),
    responses(
        (
            status=200, description = "Team patched successfully",
            body=Team,
            example=json!(get_team_example())
        ),
        (
            status=409,
            description = DUPLICATE_NAME_ERROR,
        )
    )
)]
async fn patch_team_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(new_team): Json<TeamPatch>,
) -> Response {
    let pool = &state.connection_pool;

    let team_exists_result = Team::get_by_id(id, pool).await;
    if team_exists_result.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let team = team_exists_result.unwrap();

    let team_with_this_name_exists =
        team_with_name_exists_in_tournament(&team.full_name, &team.tournament_id, pool)
            .await;
    if team_with_this_name_exists.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    } else if team_with_this_name_exists
        .is_ok_and(|name_is_duplicate| name_is_duplicate == true)
    {
        return (StatusCode::CONFLICT, DUPLICATE_NAME_ERROR).into_response();
    }

    match team.patch(new_team, pool).await {
        Ok(team) => Json(team).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Delete an existing team
///
/// This operation is only allowed when there are no entities (i.e. attendees)
/// referencing this team.
#[utoipa::path(delete, path = "/team/{id}", 
    responses
    ((status=204, description = "Team deleted successfully")),
    params(("id", description = "Team id"))
)]
async fn delete_team_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    // TO-DO: disallow deletion of teams that are referenced by other entities
    // (most notably users)

    match Team::get_by_id(id, &state.connection_pool).await {
        Ok(team) => match team.delete(&state.connection_pool).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => {
                error!("Error deleting a team with id {id}: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        // TO-DO: Handle a scenario in which the team does not exist
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn team_with_name_exists_in_tournament(
    full_name: &String,
    tournament_id: &Uuid,
    connection_pool: &Pool<Postgres>,
) -> Result<bool, Error> {
    match query!(
        "SELECT EXISTS(SELECT 1 FROM teams WHERE full_name = $1 AND tournament_id = $2)",
        full_name,
        tournament_id
    )
    .fetch_one(connection_pool)
    .await
    {
        Ok(result) => Ok(result.exists.unwrap()),
        Err(e) => Err(e),
    }
}

fn get_team_example() -> String {
    r#"{
        "id": "01941267-2685-7a62-8382-c90fae07a87b",
        "full_name": "Debate Team Buster",
        "shortened_name": "DTB",
        "tournament_id": "01941267-0109-7405-b30e-7883d309c603"
    }"#
    .to_owned()
}

fn get_teams_list_example() -> String {
    r#"
    [
        {
            "id": "01941267-2685-7a62-8382-c90fae07a87b",
            "full_name": "Debate Team Buster",
            "shortened_name": "DTB",
            "tournament_id": "01941267-0109-7405-b30e-7883d309c603"
        },
        {
            "id": "01941266-dccb-75b0-82fb-2e885f9e3500",
            "full_name": "Delusional Debaters",
            "shortened_name": "DeDe",
            "tournament_id": "01941267-0109-7405-b30e-7883d309c603"
        }
    ]
    "#
    .to_owned()
}
