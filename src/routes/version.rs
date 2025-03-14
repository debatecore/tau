use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::env;
use utoipa::ToSchema;

use crate::setup::AppState;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/version", get(version))
        .route("/version-details", get(version_details))
}

#[derive(Serialize, ToSchema)]
pub struct VersionDetails {
    version: &'static str,
    version_bits: VersionBits,
    git_info: GitInfo,
    current_endpoint_prefix: &'static str,
    repository: &'static str,
}

#[derive(Serialize, ToSchema)]
pub struct VersionBits {
    major: &'static str,
    minor: &'static str,
    patch: &'static str,
}

#[derive(Serialize, ToSchema)]
pub struct GitInfo {
    /// Latest git commit hash, or 'UNKNOWN!'
    git_commit_hash: &'static str,
    /// This is either 'clean', 'changed' or 'UNKNOWN!'
    git_status_porcelain: &'static str,
}

/// Returns API version
#[utoipa::path(
    get,
    path = "/version",
    responses((
        status = StatusCode::OK,
        description = "Returns API semver version.",
        body = str,
    ))
)]
async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns API version & other diagnostic data
#[utoipa::path(
    get,
    path = "/version-details",
    responses((
        status = StatusCode::OK,
        description = "Returns diagnostic info about the API.",
        body = VersionDetails
    ))
)]
async fn version_details() -> Json<VersionDetails> {
    Json(VersionDetails {
        version: env!("CARGO_PKG_VERSION"),
        version_bits: VersionBits {
            major: env!("CARGO_PKG_VERSION_MAJOR"),
            minor: env!("CARGO_PKG_VERSION_MINOR"),
            patch: env!("CARGO_PKG_VERSION_PATCH"),
        },
        git_info: GitInfo {
            git_commit_hash: env!("GIT_COMMIT_HASH"),
            git_status_porcelain: env!("GIT_STATUS_PORCELAIN"),
        },
        current_endpoint_prefix: env!("CARGO_PKG_VERSION_MAJOR"),
        repository: env!("CARGO_PKG_REPOSITORY"),
    })
}
