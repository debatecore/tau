use axum::Router;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{Any, CorsLayer};
use tracing::error;
use users::infradmin::guarantee_infrastructure_admin_exists;

mod database;
mod omni_error;
mod routes;
mod setup;
mod users;

#[tokio::main]
async fn main() {
    setup::initialise_logging();
    setup::read_environmental_variables();
    setup::check_secret_env_var();

    let state = setup::create_app_state().await;
    database::perform_migrations(&state.connection_pool).await;
    guarantee_infrastructure_admin_exists(&state.connection_pool).await;

    let app = Router::new()
        .merge(routes::routes())
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
        .layer(CookieManagerLayer::new());

    let addr = setup::get_socket_addr();
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Error creating a listener: {e}");
            panic!();
        }
    };
    setup::report_listener_socket_addr(&listener);

    match axum::serve(listener, app).await {
        Ok(..) => (),
        Err(e) => {
            error!("Error serving app on listener: {e}");
            panic!();
        }
    };
}
