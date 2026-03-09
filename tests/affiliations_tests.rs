use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::setup::{self, get_socket_addr};

use crate::common::{
    affiliations_utils::create_affiliation,
    auth_utils::get_session_token_for_infrastructure_admin,
    create_app, create_listener, prepare_empty_database,
    teams_utils::get_id_of_a_new_team,
    tournament_utils::{create_tournament, get_id_of_a_new_tournament},
    user_utils::{get_id_of_a_new_user, get_organizer_token},
};

mod common;

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_create_affiliations() {
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

    let handle = "Judge";
    let password = "Dredd";

    // WHEN
    let tournament_id = get_id_of_a_new_tournament("test").await;
    let token = get_organizer_token(&tournament_id).await;
    let judge_id = get_id_of_a_new_user(handle, password).await;
    let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;
    let response = create_affiliation(&judge_id, &team_id, &token).await;

    // THEN
    // assert_eq!(response.status(), StatusCode::OK);
    println!("{:?}", response.text().await.unwrap());
}
