use reqwest::{Client, Response, StatusCode};
use std::future::IntoFuture;
use tau::setup::get_socket_addr;
use serial_test::serial;
use serde_json::json;
use tau::{omni_error::OmniError, setup};
use crate::common::{
    create_app, create_listener, prepare_empty_database,
    tournament_utils::get_id_of_a_new_tournament,
    plans_utils::create_plan,
    user_utils::{
        get_organizer_token, 
        get_token_for_user_with_roles, 
        get_token_for_user_with_no_roles
    },
};

const TEST_GROUP_PHASE_ROUNDS: i32 = 4;
const TEST_GROUPS_COUNT:       i32 = 8;
const TEST_ADVANCING_TEAMS:    i32 = 4;
const TEST_TOTAL_TEAMS:        i32 = 32;

#[tokio::test]
#[serial]
async fn tournament_plan_creation_should_impossible_for_other_users() -> Result<(), OmniError>  {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let token = get_token_for_user_with_no_roles().await;
    let tournament_id = get_id_of_a_new_tournament("test").await?;

    // WHEN
    assert_eq!(
        create_plan(
            &tournament_id, 
            TEST_GROUP_PHASE_ROUNDS, 
            TEST_GROUPS_COUNT, 
            TEST_ADVANCING_TEAMS, 
            TEST_TOTAL_TEAMS,
            &token
        )
        .await
        .status(), 
        StatusCode::UNAUTHORIZED
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_create_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;

    assert_eq!(
        create_plan(
            &tournament_id, 
            TEST_GROUP_PHASE_ROUNDS, 
            TEST_GROUPS_COUNT, 
            TEST_ADVANCING_TEAMS, 
            TEST_TOTAL_TEAMS,
            &token
        )
        .await
        .status(), 
        StatusCode::OK
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_get_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;

    let create_response = create_plan(
        &tournament_id, 
        TEST_GROUP_PHASE_ROUNDS, 
        TEST_GROUPS_COUNT, 
        TEST_ADVANCING_TEAMS, 
        TEST_TOTAL_TEAMS,
        &token
    ).await;
    
    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    // WHEN
    let response = Client::new()
        .get(format!(
            "http://{}/tournaments/{}/plan/{}",
            get_socket_addr(), tournament_id, plan_id
        ))
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_patch_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;

    let create_response = create_plan(
        &tournament_id, 
        TEST_GROUP_PHASE_ROUNDS, 
        TEST_GROUPS_COUNT, 
        TEST_ADVANCING_TEAMS, 
        TEST_TOTAL_TEAMS,
        &token
    ).await;
    
    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    // Data should be valid
    let patch_data = json!({
        "group_phase_rounds": 5,
        "groups_count": 10,
        "advancing_teams": 4,
        "total_teams": 30,
    });

    // WHEN
    let response = Client::new()
        .patch(format!(
            "http://{}/tournaments/{}/plan/{}",
            get_socket_addr(), tournament_id, plan_id
        ))
        .json(&patch_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_delete_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;

    let create_response = create_plan(
        &tournament_id, 
        TEST_GROUP_PHASE_ROUNDS, 
        TEST_GROUPS_COUNT, 
        TEST_ADVANCING_TEAMS, 
        TEST_TOTAL_TEAMS,
        &token
    ).await;
    
    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    // WHEN
    let response = Client::new()
        .delete(format!(
            "http://{}/tournaments/{}/plan/{}",
            get_socket_addr(), tournament_id, plan_id
        ))
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    Ok(())
}