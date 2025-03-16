use crate::{omni_error::OmniError, setup::AppState, tournament::{Tournament, TournamentPatch}, users::{permissions::Permission, TournamentUser, User}};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;


pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament", get(get_tournaments).post(create_tournament))
        .route(
            "/tournament/:id",
            get(get_tournament_by_id)
                .delete(delete_tournament_by_id)
                .patch(patch_tournament_by_id),
        )
}

/// Get a list of all tournaments
/// 
/// This request only returns the tournaments the user is permitted to see.
/// The user must be given any role within a tournament to see it.
#[utoipa::path(get, path = "/tournament", 
    responses(
        (
            status=200, description = "Ok",
            body=Vec<Tournament>,
            example=json!(get_tournaments_list_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to list any tournaments, meaning they do not have any roles within any tournament."
        ),
        (status=500, description = "Internal server error")
    ),
    tag="tournament"
)]
async fn get_tournaments(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let user = User::authenticate(&headers, cookies, pool).await?;

    let tournaments = Tournament::get_all(pool).await?;
    let mut visible_tournaments: Vec<Tournament> = vec![];
    for tournament in tournaments {
        let tournament_id = tournament.id;
        let roles = user.get_roles(tournament_id, pool).await?;
        let tournament_user = TournamentUser {
            user: user.clone(),
            roles
        };
        if tournament_user.has_permission(Permission::ReadTournament) {
            visible_tournaments.push(tournament);
        }
    }
    if visible_tournaments.is_empty() {
        return Ok(vec![].into_response());
    }
    Ok(Json(visible_tournaments).into_response())
}

/// Create a new tournament
/// 
/// Available only to the infrastructure admin.
#[utoipa::path(
    post,
    request_body=Tournament,
    path = "/tournament",
    responses
    (
        (
            status=200, 
            description = "Tournament created successfully",
            body=Tournament,
            example=json!(get_tournament_example_with_id())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify this tournament"
        ),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error")
    ),
    tag="tournament"
)]
async fn create_tournament(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(json): Json<Tournament>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let user = User::authenticate(&headers, cookies, &pool).await?;
    if !user.is_infrastructure_admin() {
        return Err(OmniError::InsufficientPermissionsError);
    }

    let tournament = Tournament::post(json, pool).await?;
    return Ok(Json(tournament).into_response());
}

/// Get details of an existing tournament
/// 
/// The user must be given any role within the tournament to use this endpoint.
#[utoipa::path(get, path = "/tournament/{id}", 
    responses
    (
        (
            status=200, description = "Ok", body=Tournament,
            example=json!
            (get_tournament_example_with_id())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read this tournament"
        ),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error")
    ),
    tag="tournament"
)]
async fn get_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::ReadTournament) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }
    match Tournament::get_by_id(id, pool).await {
        Ok(tournament) => Ok(Json(tournament).into_response()),
        Err(e) => Err(e),
    }
}

/// Patch an existing tournament
/// 
/// Requires either the Organizer or Admin role.
#[utoipa::path(patch, path = "/tournament/{id}", 
    request_body=TournamentPatch,
    responses(
        (
            status=200, description = "Tournament patched successfully",
            body=Tournament,
            example=json!(get_tournament_example_with_id())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify this tournament"
        ),
        (status=404, description = "Tournament not found"),
        (status=409, description = "A tournament with this name already exists"),
        (status=500, description = "Internal server error")
    ),
    tag="tournament"
)]
async fn patch_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(new_tournament): Json<TournamentPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteTournament) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let tournament = Tournament::get_by_id(id, pool).await?;
    match tournament.patch(new_tournament, pool).await {
        Ok(patched_tournament) => Ok(Json(patched_tournament).into_response()),
        Err(e) => {
            error!("Error patching a tournament with id {}: {e}", id);
            Err(e)
        }
    }
}


/// Delete an existing tournament.
/// 
/// Available only to the tournament Organizers.
/// This operation is only allowed when there are no resources
/// referencing this tournament.
#[utoipa::path(delete, path = "/tournament/{id}", 
    responses(
        (status=204, description = "Tournament deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (status=403, description = "The user is not permitted to modify this tournament"),
        (status=404, description = "Tournament not found"),
        (status=409, description = "Other resources reference this tournament. They must be deleted first")
    ),
    tag="tournament"
)]
async fn delete_tournament_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::WriteTournament) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let tournament = Tournament::get_by_id(id, pool).await?;
    match tournament.delete(pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) =>
        {
            if e.is_sqlx_foreign_key_violation() {
                return Err(OmniError::DependentResourcesError)
            }
            else {
                error!("Error deleting a tournament with id {id}: {e}");
                return Err(e)?;
            }
        },
    }
}

fn get_tournament_example_with_id() -> String {
    r#"
    {
        "id": "01941265-8b3c-733f-a6ae-075c079f2f81",
        "full_name": "Kórnik Debate League",
        "shortened_name": "KDL"
    }
    "#
    .to_owned()
}

fn get_tournaments_list_example() -> String {
    r#"
        [
        {
        "id": "01941265-8b3c-733f-a6ae-075c079f2f81",
        "full_name": "Kórnik Debate League",
        "shortened_name": "KDL"
        },
        {
        "id": "01941265-507e-7987-b1ed-5c0f63ff6c6d",
        "full_name": "Poznań Debate Night",
        "shortened_name": "PND"
        }
        ]
    "#.to_owned()
}
