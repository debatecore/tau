use axum::Router;

pub mod health;
pub mod swagger;
pub mod teapot;

pub fn routes() -> Router {
    Router::new()
        .merge(health::route())
        .merge(teapot::route())
        .merge(swagger::route())
}
