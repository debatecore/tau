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

use crate::{omni_error::OmniError, setup::AppState, tournament::{phase::Phase, round::{Round, RoundPatch}, Tournament}, users::{permissions::Permission, TournamentUser}};

const DUPLICATE_NAME_ERROR: &str = "Round with this name already exists within the scope of the tournament, to which the round is assigned.";

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament/:tournament_id/phase/:phase_id/round", get(get_rounds).post(create_round))
        .route(
            "/tournament/:tournament_id/phase/:phase_id/round/:id",
            get(get_round_by_id)
                .patch(patch_round_by_id)
                .delete(delete_round_by_id),
        )
}

/// Create a new round
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(post, request_body=Round, path = "/tournament/{tournament_id}/round",
    responses
    (
        (
            status=200, description = "Round created successfully",
            body=Round,
            example=json!(get_round_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify rounds within this tournament"
        ),
        (status=404, description = "Tournament or round not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="round"
)]
async fn create_round(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, phase_id)): Path<(Uuid, Uuid)>,
    Json(json): Json<Round>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteRounds) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    json.validate(pool).await?;

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match Round::post(json, pool).await {
        Ok(round) => Ok(Json(round).into_response()),
        Err(e) => {
            error!("Error creating a new round: {e}");
            Err(e)
        },
    }
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/phase/{phase_id}/round", 
    responses
    (
        (
            status=200, description = "Ok",
            body=Vec<Round>,
            example=json!(get_rounds_list_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read rounds within this tournament"
        ),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="round"
)]
/// Get a list of all rounds
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_rounds(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, phase_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadRounds) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let phase = Phase::get_by_id(phase_id, pool).await?;
    match phase.get_rounds(pool).await {
        Ok(rounds) => Ok(Json(rounds).into_response()),
        Err(e) => Err(e)?
    }

}

/// Get details of an existing round
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/tournament/{tournament_id}/round/{id}", 
    responses(
        (
            status=200, description = "Ok", body=Round,
            example=json!(get_round_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read rounds within this tournament"
        ),
        (status=404, description = "Tournament or round not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="round"
)]
async fn get_round_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path( (_tournament_id, id)): Path<(Uuid, Uuid)>
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadRounds) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Round::get_by_id(id, pool).await {
        Ok(round) => Ok(Json(round).into_response()),
        Err(e) => {
            error!("Error getting a round with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Patch an existing round
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "/tournament/{tournament_id}/round/{id}", 
    request_body=Round,
    responses(
        (
            status=200, description = "Round patched successfully",
            body=Round,
            example=json!(get_round_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify rounds within this tournament"
        ),
        (status=404, description = "Tournament or round not found"),
        (
            status=409,
            description = DUPLICATE_NAME_ERROR,
        ),
        (status=500, description = "Internal server error"),
    ),
    tag="round"
)]
async fn patch_round_by_id(
    Path((tournament_id, phase_id, id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(new_round): Json<RoundPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteRounds) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let round = Round::get_by_id(id, pool).await?;
    round.validate(pool).await?;

    match round.patch(new_round, pool).await {
        Ok(round) => Ok(Json(round).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing round
///
/// This operation is only allowed when there are no entities
/// referencing this round. Available only to the tournament Organizers.
#[utoipa::path(delete, path = "/tournament/{tournament_id}/round/{id}", 
    responses
    (
        (status=204, description = "Round deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify rounds within this tournament"
        ),
        (status=404, description = "Tournament or round not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="round"
)]
async fn delete_round_by_id(
    Path((tournament_id, phase_id, id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteRounds) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let round = Round::get_by_id(id, pool).await?;
    match round.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a round with id {id}: {e}");
            Err(e)?
        }
    }
}

fn get_round_example() -> String {
    r#"
    {
        "address": "Poznań, Poland",
        "name": "ZSK",
        "remarks": "Where debatecore was born",
        "tournament_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6"
    }
    "#
    .to_owned()
}

fn get_rounds_list_example() -> String {
    r#"
    [
        {
            "address": "Poznań, Poland",
            "name": "ZSK",
            "remarks": "Where debatecore was born",
            "tournament_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6"
        },
        {
            "address": "Bydgoszcz, Poland",
            "name": "Library of the Kazimierz Wielki University",
            "remarks": "Where Debate Team Buster prevailed",
            "tournament_id": "57a85f64-5784-4562-4acc-35163f66afa6"
        },
    ]
    "#
    .to_owned()
}
