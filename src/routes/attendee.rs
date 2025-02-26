use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Error, Pool, Postgres};
use tower_cookies::Cookies;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    setup::AppState,
    users::{permissions::Permission, TournamentUser},
};

const POSITION_OUT_OF_RANGE_MESSAGE: &str = "Attendee position must be in range of 1-4.";

#[serde_inline_default]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Attendee {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    id: Uuid,
    name: String,
    /// Represents the attendee's position as a speaker
    /// (1 for the 1st speaker, 2 for the 2nd speaker, etc.).
    /// If the attendee is not a speaker, but is nonetheless
    /// affiliated with the team, the position should be null.
    /// Two attendees from the same team cannot be placed on the same position.
    position: Option<i32>,
    team_id: Uuid,
    #[serde_inline_default(0)]
    individual_points: i32,
    #[serde_inline_default(0)]
    penalty_points: i32,
}

#[derive(Deserialize, ToSchema)]
pub struct AttendeePatch {
    name: Option<String>,
    position: Option<i32>,
    team_id: Option<Uuid>,
    individual_points: Option<i32>,
    penalty_points: Option<i32>,
}

impl Attendee {
    async fn post(
        attendee: Attendee,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Attendee, OmniError> {
        match query_as!(
            Attendee,
            r#"INSERT INTO attendees
            (id, name, position, team_id, individual_points, penalty_points)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, position, team_id, individual_points, penalty_points"#,
            attendee.id,
            attendee.name,
            attendee.position,
            attendee.team_id,
            attendee.individual_points,
            attendee.penalty_points
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(attendee) => Ok(attendee),
            Err(e) => Err(e)?,
        }
    }

    async fn get_by_id(
        id: Uuid,
        connection_pool: &Pool<Postgres>,
    ) -> Result<Attendee, OmniError> {
        match query_as!(Attendee, "SELECT * FROM attendees WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(attendee) => Ok(attendee),
            Err(e) => Err(e)?,
        }
    }

    async fn patch(
        self,
        connection_pool: &Pool<Postgres>,
        patch: AttendeePatch,
    ) -> Result<Attendee, Error> {
        let new_attendee = Attendee {
            id: self.id,
            name: patch.name.unwrap_or(self.name),
            position: patch.position,
            team_id: patch.team_id,
            individual_points: patch.individual_points.unwrap_or(self.individual_points),
            penalty_points: patch.penalty_points.unwrap_or(self.penalty_points),
        };
        match query!(
            r#"UPDATE attendees SET name = $1, position = $2, team_id = $3,
            individual_points = $4, penalty_points = $5 WHERE id = $6"#,
            new_attendee.name,
            new_attendee.position,
            new_attendee.team_id,
            new_attendee.individual_points,
            new_attendee.penalty_points,
            new_attendee.id
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(new_attendee),
            Err(e) => Err(e),
        }
    }

    async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), Error> {
        match query!("DELETE FROM attendees WHERE id = $1", self.id)
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
        (status=400, description = "Bad request",),
        (
            status=401,
            description = "The user is not permitted to create attendees within this tournament",
        ),
        (status=404, description = "Tournament not found"),
        (
            status=409, description = "Attendee position is duplicated",
        ),
        (
            status=500, description = "Internal server error",
        ),
    )
)]
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
        false => return Err(OmniError::UnauthorizedError),
    }

    if !attendee.position.is_none() {
        if !attendee_position_is_valid(attendee.position.unwrap()) {
            return Err(OmniError::BadRequestError);
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
        (
            status=401,
            description = "The user is not get to create attendees within this tournament",
        ),
        (status=404, description = "Tournament not found"),
        (
            status=500, description = "Internal server error",
        ),
    )
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
        false => return Err(OmniError::UnauthorizedError),
    }

    match query_as!(Attendee, "SELECT * FROM attendees")
        .fetch_all(&state.connection_pool)
        .await
    {
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
        (
            status=401,
            description = "The user is not permitted to get attendees within this tournament",
        ),
        (status=404, description = "Tournament or attendee not found"),
        (
            status=500, description = "Internal server error",
        ),
    ),
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
        false => return Err(OmniError::UnauthorizedError),
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
        (
            status=401,
            description = "The user is not permitted to patch attendees within this tournament",
        ),
        (status=404, description = "Tournament or attendee not found"),
        (status=409, description = "Attendee position is duplicated"),
        (status=422, description = "Attendee position out of range [1-4]"),
        (status=500, description = "Internal server error"),
    )
)]
async fn patch_attendee_by_id(
    Path(id): Path<Uuid>,
    Path(tournament_id): Path<Uuid>,
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
        false => return Err(OmniError::UnauthorizedError),
    }

    if !new_attendee.position.is_none() {
        if !attendee_position_is_valid(new_attendee.position.unwrap()) {
            return Ok((
                StatusCode::UNPROCESSABLE_ENTITY,
                POSITION_OUT_OF_RANGE_MESSAGE,
            )
                .into_response());
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
        (
            status=401,
            description = "The user is not permitted to delete attendees within this tournament",
        ),
        (status=404, description = "Tournament or attendee not found"),
        (
            status=500, description = "Internal server error",
        ),
    ),
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
        false => return Err(OmniError::UnauthorizedError),
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
) -> Result<bool, Error> {
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
            Err(e)
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
