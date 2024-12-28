use axum::Router;

use crate::setup::AppState;

mod attendee;
mod health_check;
mod motion;
mod swagger;
mod team;
mod teapot;
mod tournament;
mod version;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(health_check::route())
        .merge(swagger::route())
        .merge(teapot::route())
        .merge(version::route())
        .merge(tournament::route())
        .merge(team::route())
        .merge(attendee::route())
        .merge(motion::route())
}
