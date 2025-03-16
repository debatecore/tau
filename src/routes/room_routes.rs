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

use crate::{omni_error::OmniError, setup::AppState, tournament::{location::Location, room::{Room, RoomPatch}, Tournament}, users::{permissions::Permission, TournamentUser}};

const DUPLICATE_NAME_ERROR: &str = "Room with this name already exists within the scope of the tournament, to which the room is assigned.";

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament/:tournament_id/location/:location_id/room", get(get_rooms).post(create_room))
        .route(
            "/tournament/:tournament_id/location/:location_id/room/:id",
            get(get_room_by_id)
                .patch(patch_room_by_id)
                .delete(delete_room_by_id),
        )
}

/// Create a new room
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(post, request_body=Room, path = "/tournament/{tournament_id}/location/{location_id}/room",
    responses
    (
        (
            status=200, description = "Room created successfully",
            body=Room,
            example=json!(get_room_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify rooms within this tournament"
        ),
        (status=404, description = "Tournament or room not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="room"
)]
async fn create_room(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, _location_id)): Path<(Uuid, Uuid)>,
    Json(json): Json<Room>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ModifyAllRoomDetails) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    if Room::room_with_name_exists_in_location(&json.name, &tournament_id, pool).await? {
        return Err(OmniError::ResourceAlreadyExistsError);
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match Room::post(json, pool).await {
        Ok(room) => Ok(Json(room).into_response()),
        Err(e) => {
            error!("Error creating a new room: {e}");
            Err(e)
        },
    }
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/location/{location_id}/room", 
    responses
    (
        (
            status=200, description = "Ok",
            body=Vec<Room>,
            example=json!(get_rooms_list_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read rooms within this tournament"
        ),
        (status=404, description = "Tournament or room not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="room"
)]
/// Get a list of all rooms within a location
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_rooms(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadRooms) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let location = Location::get_by_id(location_id, pool).await?;
    match location.get_rooms(pool).await
    {
        Ok(rooms) => Ok(Json(rooms).into_response()),
        Err(e) => {
            error!("Error getting a list of rooms: {e}");
            Err(e)?
        }
    }
}

/// Get details of an existing room
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/tournament/{tournament_id}/location/{location_id}/room/{id}", 
    responses(
        (
            status=200, description = "Ok", body=Room,
            example=json!(get_room_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read rooms within this tournament"
        ),
        (status=404, description = "Tournament or room not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="room"
)]
async fn get_room_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path((tournament_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadRooms) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Room::get_by_id(id, pool).await {
        Ok(room) => Ok(Json(room).into_response()),
        Err(e) => {
            error!("Error getting a room with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Patch an existing room
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "/tournament/{tournament_id}/location/{location_id}/room/{id}", 
    request_body=Room,
    responses(
        (
            status=200, description = "Room patched successfully",
            body=Room,
            example=json!(get_room_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify rooms within this tournament"
        ),
        (status=404, description = "Tournament or room not found"),
        (
            status=409,
            description = DUPLICATE_NAME_ERROR,
        ),
        (status=500, description = "Internal server error"),
    ),
    tag="room"
)]
async fn patch_room_by_id(
    Path(( tournament_id, _location_id, id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(new_room): Json<RoomPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ModifyAllRoomDetails) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let room = Room::get_by_id(id, pool).await?;
    let new_name = new_room.name.clone();
    if new_name.is_some() {
        if Room::room_with_name_exists_in_location(&new_name.unwrap(), &room.location_id, pool).await? {
            return Err(OmniError::ResourceAlreadyExistsError)
        }
    }

    match room.patch(new_room, pool).await {
        Ok(room) => Ok(Json(room).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing room
///
/// This operation is only allowed when there are no entities
/// referencing this room. Available only to the tournament Organizers.
#[utoipa::path(delete, path = "/tournament/{tournament_id}/location/{location_id}/room/{id}", 
    responses
    (
        (status=204, description = "Room deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify rooms within this tournament"
        ),
        (status=404, description = "Tournament or room not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="room"
)]
async fn delete_room_by_id(
    Path((tournament_id, _location_id, id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ModifyAllRoomDetails) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Location::get_by_id(_location_id, pool).await {
        Ok(_) => (),
        Err(e) => {
            if  e.is_not_found_error() {
                return Err(OmniError::ResourceNotFoundError);
            }
        } 
    }

    let room = Room::get_by_id(id, pool).await?;
    match room.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a room with id {id}: {e}");
            Err(e)?
        }
    }
}

fn get_room_example() -> String {
    r#"
    {
        "is_occupied": true,
        "location_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "name": "Room 32",
        "remarks": "Third floor"
    }
    "#
    .to_owned()
}

fn get_rooms_list_example() -> String {
    r#"
    [
        {
            "is_occupied": true,
            "location_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
            "name": "Room 32",
            "remarks": "Third floor"
        },
        {
            "is_occupied": true,
            "location_id": "77abaf34-5782-4562-b3fc-93963f66afa6",
            "name": "Room 44",
            "remarks": "Fourth floor"
        }
    ]
    "#
    .to_owned()
}
