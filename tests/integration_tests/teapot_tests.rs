use std::future::IntoFuture;

use reqwest::{Client, StatusCode};
use serial_test::serial;
use tau::setup::{self, get_local_socket_addr};

use crate::common::{create_app, create_listener, prepare_empty_database};

#[tokio::test]
#[serial]
async fn test_teapot() {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let socket_address = get_local_socket_addr().to_string();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
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
