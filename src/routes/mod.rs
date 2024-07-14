use axum::Router;

pub mod health_check;
pub mod swagger;
pub mod teapot;

pub fn routes() -> Router {
    Router::new()
        .merge(health_check::route())
        .merge(teapot::route())
        .merge(swagger::route())
}
