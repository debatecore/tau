use std::{future::IntoFuture, vec};

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup, tournaments::roles::Role};

use crate::common::{
    affiliations_utils::{
        create_affiliation, delete_affiliation, get_affiliation, get_all_affiliations,
        get_id_of_a_new_affiliation, patch_affiliation,
    },
    create_app, create_listener, get_response_json, prepare_empty_database,
    teams_utils::get_id_of_a_new_team,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{
        get_id_of_a_new_judge, get_organizer_token, get_token_for_user_with_roles,
    },
};

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_create_affiliations() -> Result<(), OmniError> {
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
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;

    // WHEN
    let response = create_affiliation(&judge_id, &team_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = get_response_json(response).await?;
    assert_eq!(response_body["judge_user_id"], judge_id.to_owned());
    assert_eq!(response_body["team_id"], team_id);
    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_get_affiliations() -> Result<(), OmniError> {
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
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;

    let affiliation_id = get_id_of_a_new_affiliation(&judge_id, &team_id).await?;

    // WHEN
    let response = get_affiliation(&affiliation_id, &judge_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_list_affiliations() -> Result<(), OmniError> {
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
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;
    let team_id2 = get_id_of_a_new_team(&tournament_id, "aff2").await;

    create_affiliation(&judge_id, &team_id, &token).await;
    create_affiliation(&judge_id, &team_id2, &token).await;

    // WHEN
    let response = get_all_affiliations(&judge_id, &tournament_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let affiliations = response.json::<Vec<serde_json::Value>>().await.unwrap();
    assert_eq!(affiliations.len(), 2);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_patch_affiliations() -> Result<(), OmniError> {
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
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;

    let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;
    let new_team_id = get_id_of_a_new_team(&tournament_id, "aff2").await;

    let affiliation_id = get_id_of_a_new_affiliation(&judge_id, &team_id).await?;

    // WHEN
    let response =
        patch_affiliation(&affiliation_id, &judge_id, &new_team_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_delete_affiliations() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;

    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("some").await?;
    let token = get_organizer_token(&tournament_id).await;
    let team_id = get_id_of_a_new_team(&tournament_id, "team").await;
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let affiliation_id = get_id_of_a_new_affiliation(&judge_id, &team_id).await?;

    // WHEN
    let response = delete_affiliation(&affiliation_id, &judge_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    Ok(())
}

#[tokio::test]
#[serial]
async fn affiliations_should_not_be_visible_to_judges_and_marshals(
) -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;

    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("some").await?;
    let team_id = get_id_of_a_new_team(&tournament_id, "team").await;
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let affiliation_id = get_id_of_a_new_affiliation(&judge_id, &team_id).await?;

    // WHEN
    let roles_to_check = vec![Role::Judge, Role::Marshal];
    for role in roles_to_check {
        let token = get_token_for_user_with_roles(vec![role], &tournament_id).await;
        let response = get_affiliation(&affiliation_id, &judge_id, &token).await;

        // THEN
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    Ok(())
}
