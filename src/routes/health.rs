use axum::{http::StatusCode, routing::get, Router};

static HEALTH_CHECK_TAG: &str = "health check";

/// Used to determine whether the API is online
#[utoipa::path(get, path = "/live", tag = HEALTH_CHECK_TAG, responses((status = StatusCode::OK, description = "OK")))]
pub fn route() -> Router {
    Router::new()
        .route("/live", get(empty()))
        .route("/health", get(empty()))
}

fn empty() -> StatusCode {
    StatusCode::OK
}
