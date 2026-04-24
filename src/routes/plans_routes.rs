use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sqlx::{query, Error, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::{omni_error::OmniError, setup::AppState, tournaments::{plans::{TournamentPlan, TournamentPlanPatch}, Tournament}, users::{permissions::Permission, TournamentUser}};

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournaments/{tournament_id}/plan", get(get_plan).post(create_plan))
        .route(
            "/tournaments/{tournament_id}/plan/{id}",
            get(get_plan_by_id)
                .patch(patch_plan_by_id)
                .delete(delete_plan_by_id),
        )
}

/// Create a new tournament plan
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(post, request_body=TournamentPlan, path = "/tournaments/{tournament_id}/plan",
    responses
    (
        (
            status=200, description = "Plan created successfully",
            body=TournamentPlan,
            example=json!(get_plan_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify plan within this tournament"
        ),
        (status=404, description = "Tournament or plan not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="plan"
)]
async fn create_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(json): Json<TournamentPlan>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WritePlan) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    json.validate()?;

    match TournamentPlan::post(tournament_id, json, pool).await {
        Ok(plan) => Ok(axum::Json::<TournamentPlan>(plan).into_response()),
        Err(e) => {
            error!("Error creating a new plan: {e}");
            Err(e)
        },
    }
}

#[utoipa::path(get, path = "/tournaments/{tournament_id}/plan", 
    responses
    (
        (
            status=200, description = "Ok",
            body=TournamentPlan,
            example=json!(get_plan_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read plan within this tournament"
        ),
        (status=404, description = "Tournament or plan not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="plan"
)]
/// Get a plan
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadPlan) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match tournament.get_plan(pool).await
    {
        Ok(plan) => Ok(Json(plan).into_response()),
        Err(e) => {
            error!("Error getting a tournament plan: {e}");
            Err(e)?
        }
    }
}

/// Get details of an existing plan
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/tournaments/{tournament_id}/plan/{id}", 
    responses(
        (
            status=200, description = "Ok", body=TournamentPlan,
            example=json!(get_plan_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read plan within this tournament"
        ),
        (status=404, description = "Tournament or plan not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="plan"
)]
async fn get_plan_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadPlan) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match TournamentPlan::get_by_id(id, pool).await {
        Ok(plan) => Ok(axum::Json::<TournamentPlan>(plan).into_response()),
        Err(e) => {
            error!("Error getting a plan with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Patch an existing plan
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "/tournaments/{tournament_id}/plan/{id}", 
    request_body=TournamentPlan,
    responses(
        (
            status=200, description = "Plan patched successfully",
            body=TournamentPlan,
            example=json!(get_plan_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify plan within this tournament"
        ),
        (status=404, description = "Tournament or plan not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="plan"
)]
async fn patch_plan_by_id(
    Path((tournament_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(new_plan): Json<TournamentPlanPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WritePlan) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    new_plan.validate()?;

    let plan = TournamentPlan::get_by_id(id, pool).await?;
    match plan.patch(new_plan, pool).await {
        Ok(plan) => Ok(axum::Json::<TournamentPlan>(plan).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing plan
///
/// This operation is only allowed when there are no entities
/// referencing this team. Available only to the tournament Organizers.
#[utoipa::path(delete, path = "/tournaments/{tournament_id}/plan/{id}", 
    responses
    (
        (status=204, description = "Plan deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify plan within this tournament"
        ),
        (status=404, description = "Tournament or plan not found"),
    ),
    tag="plan"
)]
async fn delete_plan_by_id(
    Path((tournament_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WritePlan) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let plan = TournamentPlan::get_by_id(id, pool).await?;
    match plan.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a plan with id {id}: {e}");
            Err(e)?
        }
    }
}

fn get_plan_example() -> String {
    r#"{
        "id": "01941267-2685-7a62-8382-c90fae07a87b",
        "groups_count": 6,
        "group_phase_rounds": 3,
        "advancing_teams": 16,
        "total_teams": 32
        "tournament_id": "01941267-0109-7405-b30e-7883d309c603"
    }"#
    .to_owned()
}