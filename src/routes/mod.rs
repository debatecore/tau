use axum::Router;

mod health;
mod version;

pub fn routes() -> Router {
    Router::new().merge(health::route()).merge(version::route())
}
