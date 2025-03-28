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

use crate::{omni_error::OmniError, setup::AppState, tournament::{phase::{Phase, PhasePatch}, Tournament}, users::{permissions::Permission, TournamentUser}};

const CONFLICT_MESSAGE: &str = "Conflict";

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament/:tournament_id/phase", get(get_phases).post(create_phase))
        .route(
            "/tournament/:tournament_id/phase/:id",
            get(get_phase_by_id)
                .patch(patch_phase_by_id)
                .delete(delete_phase_by_id),
        )
}

/// Create a new phase
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(post, request_body=Phase, path = "/tournament/{tournament_id}/phase",
    responses
    (
        (
            status=200, description = "Phase created successfully",
            body=Phase,
            example=json!(get_phase_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify phases within this tournament"
        ),
        (status=404, description = "Tournament or phase not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="phase"
)]
async fn create_phase(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(json): Json<Phase>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WritePhases) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    json.validate(pool).await?;

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match Phase::post(json, pool).await {
        Ok(phase) => Ok(Json(phase).into_response()),
        Err(e) => {
            error!("Error creating a new phase: {e}");
            Err(e)
        },
    }
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/phase", 
    responses
    (
        (
            status=200, description = "Ok",
            body=Vec<Phase>,
            example=json!(get_phases_list_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read phases within this tournament"
        ),
        (status=404, description = "Tournament not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="phase"
)]
/// Get a list of all phases
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_phases(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadPhases) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match tournament.get_phases(pool).await {
        Ok(phases) => Ok(Json(phases).into_response()),
        Err(e) => Err(e)?
    }

}

/// Get details of an existing phase
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/tournament/{tournament_id}/phase/{id}", 
    responses(
        (
            status=200, description = "Ok", body=Phase,
            example=json!(get_phase_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read phases within this tournament"
        ),
        (status=404, description = "Tournament or phase not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="phase"
)]
async fn get_phase_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path( (_tournament_id, id)): Path<(Uuid, Uuid)>
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadPhases) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Phase::get_by_id(id, pool).await {
        Ok(phase) => Ok(Json(phase).into_response()),
        Err(e) => {
            error!("Error getting a phase with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Patch an existing phase
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "/tournament/{tournament_id}/phase/{id}", 
    request_body=Phase,
    responses(
        (
            status=200, description = "Phase patched successfully",
            body=Phase,
            example=json!(get_phase_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify phases within this tournament"
        ),
        (status=404, description = "Tournament or phase not found"),
        (
            status=409,
            description = CONFLICT_MESSAGE,
        ),
        (status=500, description = "Internal server error"),
    ),
    tag="phase"
)]
async fn patch_phase_by_id(
    Path((tournament_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(patch): Json<PhasePatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WritePhases) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let phase = Phase::get_by_id(id, pool).await?;
    let new_phase = patch.create_phase_with(phase.clone());
    new_phase.validate(pool).await?;

    match phase.patch(new_phase, pool).await {
        Ok(phase) => Ok(Json(phase).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing phase
///
/// This operation is only allowed when there are no entities
/// referencing this phase. Available only to the tournament Organizers.
#[utoipa::path(delete, path = "/tournament/{tournament_id}/phase/{id}", 
    responses
    (
        (status=204, description = "Phase deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify phases within this tournament"
        ),
        (status=404, description = "Tournament or phase not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="phase"
)]
async fn delete_phase_by_id(
    Path((tournament_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WritePhases) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let phase = Phase::get_by_id(id, pool).await?;
    match phase.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            if e.is_sqlx_foreign_key_violation() {
                return Err(OmniError::DependentResourcesError)
            }
            else {
                error!("Error deleting a phase with id {id}: {e}");
                Err(e)?
            }
        }
    }
}

fn get_phase_example() -> String {
    r#"
    {

        id: "0195d2e8-f9ba-7773-a5dd-65785b6396a1",
        name: "phase 1",
        tournament_id: "0195d2e7-b034-7700-9888-39d4b1c73ada",
        is_finals: false,
        previous_phase_id: "0195d2e8-f9ba-7773-a5dd-65785b6396a1",
        group_size: 4,
        status: "Planned"

    }
    "#
    .to_owned()
}

fn get_phases_list_example() -> String {
    r#"
        {
            id: "0195d2e8-f9ba-7773-a5dd-65785b6396a1",
            name: "phase 1",
            tournament_id: "0195d2e7-b034-7700-9888-39d4b1c73ada",
            is_finals: false,
            group_size: 4,
            status: "Ongoing"
        },
        {
            id: "0195d2ea-74ab-7a82-9435-64f807042389",
            name: "phase 2",
            tournament_id: "0195d2e7-b034-7700-9888-39d4b1c73ada",
            is_finals: true,
            previous_phase_id: "0195d2e8-f9ba-7773-a5dd-65785b6396a1",
            group_size: null,
            status: "Planned"
        }
    ]
    "#
    .to_owned()
}
