use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sqlx::{query, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    tournament::attendee::{Attendee, AttendeePatch},
    users::{permissions::Permission, TournamentUser},
};

const POSITION_OUT_OF_RANGE_MESSAGE: &str = "Attendee position must be in range of 1-4.";

pub fn route() -> Router<AppState> {
    Router::new()
        .route(
            "/tournament/:tournament_id/attendee",
            post(create_attendee).get(get_attendees),
        )
        .route(
            "/:tournament_id/attendee/:id",
            get(get_attendee_by_id)
                .patch(patch_attendee_by_id)
                .delete(delete_attendee_by_id),
        )
}

/// Create an attendee
///
/// Requires the WritesAttendee permission.
#[utoipa::path(
    post,
    request_body=Attendee,
    path = "/tournament/{tournament_id}/attendee",
    responses(
        (
            status=200, description = "Attendee created successfully",
            body=Attendee,
            example=json!(get_attendee_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to create attendees within this tournament",
        ),
        (status=404, description = "Tournament not found"),
        (
            status=409, description = "Attendee position is duplicated",
        ),
        (
            status=500, description = "Internal server error",
        ),
    ),
    tag="attendee"
)]
#[axum::debug_handler]
async fn create_attendee(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(attendee): Json<Attendee>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAttendees) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    if !attendee.position.is_none() {
        if !attendee_position_is_valid(attendee.position.unwrap()) {
            return Err(OmniError::ExplicitError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                message: POSITION_OUT_OF_RANGE_MESSAGE.to_owned(),
            });
        }
        match attendee_position_is_duplicated(&attendee, pool).await {
            Ok(position_duplicated) => {
                if position_duplicated {
                    return Err(OmniError::ResourceAlreadyExistsError);
                }
            }
            Err(_) => return Err(OmniError::InternalServerError),
        }
    }

    match Attendee::post(attendee, pool).await {
        Ok(attendee) => Ok(Json(attendee).into_response()),
        Err(e) => Err(e)?,
    }
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/attendee", 
    responses(
        (
            status=200,
            description = "Ok",
            body=Vec<Attendee>,
            example=json!(get_attendees_list_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not get to create attendees within this tournament",
        ),
        (status=404, description = "Tournament not found"),
        (
            status=500, description = "Internal server error",
        ),
    ),
    tag="attendee"
)]
/// Get a list of all attendees
async fn get_attendees(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAttendees) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Attendee::get_all(pool).await {
        Ok(attendees) => Ok(Json(attendees).into_response()),
        Err(e) => {
            error!("Error getting a list of attendees: {e}");
            Err(e)?
        }
    }
}

/// Get details of an existing attendee
#[utoipa::path(get, path = "/tournament/{tournament_id}/attendee/{id}", 
    responses(
        (
            status=200, description = "Ok", body=Attendee,
            example=json!(get_attendee_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to get attendees within this tournament",
        ),
        (status=404, description = "Tournament or attendee not found"),
        (
            status=500, description = "Internal server error",
        ),
    ),
    tag="attendee"
)]
async fn get_attendee_by_id(
    Path(id): Path<Uuid>,
    Path(tournament_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadAttendees) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Attendee::get_by_id(id, &state.connection_pool).await {
        Ok(attendee) => Ok(Json(attendee).into_response()),
        Err(e) => match e {
            OmniError::ResourceNotFoundError => Err(e),
            _ => Err(e)?,
        },
    }
}

/// Patch an existing attendee
#[utoipa::path(patch, path = "/tournament/{tournament_id}/attendee/{id}", 
    request_body=AttendeePatch,
    responses(
        (
            status=200, description = "Attendee patched successfully",
            body=Attendee,
            example=json!(get_attendee_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to patch attendees within this tournament",
        ),
        (status=404, description = "Tournament or attendee not found"),
        (status=409, description = "Attendee position is duplicated"),
        (status=422, description = "Attendee position out of range [1-4]"),
        (status=500, description = "Internal server error"),
    ),
    tag="attendee"
)]
async fn patch_attendee_by_id(
    Path((id, tournament_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(new_attendee): Json<AttendeePatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAttendees) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    if !new_attendee.position.is_none() {
        if !attendee_position_is_valid(new_attendee.position.unwrap()) {
            return Err(OmniError::ExplicitError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                message: POSITION_OUT_OF_RANGE_MESSAGE.to_owned(),
            });
        }
    }
    let attendee = Attendee::get_by_id(id, pool).await?;
    let position_is_unique = attendee_position_is_duplicated(&attendee, pool).await?;
    if !position_is_unique {
        return Err(OmniError::ResourceAlreadyExistsError);
    }

    match attendee.patch(pool, new_attendee).await {
        Ok(attendee) => Ok(Json(attendee).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing attendee
#[utoipa::path(delete, path = "/tournament/{tournament_id}/attendee/{id}", 
    responses
    (
        (status=204, description = "Attendee deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403,
            description = "The user is not permitted to delete attendees within this tournament",
        ),
        (status=404, description = "Tournament or attendee not found"),
        (
            status=500, description = "Internal server error",
        ),
    ),
    tag="attendee"
)]
async fn delete_attendee_by_id(
    Path(id): Path<Uuid>,
    Path(tournament_id): Path<Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteAttendees) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let attendee = Attendee::get_by_id(id, pool).await?;
    match attendee.delete(pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => Err(e)?,
    }
}

fn attendee_position_is_valid(position: i32) -> bool {
    return position >= 1 && position <= 4;
}

async fn attendee_position_is_duplicated(
    attendee: &Attendee,
    connection_pool: &Pool<Postgres>,
) -> Result<bool, OmniError> {
    match query!(
        "SELECT EXISTS(SELECT 1 FROM attendees WHERE team_id = $1 AND position = $2)",
        attendee.team_id,
        attendee.position
    )
    .fetch_one(connection_pool)
    .await
    {
        Ok(result) => Ok(result.exists.unwrap()),
        Err(e) => {
            error!("Error checking speaker position uniqueness: {e}");
            Err(e)?
        }
    }
}

fn get_attendee_example() -> String {
    r#"
    {
    "id": "019411fd-9665-77f0-9829-217f1df749ad",
    "name": "John Doe",
    "position": 2,
    "team_id": "01941266-18d3-72d3-b48b-49cabe6a91c2",
    "individual_points": 15,
    "penalty_points": 0
    }
    "#
    .to_owned()
}

fn get_attendees_list_example() -> String {
    r#"
    [
    {
    "id": "019411fd-9665-77f0-9829-217f1df749ad",
    "name": "John Doe",
    "position": 2,
    "team_id": "01941266-18d3-72d3-b48b-49cabe6a91c2",
    "individual_points": 15,
    "penalty_points": 0
    },
    {
    "id": "01941265-f629-76b7-a13b-e387d3fcad10",
    "name": "Melinda Landsgale",
    "position": 3,
    "team_id": "01941266-18d3-72d3-b48b-49cabe6a91c2",
    "individual_points": 17,
    "penalty_points": 8
    }
    ]
    "#
    .to_owned()
}
