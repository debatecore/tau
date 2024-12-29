use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tower_cookies::Cookies;

use crate::{setup::AppState, users::User};

pub fn route() -> Router<AppState> {
    Router::new().route("/infradmin/all-users", get(allusers))
}

async fn allusers(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Response {
    match User::authenticate(&headers, cookies, &state.connection_pool).await {
        Ok(u) => match u.is_infrastructure_admin() {
            true => (),
            false => return StatusCode::FORBIDDEN.into_response(),
        },
        Err(e) => return e.respond(),
    };
    match User::get_all(&state.connection_pool).await {
        Ok(users) => Json(users).into_response(),
        Err(e) => e.respond(),
    }
}
