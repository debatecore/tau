use reqwest::{Client, StatusCode};
use std::{collections::HashMap, future::IntoFuture};
use tau::setup::{self, get_socket_addr};

use crate::common::{create_app, create_listener, prepare_empty_database};
mod common;

#[tokio::test]
async fn login_as_infraadmin_should_work_out_of_the_box() {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let socket_address = get_socket_addr().to_string();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let mut request_body = HashMap::new();
    request_body.insert("login", "admin");
    request_body.insert("password", "admin");

    // WHEN
    let client = Client::new();
    let res = client
        .post(format!("http://{}/auth/login", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .send()
        .await;
    // THEN
    assert_eq!(res.unwrap().status(), StatusCode::OK);
}
