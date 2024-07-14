use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::env;

pub fn route() -> Router {
    Router::new()
        .route("/version", get(version))
        .route("/version-details", get(version_details))
}

#[derive(Serialize)]
struct VersionDetails {
    version: &'static str,
    version_bits: VersionBits,
    git_commit_hash: &'static str,
    current_endpoint_prefix: &'static str,
    repository: &'static str,
}

#[derive(Serialize)]
struct VersionBits {
    major: &'static str,
    minor: &'static str,
    patch: &'static str,
}

async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

async fn version_details() -> Json<VersionDetails> {
    Json(VersionDetails {
        version: env!("CARGO_PKG_VERSION"),
        version_bits: VersionBits {
            major: env!("CARGO_PKG_VERSION_MAJOR"),
            minor: env!("CARGO_PKG_VERSION_MINOR"),
            patch: env!("CARGO_PKG_VERSION_PATCH"),
        },
        git_commit_hash: env!("GIT_COMMIT_HASH"),
        current_endpoint_prefix: env!("CARGO_PKG_VERSION_MAJOR"),
        repository: env!("CARGO_PKG_REPOSITORY"),
    })
}
