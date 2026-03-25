use axum::{
    Json, Router, extract::{Path, Query, RawQuery, State}, http::{HeaderMap, StatusCode, Uri}, response::{IntoResponse, Response}, routing::get
};

use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;
use uuid::Uuid;


use crate::{
    omni_error::OmniError, setup::AppState, users::{TournamentUser, permissions::Permission}
};

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/users/:id/tournaments/:tournament_id/permissions", get(has_permission))
        
}

/// Create a new affiliation
///
/// Available only to Organizers and the infrastructure admin.
#[utoipa::path(get, path = "/users/{id}/tournaments/{tournament_id}/permissions",
    responses(
        (status=200, description = "Ok", body=bool),
        (status=400, description = "Bad request"),
        (status=401, description = "Unauthorized"),
        (status=404, description = "Resource not found"),
        (status=500, description = "Internal server error"),
    ),
    tag="users"

)]
async fn has_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
    uri: Uri,
    Path((user_id, tournament_id)): Path<(Uuid, Uuid)>,
) -> Result <Response, OmniError>{
    let pool = &state.connection_pool;
    let tournament_user =
    TournamentUser::authenticate(tournament_id, &headers, cookies, &pool).await?;

    let result: Query<Params> = Query::try_from_uri(&uri).unwrap();

    match tournament_user.has_permission(result.permission_name){
        true => Ok((StatusCode::OK, "true").into_response()),
        false => Ok((StatusCode::OK, "false").into_response()),
    }

}

#[derive(Deserialize)]
struct Params{
    permission_name: Permission
}



// ---------------------------------------------------------------------------
// Query-string shape
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PermissionQuery{
    permission_name: String, 
}














    
    
  
