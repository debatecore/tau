use axum::{
    Router, extract::{Path, State}, http::{HeaderMap, StatusCode, Uri}, response::{IntoResponse, Response}, routing::get
};

use std::str::FromStr;
use tower_cookies::Cookies;
use uuid::Uuid;


use crate::{
    omni_error::OmniError, setup::AppState, users::{TournamentUser, permissions::Permission}
};

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/users/:id/tournaments/:tournament_id/permissions", get(has_permission))
        
}

/// Check if a user has a specific permission within a tournament
///
/// This endpoint allows users to check whether they have a specific permission
/// for performing operations within a tournament. This is useful for determining
/// which UI elements to render based on user permissions.
///
/// The endpoint accepts a single `permission_name` query parameter containing
/// one of the valid Permission enum variants.
#[utoipa::path(
    get, 
    path = "/users/{id}/tournaments/{tournament_id}/permissions",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("tournament_id" = Uuid, Path, description = "Tournament ID"),
        ("permission_name" = String, Query, description = "The permission to check. Must be exactly one of: ReadAttendees, WriteAttendees, ReadDebates, WriteDebates, ReadTeams, WriteTeams, ReadMotions, WriteMotions, ReadTournament, WriteTournament, CreateUsersManually, CreateUsersWithLink, DeleteUsers, ModifyUserRoles, SubmitOwnVerdictVote, SubmitVerdict, WriteRoles, ReadLocations, WriteLocations, ReadRooms, ModifyAllRoomDetails, ChangeRoomOccupationStatus, ReadAffiliations, WriteAffiliations, ReadPhases, WritePhases, ReadRounds, WriteRounds"),
    ),
    responses(
        (status=200, description = "Permission check result", body=bool, example=json!(true)),
        (status=400, description = "Bad request - multiple permissions provided or invalid query format"),
        (status=401, description = "Unauthorized - user not assigned to tournament or invalid credentials"),
        (status=404, description = "Permission not found - invalid permission name"),
        (status=500, description = "Internal server error"),
    ),
    tag="users"
)]
async fn has_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    uri: Uri,
    Path((_user_id, tournament_id)): Path<(Uuid, Uuid)>,
) -> Result <Response, OmniError>{
    let pool = &state.connection_pool;
    let tournament_user =
    TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;
    if tournament_user.roles.is_empty() && !tournament_user.user.is_infrastructure_admin() {
        return Err(OmniError::UnauthorizedError);
    }
    // Parse query string to validate permission_name parameter
    let query_string = uri.query().unwrap_or("");
    
    // Check for multiple permission_name parameters and extract value
    let permission_name = extract_single_permission_name(query_string)?;

    // Parse the permission name into the Permission enum
    let permission = Permission::from_str(&permission_name)
        .map_err(|_| OmniError::ResourceNotFoundError)?;

    match tournament_user.has_permission(permission) {
        true => Ok((StatusCode::OK, "true").into_response()),
        false => Ok((StatusCode::OK, "false").into_response()),
    }
}

/// Extract a single permission_name value from query string
/// Returns BadRequestError if multiple permission_name params exist
/// Returns ResourceNotFoundError (404) if no permission_name param found
fn extract_single_permission_name(query_string: &str) -> Result<String, OmniError> {
    if query_string.is_empty() {
        return Err(OmniError::BadRequestError);
    }

    let mut permission_values = Vec::new();
    
    for param in query_string.split('&') {
        if let Some(value) = param.strip_prefix("permission_name=") {
            permission_values.push(value.to_string());
        }
    }

    match permission_values.len() {
        0 => Err(OmniError::BadRequestError),
        1 => Ok(permission_values.into_iter().next().unwrap()),
        _ => Err(OmniError::BadRequestError),
    }
}













    
    
  
