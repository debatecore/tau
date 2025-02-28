use axum::Router;

use crate::setup::AppState;

mod attendee;
mod auth;
mod debate;
mod health_check;
mod infradmin;
mod motion;
mod role;
mod swagger;
mod team;
mod teapot;
mod tournament;
mod user;
mod utils;
mod version;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(health_check::route())
        .merge(swagger::route())
        .merge(teapot::route())
        .merge(version::route())
        .merge(infradmin::route())
        .merge(auth::route())
        .merge(tournament::route())
        .merge(team::route())
        .merge(attendee::route())
        .merge(motion::route())
        .merge(debate::route())
        .merge(user::route())
        .merge(role::route())
}
