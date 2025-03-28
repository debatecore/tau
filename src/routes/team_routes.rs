use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sqlx::{query, query_as, Error, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::{omni_error::OmniError, setup::AppState, tournament::{team::{Team, TeamPatch}, Tournament}, users::{permissions::Permission, TournamentUser}};

const DUPLICATE_NAME_ERROR: &str = r#"
    Team with this name already exists within the
    scope of the tournament, to which the team is assigned."#;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament/:tournament_id/team", get(get_teams).post(create_team))
        .route(
            "/tournament/:tournament_id/team/:id",
            get(get_team_by_id)
                .patch(patch_team_by_id)
                .delete(delete_team_by_id),
        )
}

/// Create a new team
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(post, request_body=Team, path = "/tournament/{tournament_id}/team",
    responses
    (
        (
            status=200, description = "Team created successfully",
            body=Team,
            example=json!(get_team_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
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
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    if team_with_name_exists_in_tournament(&json.full_name, &tournament_id, pool).await? {
        return Err(OmniError::ResourceAlreadyExistsError);
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match Team::post(json, pool).await {
        Ok(team) => Ok(Json(team).into_response()),
        Err(e) => {
            error!("Error creating a new team: {e}");
            Err(e)
        },
    }
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/team", 
    responses
    (
        (
            status=200, description = "Ok",
            body=Vec<Team>,
            example=json!(get_teams_list_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
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
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match tournament.get_debates(pool).await
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
#[utoipa::path(get, path = "/tournament/{tournament_id}/team/{id}", 
    responses(
        (
            status=200, description = "Ok", body=Team,
            example=json!(get_team_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
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
        false => return Err(OmniError::InsufficientPermissionsError),
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
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "/tournament/{tournament_id}/team/{id}", 
    request_body=Team,
    responses(
        (
            status=200, description = "Team patched successfully",
            body=Team,
            example=json!(get_team_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
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
        false => return Err(OmniError::InsufficientPermissionsError),
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
/// referencing this team. Available only to the tournament Organizers.
#[utoipa::path(delete, path = "/team/{id}", 
    responses
    (
        (status=204, description = "Team deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
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
        false => return Err(OmniError::InsufficientPermissionsError),
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
