use crate::{
    setup::AppState,
    users::{auth::session::Session, User},
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tower_cookies::Cookies;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/infradmin/all-users", get(allusers))
        .route("/infradmin/all-sessions", get(allsessions))
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

async fn allsessions(
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
    match Session::get_all(&state.connection_pool).await {
        Ok(sessions) => Json(sessions).into_response(),
        Err(e) => e.respond(),
    }
}
