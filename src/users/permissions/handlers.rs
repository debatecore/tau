use axum::{
    extract::{Path, Query, RawQuery, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
    users::{permissions::Permission, TournamentUser},
};

// ---------------------------------------------------------------------------
// Query-string shape
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PermissionQuery{
    permission_name: String, 
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------
 
/// Check whether a user has a specific permission within a tournament.
///
/// Returns `true` if the user holds the requested permission, `false` otherwise.
///
/// # Errors
/// * **400 Bad Request** – more than one `permission_name` value was supplied.
/// * **401 Unauthorized** – the authenticated user is not a member of this tournament.
/// * **404 Not Found** – the supplied `permission_name` does not correspond to any
///   known [`Permission`] variant.

#[utoipa::path(
    get,
    path = "/user/{id}/tournaments/{tournament_id}/permissions",
    params(
        ("id"            = Uuid, Path, description = "UUID of the user whose permissions are being queried"),
        ("tournament_id" = Uuid, Path, description = "UUID of the tournament"),
        ("permission_name" = String, Query, description = "Exact name of the permission to check, e.g. `WriteTeams`. \
         Must be supplied exactly once. Supplying multiple values returns 400."),
    ),
    responses(
        (status = 200, description = "Permission check result", body = bool,
         example = json!(true)),
        (status = 400, description = "Multiple `permission_name` query parameters were provided"),
        (status = 401, description = "The user is not assigned to this tournament"),
        (status = 404, description = "The supplied `permission_name` is not a recognised permission"),
    ),
    tag = "Permissions",
)]

pub async fn get_user_tournament_permission(
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
    RawQuery(raw_query): RawQuery,
    Query(query): Query<PermissionQuery>,
    headers: HeaderMap,
    cookies: Cookies,
    State(pool): State<Pool<Postgres>>,
) -> Result<impl IntoResponse, OmniError> {
    // -----------------------------------------------------------------------
    // 400 – reject if the caller supplied more than one permission_name value.
    // -----------------------------------------------------------------------
    let raw = raw_query.unwrap_or_default();
    let permission_name_conut = raw
        .split('&')
        .filter(|kv| kv.starts_with("permission_name= "))
        .count();

    if permission_name_count > 1 {
        return Ok ((
            StatusCode::BAD_REQUEST;
            Json(serde_json::json!({
                "error": "Exactly one `permission_name` must be provided. \
                          Multiple values are not supported."
            })),
        )
        .into_response());
    }

// -----------------------------------------------------------------------
    // 404 – parse the permission name; unknown names are rejected here.
    // -----------------------------------------------------------------------
    
    let permission: Permission = query
        .permission_name
        .parse()
        .map_err(|_| OmniError::ResourceNotFound)?;
    
    // -----------------------------------------------------------------------
    // 401 – authenticate; if the user is not in the tournament this returns
    // Err which maps to 401 via OmniError's IntoResponse impl.
    // -----------------------------------------------------------------------
        
    let tournament_user = TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

     // -----------------------------------------------------------------------
    // 200 – delegate to the existing has_permission helper.
    // -----------------------------------------------------------------------

    let has_permission = tournament_user.has_permission(permission);
    Ok(Json(has_permission).into_response())

}












    
    
  
