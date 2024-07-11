use std::env;

use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;

pub fn route() -> Router {
    Router::new()
        .route("/version", get(version))
        .route("/version-details", get(version_details))
}

#[derive(Serialize)]
struct VersionDetails {
    version: &'static str,
    version_bits: VersionBits,
    current_endpoint_prefix: &'static str,
    repository: &'static str,
}

#[derive(Serialize)]
struct VersionBits {
    major: &'static str,
    minor: &'static str,
    patch: &'static str,
}

async fn version() -> Response {
    let v = env!("CARGO_PKG_VERSION");

    return v.into_response();
}

async fn version_details() -> Response {
    let vd = VersionDetails {
        version: env!("CARGO_PKG_VERSION"),
        version_bits: VersionBits {
            major: env!("CARGO_PKG_VERSION_MAJOR"),
            minor: env!("CARGO_PKG_VERSION_MINOR"),
            patch: env!("CARGO_PKG_VERSION_PATCH"),
        },
        current_endpoint_prefix: env!("CARGO_PKG_VERSION_MAJOR"),
        repository: env!("CARGO_PKG_REPOSITORY"),
    };

    return Json(vd).into_response();
}
