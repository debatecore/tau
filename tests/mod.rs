use std::future::IntoFuture;

use axum::{routing::IntoMakeService, Router};
use reqwest::{Client, StatusCode};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use tau::{
    database, routes,
    setup::{self, get_socket_addr},
    users::infradmin::guarantee_infrastructure_admin_exists,
};

pub async fn create_app() -> IntoMakeService<Router> {
    setup::read_environmental_variables();
    setup::check_secret_env_var();

    let state = setup::create_app_state().await;
    database::perform_migrations(&state.connection_pool).await;
    guarantee_infrastructure_admin_exists(&state.connection_pool).await;

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

#[tokio::test]
async fn test_teapot() {
    // GIVEN
    let socket_address = get_socket_addr().to_string();
    let app = create_app().await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    // WHEN
    let client = Client::new();
    let res = client
        .get(format!("http://{}/brew-coffee", socket_address))
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(res.status(), StatusCode::IM_A_TEAPOT);
}
