use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup, tournaments::roles::Role};
use uuid::Uuid;

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin,
    create_app, create_listener, prepare_empty_database,
    roles_utils::create_roles,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{
        check_permission, get_id_of_a_new_judge, get_id_of_a_new_user, get_judge_token,
        get_organizer_token,
    },
};

#[tokio::test]
#[serial]
async fn user_with_organizer_role_can_check_permission_they_have() -> Result<(), OmniError>
{
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let user_id = get_id_of_a_new_user(&Uuid::now_v7().to_string(), "password").await;
    let admin_token = get_session_token_for_infrastructure_admin().await;

    // Assign organizer role to user
    let role_response = create_roles(
        &user_id,
        &tournament_id,
        vec![Role::Organizer],
        &admin_token,
    )
    .await;
    assert_eq!(role_response.status(), StatusCode::OK);

    let organizer_token = get_organizer_token(&tournament_id).await;

    // WHEN
    let response = check_permission(
        &user_id,
        &tournament_id,
        "WriteTournament",
        &organizer_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert_eq!(body, "true");

    Ok(())
}

#[tokio::test]
#[serial]
async fn user_can_verify_lack_of_permission() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let judge_token = get_judge_token(&tournament_id).await;

    // WHEN
    let response =
        check_permission(&judge_id, &tournament_id, "WriteTournament", &judge_token)
            .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert_eq!(body, "false");

    Ok(())
}

#[tokio::test]
#[serial]
async fn infrastructure_admin_has_all_permissions() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let admin_token = get_session_token_for_infrastructure_admin().await;

    let admin_id = Uuid::max().to_string();

    // WHEN - Try various permissions
    let response1 =
        check_permission(&admin_id, &tournament_id, "WriteTournament", &admin_token)
            .await;

    let response2 =
        check_permission(&admin_id, &tournament_id, "WriteTeams", &admin_token).await;

    // THEN
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response1.text().await.unwrap(), "true");

    assert_eq!(response2.status(), StatusCode::OK);
    assert_eq!(response2.text().await.unwrap(), "true");

    Ok(())
}

#[tokio::test]
#[serial]
async fn invalid_permission_name_returns_404() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let user_id = get_id_of_a_new_user(&Uuid::now_v7().to_string(), "password").await;
    let _admin_token = get_session_token_for_infrastructure_admin().await;
    let organizer_token = get_organizer_token(&tournament_id).await;

    // WHEN
    let response = check_permission(
        &user_id,
        &tournament_id,
        "InvalidPermission",
        &organizer_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
#[serial]
async fn multiple_permission_names_returns_400() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let user_id = get_id_of_a_new_user(&Uuid::now_v7().to_string(), "password").await;
    let organizer_token = get_organizer_token(&tournament_id).await;

    // WHEN
    let socket_address = tau::setup::get_socket_addr().to_string().replace("0.0.0.0", "127.0.0.1");
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/users/{}/tournaments/{}/permissions?permission_name=WriteTeams&permission_name=ReadTeams",
            socket_address, user_id, tournament_id
        ))
        .bearer_auth(&organizer_token)
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
#[serial]
async fn user_not_assigned_to_tournament_returns_401() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_alpha_id = get_id_of_a_new_tournament("tournament alpha").await?;
    let tournament_beta_id = get_id_of_a_new_tournament("tournament beta").await?;
    let user_name = Uuid::now_v7().to_string();

    let user_id = get_id_of_a_new_user(&user_name, "password").await;

    // Assign user to tournament alpha only
    let admin_token = get_session_token_for_infrastructure_admin().await;
    create_roles(
        &user_id,
        &tournament_alpha_id,
        vec![Role::Organizer],
        &admin_token,
    )
    .await;

    let user_token =
        crate::common::auth_utils::get_session_token_for(&user_name, "password")
            .await
            .unwrap();

    // WHEN - Try to check permission in tournament they're not assigned to
    let response = check_permission(
        &user_id,
        &tournament_beta_id,
        "WriteTournament",
        &user_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}
