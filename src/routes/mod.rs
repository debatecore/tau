use axum::Router;

pub mod health;

pub fn routes() -> Router {
    Router::new().merge(health::route())
}
