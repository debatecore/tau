use reqwest::{Client, Response, StatusCode};
use std::future::IntoFuture;
use tau::setup::get_socket_addr;
use serial_test::serial;
use tau::{omni_error::OmniError, setup};
use crate::common::{
    create_app, create_listener, prepare_empty_database,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{get_organizer_token, get_token_for_user_with_roles},
};
use uuid::Uuid;
use serde_json::json;

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

    let plan_data = json!({
        "tournament_id": tournament_id,
        "group_phase_rounds": 4,
        "groups_count": 8,
        "advancing_teams": 4,
        "total_teams": 32,
    });

    // WHEN
    let response = Client::new()
        .post(format!(
            "http://{}/tournaments/{}/plan",
            get_socket_addr(), tournament_id
        ))
        .json(&plan_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    // Store the result in a variable to avoid temporary value issues
    let response_body = response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();
    assert_eq!(plan_id, response_body["id"].as_str().unwrap());

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

    let plan_data = json!({
        "tournament_id": tournament_id,
        "group_phase_rounds": 4,
        "groups_count": 8,
        "advancing_teams": 4,
        "total_teams": 32,
    });

    let create_response = Client::new()
        .post(format!(
            "http://{}/tournaments/{}/plan",
            get_socket_addr(), tournament_id
        ))
        .json(&plan_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();
    
    assert_eq!(create_response.status(), StatusCode::OK);

    // Store the result in a variable to avoid temporary value issues
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

    let plan_data = json!({
        "tournament_id": tournament_id,
        "group_phase_rounds": 4,
        "groups_count": 8,
        "advancing_teams": 4,
        "total_teams": 32,
    });

    let create_response = Client::new()
        .post(format!(
            "http://{}/tournaments/{}/plan",
            get_socket_addr(), tournament_id
        ))
        .json(&plan_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();
    
    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    let patch_data = json!({
        "group_phase_rounds": 5,
        "groups_count": 8,
        "advancing_teams": 4,
        "total_teams": 32,
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

    let plan_data = json!({
        "tournament_id": tournament_id,
        "group_phase_rounds": 4,
        "groups_count": 8,
        "advancing_teams": 4,
        "total_teams": 32,
    });

    let create_response = Client::new()
        .post(format!(
            "http://{}/tournaments/{}/plan",
            get_socket_addr(), tournament_id
        ))
        .json(&plan_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();
    
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