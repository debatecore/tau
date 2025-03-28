use crate::{omni_error::OmniError, setup::AppState, tournament::{debate::{Debate, DebatePatch}, Tournament}, users::{permissions::Permission, TournamentUser}};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sqlx::query_as;
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;


pub fn route() -> Router<AppState> {
    Router::new()
        .route("/:tournament_id/debate", get(get_debates).post(create_debate))
        .route(
            "/:tournament_id/debate/:id",
            get(get_debate_by_id)
                .delete(delete_debate_by_id)
                .patch(patch_debate_by_id),
        )
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/debate", 
    responses(
        (
            status=200, description = "Ok",
            body=Vec<Debate>,
        ),
        (status=400, description = "Bad request"),
        (
            status=401,
            description = "The user is not permitted to read debates within this tournament",
        ),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error"),
    )
)]
/// Get a list of all debates
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_debates(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadDebates) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Tournament::get_by_id(tournament_id, pool).await?.get_debates(pool).await
    {
        Ok(debates) => Ok(Json(debates).into_response()),
        Err(e) => {
            error!("Error getting a list of debates: {e}");
            Err(e)?
        }
    }
}

/// Create a new debate
/// 
/// Available only to Organizers and Admins.
#[utoipa::path(post, request_body=Debate, path = "/tournament/{tournament_id}/debate",
    responses(
        (
            status=200,
            description = "Debate created successfully",
            body=Debate,
        ),
        (status=400, description = "Bad request"),
        (
            status=401,
            description = "The user is not permitted to modify debates within this tournament",
        ),
        (status=404, description = "Tournament or attendee not found"),
        (status=500, description = "Internal server error"),
    )
)]
async fn create_debate(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(json): Json<Debate>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteDebates) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Debate::post(json, &state.connection_pool).await {
        Ok(debate) => Ok(Json(debate).into_response()),
        Err(e) => {
            error!("Error creating a new debate: {e}");
            Err(e)?
        }
    }
}

/// Get details of an existing debate
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/tournament/{tournament_id}/debate/{id}", 
    responses(
        (
            status=200,
            description = "Ok",
            body=Debate,
        ),
        (status=400, description = "Bad request"),
        (
            status=401,
            description = "The user is not permitted to read debates within this tournament",
        ),
        (status=404, description = "Tournament or debate not found"),
        (status=500, description = "Internal server error"),
    ),
)]
async fn get_debate_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadDebates) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Debate::get_by_id(id, &state.connection_pool).await {
        Ok(debate) => Ok(Json(debate).into_response()),
        Err(e) => match e {
            OmniError::ResourceNotFoundError => Err(e),
            _ => {
                error!("Error getting a debate with id {id}: {e}");
                Err(e)?
            }
        },
    }
}

/// Patch an existing debate
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "tournament/{tournament_id}/debate/{id}", 
    request_body=DebatePatch,
    responses(
        (
            status=200, description = "Debate patched successfully",
            body=Debate,
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify debates within this tournament"
        ),
        (status=404, description = "Tournament or debate not found"),
        (status=500, description = "Internal server error"),
    )
)]
async fn patch_debate_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((id, tournament_id)): Path<(Uuid, Uuid)>,
    Json(new_debate): Json<DebatePatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteDebates) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let existing_debate = Debate::get_by_id(id, &state.connection_pool).await?;
    match existing_debate
        .patch(new_debate, &state.connection_pool)
        .await
    {
        Ok(debate) => Ok(Json(debate).into_response()),
        Err(e) => {
            error!("Error patching a debate with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Delete an existing debate
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(delete, path = "{tournament_id}/debate/{id}", 
    responses
    (
        (status=204, description = "Debate deleted successfully"),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify debates within this tournament"
        ),
        (status=404, description = "Tournament or debate not found"),
        (status=500, description = "Internal server error"),
    ),
)]
async fn delete_debate_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteDebates) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Debate::get_by_id(id, &state.connection_pool).await {
        Ok(debate) => match debate.delete(&state.connection_pool).await {
            Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
            Err(e) => match e {
                OmniError::ResourceAlreadyExistsError => Err(e),
                _ => Err(e),
            },
        },
        Err(e) => Err(e),
    }
}
