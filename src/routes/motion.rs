use crate::{omni_error::OmniError, setup::AppState, tournament_impl::motion_impl::{Motion, MotionPatch}, users::{permissions::Permission, TournamentUser}};
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

const DUPLICATE_MOTION_ERROR: &str = "Motion with such content already exists";

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament/:tournament/motion", get(get_motions).post(create_motion))
        .route(
            "/tournament/:tournament_id/motion/:id",
            get(get_motion_by_id)
                .delete(delete_motion_by_id)
                .patch(patch_motion_by_id),
        )
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/motion", 
    responses((
    status=200, description = "Ok",
    body=Vec<Motion>,
    example=json!(get_motions_list_example())
)))]
/// Get a list of all motions
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_motions(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::ReadMotions) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match query_as!(Motion, "SELECT * FROM motions")
        .fetch_all(pool)
        .await
    {
        Ok(motions) => Ok(Json(motions).into_response()),
        Err(e) => {
            error!("Error getting a list of motions: {e}");
            Err(e)?
        }
    }
}

/// Create a new motion
/// 
/// Available only to Organizers and Admins.
#[utoipa::path(
    post,
    request_body=Motion,
    path = "/tournament/{tournament_id}/motion",
    responses(
        (
        status=200, description = "Motion created successfully",
        body=Motion, 
        example=json!(get_motion_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify motions within this tournament"
        ),
        (status=404, description = "Tournament or motion not found"),
        (status=409, description = DUPLICATE_MOTION_ERROR)
    
    )
)]
async fn create_motion(
State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(json): Json<Motion>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::WriteMotions) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Motion::post(json, &state.connection_pool).await {
        Ok(motion) => {
            Ok(Json(motion).into_response())
        },
        Err(e) => Err(e)?
    }
}

/// Get details of an existing motion
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/motion/{id}", 
    responses((status=200, description = "Ok", body=Motion,
    example=json!(get_motion_example())
    )),
)]
async fn get_motion_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadMotions) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    match Motion::get_by_id(id, &state.connection_pool).await {
        Ok(motion) => Ok(Json(motion).into_response()),
        Err(e) => Err(e)?
    }
}

/// Patch an existing motion
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "/tournament/{tournament_id}/motion/{id}", 
    request_body=MotionPatch,
    responses(
        (
            status=200, description = "Motion patched successfully",
            body=Motion,
            example=json!(get_motion_example())
        ),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify motions within this tournament"
        ),
        (status=404, description = "Tournament or motion not found")
    )
)]
async fn patch_motion_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(new_motion): Json<MotionPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::WriteMotions) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let existing_motion = Motion::get_by_id(id, pool).await?;
    match existing_motion.patch(new_motion, &state.connection_pool).await {
        Ok(patched_motion) => Ok(Json(patched_motion).into_response()),
        Err(e) => Err(e)
    }
}

/// Delete an existing motion
/// This operation is only allowed when there are no entities (i.e. debates)
/// referencing this tournament. Available only to the tournament Organizers.
#[utoipa::path(delete, path = "/tournament/{tournament_id}/motion/{id}", 
    responses
    (
        (status=204, description = "Motion deleted successfully"),
        (status=400, description = "Bad request"),
        (
            status=401, 
            description = "The user is not permitted to modify motions within this tournament"
        ),
        (status=404, description = "Tournament or motion not found")
    ),
    
)]
async fn delete_motion_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, pool).await?;

    match tournament_user.has_permission(Permission::WriteMotions) {
        true => (),
        false => return Err(OmniError::UnauthorizedError),
    }

    let motion = Motion::get_by_id(id, pool).await?;
    match motion.delete(pool).await {
            Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
            Err(e) => Err(e)?
    }
}

fn get_motion_example() -> String {
    r#"
    {
    "id": "01941266-8dda-7e88-82ab-38180d9d8e27",
    "motion": "This House Would abolish the UN Security Council."
    }
    "#
    .to_owned()
}

fn get_motions_list_example() -> String {
    r#"
    [
        {
        "id": "01941266-8dda-7e88-82ab-38180d9d8e27",
        "motion": "This House Would abolish the UN Security Council."
        },
        {
        "id": "01941266-725b-7d8d-be4e-4f71bb0d0e1c",
        "motion": "As a society of a newly established state, we would opt for a representative democracy system.",
        "adinfo": "In the middle of the Baltic Sea, an island with a population has appeared. The new state of 'Balticstan' is seeking the best political system to govern itself. The country has guaranteed independence and is sovereign over regional powers at the time of the debate. Balticstan represents the maximum average of all countries bordering the Baltic Sea (nine countries in total) regarding population, economy, problems and opportunities."
        }
    ]
    "#.to_owned()
}
