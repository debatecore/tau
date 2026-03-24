use axum::{
    extract::{Path, RawQuery, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use std::str::FromStr;
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::permissions::Permission;
use crate::TournamentUser; // Adjust path as necessary based on your crate structure

#[utoipa::path(
    get,
    path = "/user/{id}/tournaments/{tournament_id}/permissions",
    params(
        ("id" = Uuid, Path, description = "The ID of the user"),
        ("tournament_id" = Uuid, Path, description = "The ID of the tournament"),
        ("permission_name" = String, Query, description = "The exact name of the permission to check (e.g., 'WriteTeams')")
    ),
    responses(
        (status = 200, description = "Boolean indicating if the user has the permission", body = bool),
        (status = 400, description = "Multiple permissions provided in the query string"),
        (status = 401, description = "User is not authenticated or not assigned to the tournament"),
        (status = 404, description = "The requested permission does not exist")
    )
)]
pub async fn check_permission_endpoint(
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    cookies: Cookies,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<bool>, (StatusCode, String)> {
    
    // 1. Validate Query Parameters (Requirement: 400 on multiple permissions)
    let query_str = raw_query.unwrap_or_default();
    let permission_params: Vec<&str> = query_str
        .split('&')
        .filter(|param| param.starts_with("permission_name="))
        .collect();

    if permission_params.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Missing 'permission_name' query parameter".to_string()));
    }
    if permission_params.len() > 1 {
        return Err((StatusCode::BAD_REQUEST, "Cannot check multiple permissions at once".to_string()));
    }

    // Extract the actual string value
    let permission_str = permission_params[0].trim_start_matches("permission_name=");

    // 2. Parse Permission (Requirement: 404 on nonexistent permissions)
    let permission = Permission::from_str(permission_str).map_err(|_| {
        (StatusCode::NOT_FOUND, format!("Permission '{}' does not exist", permission_str))
    })?;

    // 3. Authenticate and Validate Assignment (Requirement: 401 on unassigned/unauthorized)
    // Note: Assuming `TournamentUser::authenticate` returns your `OmniError` if unassigned.
    let tournament_user = TournamentUser::authenticate(tournament_id, &headers, cookies, &pool)
        .await
        .map_err(|_| {
            (StatusCode::UNAUTHORIZED, "User is not assigned to this tournament or unauthorized".to_string())
        })?;

    // Ensure the user ID in the path matches the authenticated token's user ID
    if tournament_user.user.id != user_id {
        return Err((StatusCode::UNAUTHORIZED, "Token does not match requested user ID".to_string()));
    }

    // 4. Check Permission (Requirement: 200 OK with boolean)
    let has_perm = tournament_user.has_permission(permission);
    Ok(Json(has_perm))
}
