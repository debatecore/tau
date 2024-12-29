use axum::Router;

use crate::setup::AppState;

mod health_check;
mod infradmin;
mod swagger;
mod teapot;
mod version;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(health_check::route())
        .merge(swagger::route())
        .merge(teapot::route())
        .merge(version::route())
        .merge(infradmin::route())
}
