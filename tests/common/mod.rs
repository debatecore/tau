use std::future::IntoFuture;

use axum::{routing::IntoMakeService, Router};
use reqwest::{Client, StatusCode};
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use tau::{
    database, routes,
    setup::{self, get_socket_addr, AppState},
    users::infradmin::guarantee_infrastructure_admin_exists,
};

pub async fn prepare_empty_database(pool: &Pool<Postgres>) {
    database::clear_database(pool).await;
    database::perform_migrations(pool).await;
    guarantee_infrastructure_admin_exists(pool).await;
}

pub async fn create_app(state: AppState) -> IntoMakeService<Router> {
    setup::read_environmental_variables();
    setup::check_secret_env_var();

    Router::new()
        .merge(routes::routes())
        .with_state(state)
        .layer(setup::configure_cors())
        .layer(CookieManagerLayer::new())
        .into_make_service()
}

pub async fn create_listener() -> TcpListener {
    let addr = setup::get_socket_addr();
    TcpListener::bind(addr).await.unwrap()
}
