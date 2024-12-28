use axum::Router;

use crate::setup::AppState;

mod health_check;
mod swagger;
mod teapot;
mod users;
mod version;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(health_check::route())
        .merge(swagger::route())
        .merge(teapot::route())
        .merge(version::route())
        .merge(users::route())
}
