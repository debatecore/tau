use axum::{routing::IntoMakeService, Router};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use crate::{
    database, routes,
    setup::{self, AppState},
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
