use axum::{http::StatusCode, routing::get, Router};

use crate::setup::AppState;

static HEALTH_CHECK_TAG: &str = "health check";

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/live", get(live()))
        .route("/health", get(health()))
}

/// Used to determine whether the API is online
#[utoipa::path(get, path = "/health", tag = HEALTH_CHECK_TAG, responses((status = StatusCode::OK, description = "OK")))]
pub fn health() -> StatusCode {
    empty()
}

/// Used to determine whether the API is online
#[utoipa::path(get, path = "/live", tag = HEALTH_CHECK_TAG, responses((status = StatusCode::OK, description = "OK")))]
pub fn live() -> StatusCode {
    empty()
}

fn empty() -> StatusCode {
    StatusCode::OK
}