use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup, tournaments::roles::Role};
use uuid::Uuid;

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin,
    create_app, create_listener, prepare_empty_database,
    roles_utils::create_roles,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{
        check_permission, get_id_of_a_new_judge, get_id_of_a_new_user, get_judge_token,
        get_organizer_token,
    },
};

#[tokio::test]
#[serial]
async fn user_with_organizer_role_can_check_permission_they_have() -> Result<(), OmniError>
{
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let user_id = get_id_of_a_new_user(&Uuid::now_v7().to_string(), "password").await;
    let admin_token = get_session_token_for_infrastructure_admin().await;

    // Assign organizer role to user
    let role_response = create_roles(
        &user_id,
        &tournament_id,
        vec![Role::Organizer],
        &admin_token,
    )
    .await;
    assert_eq!(role_response.status(), StatusCode::OK);

    let organizer_token = get_organizer_token(&tournament_id).await;

    // WHEN
    let response = check_permission(
        &user_id,
        &tournament_id,
        "WriteTournament",
        &organizer_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert_eq!(body, "true");

    Ok(())
}

#[tokio::test]
#[serial]
async fn user_can_verify_lack_of_permission() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let judge_token = get_judge_token(&tournament_id).await;

    // WHEN
    let response =
        check_permission(&judge_id, &tournament_id, "WriteTournament", &judge_token)
            .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert_eq!(body, "false");

    Ok(())
}

#[tokio::test]
#[serial]
async fn infrastructure_admin_has_all_permissions() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let admin_token = get_session_token_for_infrastructure_admin().await;

    let admin_id = Uuid::max().to_string();

    // WHEN - Try various permissions
    let response1 =
        check_permission(&admin_id, &tournament_id, "WriteTournament", &admin_token)
            .await;

    let response2 =
        check_permission(&admin_id, &tournament_id, "WriteTeams", &admin_token).await;

    // THEN
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response1.text().await.unwrap(), "true");

    assert_eq!(response2.status(), StatusCode::OK);
    assert_eq!(response2.text().await.unwrap(), "true");

    Ok(())
}

#[tokio::test]
#[serial]
async fn invalid_permission_name_returns_404() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let user_id = get_id_of_a_new_user(&Uuid::now_v7().to_string(), "password").await;
    let _admin_token = get_session_token_for_infrastructure_admin().await;
    let organizer_token = get_organizer_token(&tournament_id).await;

    // WHEN
    let response = check_permission(
        &user_id,
        &tournament_id,
        "InvalidPermission",
        &organizer_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
#[serial]
async fn multiple_permission_names_returns_400() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test tournament").await?;
    let user_id = get_id_of_a_new_user(&Uuid::now_v7().to_string(), "password").await;
    let organizer_token = get_organizer_token(&tournament_id).await;

    // WHEN
    let socket_address = tau::setup::get_local_socket_addr();
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/users/{}/tournaments/{}/permissions?permission_name=WriteTeams&permission_name=ReadTeams",
            socket_address, user_id, tournament_id
        ))
        .bearer_auth(&organizer_token)
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
#[serial]
async fn user_not_assigned_to_tournament_returns_401() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_alpha_id = get_id_of_a_new_tournament("tournament alpha").await?;
    let tournament_beta_id = get_id_of_a_new_tournament("tournament beta").await?;
    let user_name = Uuid::now_v7().to_string();

    let user_id = get_id_of_a_new_user(&user_name, "password").await;

    // Assign user to tournament alpha only
    let admin_token = get_session_token_for_infrastructure_admin().await;
    create_roles(
        &user_id,
        &tournament_alpha_id,
        vec![Role::Organizer],
        &admin_token,
    )
    .await;

    let user_token =
        crate::common::auth_utils::get_session_token_for(&user_name, "password")
            .await
            .unwrap();

    // WHEN - Try to check permission in tournament they're not assigned to
    let response = check_permission(
        &user_id,
        &tournament_beta_id,
        "WriteTournament",
        &user_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}
