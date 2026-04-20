use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup};

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin,
    create_app, create_listener, prepare_empty_database,
    teams_utils::{create_team, delete_team, get_id_of_a_new_team, get_team, patch_team},
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{
        get_judge_token, get_organizer_token, get_token_for_user_with_no_roles,
    },
};

#[tokio::test]
#[serial]
async fn admin_should_be_able_to_create_teams() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let token = get_session_token_for_infrastructure_admin().await;
    let full_name = "Team A";
    let shortened_name = "A";

    // WHEN
    let tournament_id = get_id_of_a_new_tournament("T1").await?;
    let response = create_team(&tournament_id, full_name, shortened_name, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(response_body["full_name"], full_name);
    assert_eq!(response_body["shortened_name"], shortened_name);
    assert_eq!(response_body["tournament_id"], tournament_id);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_create_teams() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let full_name = "Team A";
    let shortened_name = "A";

    // WHEN
    let tournament_id = get_id_of_a_new_tournament("T1").await?;
    let token = get_organizer_token(&tournament_id).await;
    let response = create_team(&tournament_id, full_name, shortened_name, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(response_body["full_name"], full_name);
    assert_eq!(response_body["shortened_name"], shortened_name);
    assert_eq!(response_body["tournament_id"], tournament_id);

    Ok(())
}

#[tokio::test]
#[serial]
async fn teams_should_be_patchable() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let full_name = "Team A";
    let new_full_name = "Team B";
    let new_shortened_name = "B";

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let tournament_id = get_id_of_a_new_tournament("T1").await?;
    let id = get_id_of_a_new_team(&tournament_id, full_name).await;
    let response = patch_team(
        &id,
        &tournament_id,
        new_full_name,
        new_shortened_name,
        &token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(response_body["full_name"], new_full_name);
    assert_eq!(response_body["shortened_name"], new_shortened_name);
    assert_eq!(response_body["tournament_id"], tournament_id);

    Ok(())
}

#[tokio::test]
#[serial]
async fn team_names_should_be_enforced_as_unique_within_a_tournament(
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

    let full_name = "Team A";
    let new_shortened_name = "B";

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let tournament_id = get_id_of_a_new_tournament("T1").await?;
    get_id_of_a_new_team(&tournament_id, full_name).await;
    let response =
        create_team(&tournament_id, full_name, new_shortened_name, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::CONFLICT);

    Ok(())
}

#[tokio::test]
#[serial]
async fn duplicate_team_names_should_be_allowed_in_different_tournaments(
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

    let full_name = "Team A";
    let shortened_name = "A";

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let tournament_id1 = get_id_of_a_new_tournament("T1").await?;
    let tournament_id2 = get_id_of_a_new_tournament("T2").await?;
    create_team(&tournament_id1, full_name, shortened_name, &token).await;
    let response = create_team(&tournament_id2, full_name, shortened_name, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[serial]
async fn teams_should_be_visible_for_users_within_tournament() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let full_name = "Team A";

    // WHEN
    let tournament_id = get_id_of_a_new_tournament("T1").await?;
    let id = get_id_of_a_new_team(&tournament_id, full_name).await;
    let judge_token = get_judge_token(&tournament_id).await;
    let response = get_team(&id, &tournament_id, &judge_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(response_body["full_name"], full_name);
    assert_eq!(response_body["tournament_id"], tournament_id);

    Ok(())
}

#[tokio::test]
#[serial]
async fn teams_should_not_be_visible_for_users_outside_of_tournament(
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

    let full_name = "Team A";

    // WHEN
    let tournament_id = get_id_of_a_new_tournament("T1").await?;
    let id = get_id_of_a_new_team(&tournament_id, full_name).await;
    let mallory_token = get_token_for_user_with_no_roles().await;
    let response = get_team(&id, &tournament_id, &mallory_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
#[serial]
async fn teams_should_be_deletable() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let full_name = "Team A";

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let tournament_id = get_id_of_a_new_tournament("T1").await?;
    let id = get_id_of_a_new_team(&tournament_id, full_name).await;
    let response = delete_team(&id, &tournament_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    Ok(())
}
