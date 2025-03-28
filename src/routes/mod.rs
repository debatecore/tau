use axum::Router;

use crate::setup::AppState;

mod attendee_routes;
mod auth;
mod debate_routes;
mod health_check;
mod infradmin_routes;
mod motion_routes;
mod swagger;
mod team_routes;
mod teapot;
mod tournament_routes;
mod version;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(health_check::route())
        .merge(swagger::route())
        .merge(teapot::route())
        .merge(version::route())
        .merge(infradmin_routes::route())
        .merge(auth::route())
        .merge(tournament_routes::route())
        .merge(team_routes::route())
        .merge(attendee_routes::route())
        .merge(motion_routes::route())
        .merge(debate_routes::route())
}
