use app_state::{AppState, AppStateTrait};
use axum::Router;
use setup::TauState;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::error;
mod database;
mod routes;
mod setup;
mod users;

#[tokio::main]
async fn main() {
    setup::initialise_logging();
    setup::read_environmental_variables();

    setup::create_app_state().await;
    let state = AppState::<TauState>::get();
    let app = Router::new()
        .with_state(state.clone())
        .merge(routes::routes())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

    let addr = setup::get_socket_addr();
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Error creating a listener: {e}");
            panic!();
        }
    };
    setup::report_listener_socket_addr(&listener);
    database::perform_migrations(state.connection_pool.clone()).await;

    match axum::serve(listener, app).await {
        Ok(..) => (),
        Err(e) => {
            error!("Error serving app on listener: {e}");
            panic!();
        }
    };
}
