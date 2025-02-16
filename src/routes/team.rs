use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{omni_error::OmniError, setup::AppState, users::{permissions::Permission, TournamentUser}};

use super::tournament::Tournament;

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
    async fn post(
        team: Team,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Team, OmniError> {
        match team_with_name_exists_in_tournament(
            &team.full_name,
            &team.tournament_id,
            connection_pool,
        )
        .await
        {
            Ok(exists) => {
                if exists {
                    return Err(OmniError::ResourceAlreadyExistsError);
                }
            }
            Err(e) => return Err(e)?,
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
            Err(e) => Err(e)?,
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
        .route("/:tournament_id/team", get(get_teams).post(create_team))
        .route(
            "/:tournament_id/team/:id",
            get(get_team_by_id)
                .patch(patch_team_by_id)
                .delete(delete_team_by_id),
        )
}

/// Create a new team
/// 
/// Available only to Organizers and Admins
#[utoipa::path(post, request_body=Team, path = "/{tournament_id}/team",
    responses
    (
        (
            status=200, description = "Team created successfully",
            body=Team,
            example=json!(get_team_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify teams within this tournament"
        ),
        (status=404, description = "Tournament or team not found"),
        (status=500, description = "Internal server error"),
    )
)]
async fn create_team(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(json): Json<Team>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteTeams) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match Team::post(json, pool).await {
        Ok(team) => Ok(Json(team).into_response()),
        Err(e) => Err(e),
    }
}

#[utoipa::path(get, path = "/{tournament_id}/team", 
    responses
    (
        (
            status=200, description = "Ok",
            body=Vec<Motion>,
            example=json!(get_teams_list_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to read teams within this tournament"
        ),
        (status=404, description = "Tournament or team not found"),
        (status=500, description = "Internal server error"),
    )
)]
/// Get a list of all teams
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_teams(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadTeams) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match query_as!(Team, "SELECT * FROM teams WHERE tournament_id = $1", tournament_id)
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(teams) => Ok(Json(teams).into_response()),
        Err(e) => {
            error!("Error getting a list of teams: {e}");
            Err(e)?
        }
    }
}

/// Get details of an existing team
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "{tournament_id}/team/{id}", 
    responses(
        (
            status=200, description = "Ok", body=Team,
            example=json!(get_team_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to read teams within this tournament"
        ),
        (status=404, description = "Tournament or team not found"),
        (status=500, description = "Internal server error"),
    ),
)]
async fn get_team_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Path(id): Path<Uuid>
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadTeams) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Team::get_by_id(id, pool).await {
        Ok(team) => Ok(Json(team).into_response()),
        Err(e) => {
            error!("Error getting a team with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Patch an existing team
/// 
/// Available only to Organizers and Admins.
#[utoipa::path(patch, path = "/team/{id}", 
    request_body=Team,
    responses(
        (
            status=200, description = "Team patched successfully",
            body=Team,
            example=json!(get_team_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify teams within this tournament"
        ),
        (status=404, description = "Tournament or team not found"),
        (
            status=409,
            description = DUPLICATE_NAME_ERROR,
        ),
        (status=500, description = "Internal server error"),
    )
)]
async fn patch_team_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(new_team): Json<TeamPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteTeams) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let team = Team::get_by_id(id, pool).await?;
    if team_with_name_exists_in_tournament(&team.full_name, &team.tournament_id, pool).await? {
        return Err(OmniError::ResourceAlreadyExistsError)
    }

    match team.patch(new_team, pool).await {
        Ok(team) => Ok(Json(team).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing team
///
/// This operation is only allowed when there are no entities
/// referencing this team. Available only to Organizers and Admins.
#[utoipa::path(delete, path = "/team/{id}", 
    responses
    (
        (status=204, description = "Team deleted successfully"),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify teams within this tournament"
        ),
        (status=404, description = "Tournament or team not found"),
    ),
)]
async fn delete_team_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteTeams) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let team = Team::get_by_id(id, pool).await?;
    match team.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a team with id {id}: {e}");
            Err(e)?
        }
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
