use axum::{routing::IntoMakeService, Router};
use reqwest::Response;
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
pub mod affiliations_utils;
pub mod auth_utils;
pub mod debates_utils;
pub mod phases_utils;
pub mod plans_utils;
pub mod roles_utils;
pub mod rounds_utils;
pub mod teams_utils;
pub mod tournament_utils;
pub mod user_utils;
pub mod verdicts_utils;
pub mod test_app;

use tau::{
    database,
    omni_error::OmniError,
    routes,
    setup::{self, AppState},
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

pub async fn create_test_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind random test port")
}

/// TO-DO: refactor existing tests to use this function
pub async fn get_response_json(
    response: Response,
) -> Result<serde_json::Value, OmniError> {
    let status = response.status();
    let text = match response.text().await {
        Ok(text) => text,
        Err(e) => format!("{}", e).to_string(),
    };

    serde_json::from_str::<serde_json::Value>(&text).map_err(|e| {
        OmniError::ExplicitError {
            status,
            message: format!(
                "Failed to parse response body\nError message: {}\nResponse text: {}",
                e, text
            ),
        }
    })
}
