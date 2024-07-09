use axum::{http::StatusCode, routing::get, Router};

pub fn route() -> Router {
    Router::new()
        .route("/live", get(empty()))
        .route("/health", get(empty()))
}

fn empty() -> StatusCode {
    StatusCode::OK
}
