use axum::Router;

mod health_check;
mod swagger;
mod teapot;
mod version;

pub fn routes() -> Router {
    Router::new()
        .merge(health_check::route())
        .merge(swagger::route())
        .merge(teapot::route())
        .merge(version::route())
}
