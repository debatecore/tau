use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::setup::{self, get_socket_addr};

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin, create_app, create_listener,
    prepare_empty_database, teams_utils::create_team,
    tournament_utils::create_tournament, user_utils::get_organizer_token,
};

mod common;

#[tokio::test]
#[serial]
async fn admin_should_be_able_to_create_teams() {
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

    let token = get_session_token_for_infrastructure_admin().await.unwrap();
    let full_name = "Team A";
    let shortened_name = "A";

    // WHEN
    let tournament_id = create_tournament("Wrocławska Liga Debat", "WrLD", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let response = create_team(&tournament_id, full_name, shortened_name, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(response_body["full_name"], full_name);
    assert_eq!(response_body["shortened_name"], shortened_name);
    assert_eq!(response_body["tournament_id"], tournament_id);
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_create_teams() {
    todo!()
}

#[tokio::test]
#[serial]
async fn teams_should_be_patchable() {
    todo!()
}

#[tokio::test]
#[serial]
async fn teams_should_be_deletable() {
    todo!()
}
