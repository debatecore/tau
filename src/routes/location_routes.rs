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

use crate::{omni_error::OmniError, setup::AppState, tournament::{location::{Location, LocationPatch}, Tournament}, users::{permissions::Permission, TournamentUser}};

const DUPLICATE_NAME_ERROR: &str = "Location with this name already exists within the scope of the tournament, to which the location is assigned.";

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tournament/:tournament_id/location", get(get_locations).post(create_location))
        .route(
            "/tournament/:tournament_id/location/:id",
            get(get_location_by_id)
                .patch(patch_location_by_id)
                .delete(delete_location_by_id),
        )
}

/// Create a new location
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(post, request_body=Location, path = "/tournament/{tournament_id}/location",
    responses
    (
        (
            status=200, description = "Location created successfully",
            body=Location,
            example=json!(get_location_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify locations within this tournament"
        ),
        (status=404, description = "Tournament or location not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="location"
)]
async fn create_location(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
    Json(json): Json<Location>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteLocations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    if Location::location_with_name_exists_in_tournament(&json.name, &tournament_id, pool).await? {
        return Err(OmniError::ResourceAlreadyExistsError);
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match Location::post(json, pool).await {
        Ok(location) => Ok(Json(location).into_response()),
        Err(e) => {
            error!("Error creating a new location: {e}");
            Err(e)
        },
    }
}

#[utoipa::path(get, path = "/tournament/{tournament_id}/location", 
    responses
    (
        (
            status=200, description = "Ok",
            body=Vec<Location>,
            example=json!(get_locations_list_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read locations within this tournament"
        ),
        (status=404, description = "Tournament or location not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="location"
)]
/// Get a list of all locations
/// 
/// The user must be given a role within this tournament to use this endpoint.
async fn get_locations(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path(tournament_id): Path<Uuid>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadLocations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let _tournament = Tournament::get_by_id(tournament_id, pool).await?;
    match query_as!(Location, "SELECT * FROM locations WHERE tournament_id = $1", tournament_id)
        .fetch_all(&state.connection_pool)
        .await
    {
        Ok(locations) => Ok(Json(locations).into_response()),
        Err(e) => {
            error!("Error getting a list of locations: {e}");
            Err(e)?
        }
    }
}

/// Get details of an existing location
/// 
/// The user must be given a role within this tournament to use this endpoint.
#[utoipa::path(get, path = "/tournament/{tournament_id}/location/{id}", 
    responses(
        (
            status=200, description = "Ok", body=Location,
            example=json!(get_location_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to read locations within this tournament"
        ),
        (status=404, description = "Tournament or location not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="location"
)]
async fn get_location_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Path( (_tournament_id, id)): Path<(Uuid, Uuid)>
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::ReadLocations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    match Location::get_by_id(id, pool).await {
        Ok(location) => Ok(Json(location).into_response()),
        Err(e) => {
            error!("Error getting a location with id {id}: {e}");
            Err(e)?
        }
    }
}

/// Patch an existing location
/// 
/// Available only to the tournament Organizers.
#[utoipa::path(patch, path = "/tournament/{tournament_id}/location/{id}", 
    request_body=Location,
    responses(
        (
            status=200, description = "Location patched successfully",
            body=Location,
            example=json!(get_location_example())
        ),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify locations within this tournament"
        ),
        (status=404, description = "Tournament or location not found"),
        (
            status=409,
            description = DUPLICATE_NAME_ERROR,
        ),
        (status=500, description = "Internal server error"),
    ),
    tag="location"
)]
async fn patch_location_by_id(
    Path((_tournament_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    Json(new_location): Json<LocationPatch>,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteLocations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let location = Location::get_by_id(id, pool).await?;
    
    let name = new_location.name.clone();
    if name.is_some() {
        if Location::location_with_name_exists_in_tournament(&name.unwrap(), &location.tournament_id, pool).await? {
            return Err(OmniError::ResourceAlreadyExistsError);
        }
    }

    match location.patch(new_location, pool).await {
        Ok(location) => Ok(Json(location).into_response()),
        Err(e) => Err(e)?,
    }
}

/// Delete an existing location
///
/// This operation is only allowed when there are no entities
/// referencing this location. Available only to the tournament Organizers.
#[utoipa::path(delete, path = "/tournament/{tournament_id}/location/{id}", 
    responses
    (
        (status=204, description = "Location deleted successfully"),
        (status=400, description = "Bad request"),
        (status=401, description = "Authentication error"),
        (
            status=403, 
            description = "The user is not permitted to modify locations within this tournament"
        ),
        (status=404, description = "Tournament or location not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="location"
)]
async fn delete_location_by_id(
    Path((tournament_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, OmniError> {
    let pool = &state.connection_pool;
    let tournament_user =
        TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    match tournament_user.has_permission(Permission::WriteLocations) {
        true => (),
        false => return Err(OmniError::InsufficientPermissionsError),
    }

    let location = Location::get_by_id(id, pool).await?;
    match location.delete(&state.connection_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => {
            error!("Error deleting a location with id {id}: {e}");
            Err(e)?
        }
    }
}

fn get_location_example() -> String {
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

fn get_locations_list_example() -> String {
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
