use std::{collections::HashMap, future::IntoFuture};

use reqwest::{Client, StatusCode};
use serial_test::serial;
use tau::setup::{self, get_socket_addr};

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin, create_app, create_listener,
    prepare_empty_database, tournament_utils::create_tournament,
    user_utils::get_token_for_user_with_no_roles,
};

#[tokio::test]
#[serial]
async fn tournament_creation_should_require_login() {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);
    let socket_address = get_socket_addr();

    let mut request_body = HashMap::new();
    request_body.insert("full_name", "Wrocławska Liga Debat");
    request_body.insert("shortened_name", "WrLD");

    // WHEN
    let client = Client::new();
    let res = client
        .post(format!("http://{}/tournament", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn tournament_creation_should_be_possible_for_infrastructure_admin() {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let res = create_tournament("Wrocławska Liga Debat", "WrLD", &token).await;

    // THEN
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn tournament_creation_should_impossible_for_other_users() {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);
    let user_token = get_token_for_user_with_no_roles().await;

    // WHEN
    let response =
        create_tournament("illegal tournament", "will not be created", &user_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn tournament_names_should_not_allow_duplicates() {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let full_name = "Wrocławska Liga Debat";
    let shortened_name = "WrLD";
    let token = get_session_token_for_infrastructure_admin().await;

    // WHEN
    let first_response = create_tournament(full_name, shortened_name, &token).await;
    let second_response = create_tournament(full_name, shortened_name, &token).await;

    // THEN
    assert_eq!(first_response.status(), StatusCode::OK);
    assert_eq!(second_response.status(), StatusCode::CONFLICT);
}
