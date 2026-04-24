use std::{collections::HashMap, future::IntoFuture};

use reqwest::{Client, StatusCode};
use serial_test::serial;
use tau::setup::{self, get_local_socket_addr};

use crate::common::{
    test_app::TestApp,
    auth_utils::get_session_token_for_infrastructure_admin, create_app, create_listener,
    prepare_empty_database, tournament_utils::create_tournament,
    user_utils::get_token_for_user_with_no_roles,
};

#[tokio::test]
#[serial]
async fn tournament_creation_should_require_login() {
    // GIVEN
    let app = TestApp::spawn().await;

    let mut request_body = HashMap::new();
    request_body.insert("full_name", "Wrocławska Liga Debat");
    request_body.insert("shortened_name", "WrLD");

    // WHEN
    let res = app.client
        .post(app.url(&format!("/tournaments")))
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
    let app = TestApp::spawn().await;

    // WHEN
    let token = get_session_token_for_infrastructure_admin(&app).await;
    let res = create_tournament(&app, "Wrocławska Liga Debat", "WrLD", &token).await;

    // THEN
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn tournament_creation_should_impossible_for_other_users() {
    // GIVEN
    let app = TestApp::spawn().await;
    let user_token = get_token_for_user_with_no_roles(&app).await;

    // WHEN
    let response =
        create_tournament(&app, "illegal tournament", "will not be created", &user_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn tournament_names_should_not_allow_duplicates() {
    // GIVEN
    let app = TestApp::spawn().await;

    let full_name = "Wrocławska Liga Debat";
    let shortened_name = "WrLD";
    let token = get_session_token_for_infrastructure_admin(&app).await;

    // WHEN
    let first_response = create_tournament(&app, full_name, shortened_name, &token).await;
    let second_response = create_tournament(&app, full_name, shortened_name, &token).await;

    // THEN
    assert_eq!(first_response.status(), StatusCode::OK);
    assert_eq!(second_response.status(), StatusCode::CONFLICT);
}
