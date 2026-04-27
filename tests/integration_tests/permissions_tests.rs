use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup, tournaments::roles::Role};
use uuid::Uuid;

use crate::common::{
    test_app::TestApp,
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
async fn user_with_organizer_role_can_check_permission_they_have() -> Result<(), OmniError>
{
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test tournament").await?;
    let user_id = get_id_of_a_new_user(&app, &Uuid::now_v7().to_string(), "password").await;
    let admin_token = get_session_token_for_infrastructure_admin(&app).await;

    // Assign organizer role to user
    let role_response = create_roles(
        &app,
        &user_id,
        &tournament_id,
        vec![Role::Organizer],
        &admin_token,
    )
    .await;
    assert_eq!(role_response.status(), StatusCode::OK);

    let organizer_token = get_organizer_token(&app, &tournament_id).await;

    // WHEN
    let response = check_permission(
        &app,
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
async fn user_can_verify_lack_of_permission() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test tournament").await?;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let judge_token = get_judge_token(&app, &tournament_id).await;

    // WHEN
    let response =
        check_permission(&app, &judge_id, &tournament_id, "WriteTournament", &judge_token)
            .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert_eq!(body, "false");

    Ok(())
}

#[tokio::test]
async fn infrastructure_admin_has_all_permissions() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test tournament").await?;
    let admin_token = get_session_token_for_infrastructure_admin(&app).await;

    let admin_id = Uuid::max().to_string();

    // WHEN - Try various permissions
    let response1 =
        check_permission(&app, &admin_id, &tournament_id, "WriteTournament", &admin_token)
            .await;

    let response2 =
        check_permission(&app, &admin_id, &tournament_id, "WriteTeams", &admin_token).await;

    // THEN
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response1.text().await.unwrap(), "true");

    assert_eq!(response2.status(), StatusCode::OK);
    assert_eq!(response2.text().await.unwrap(), "true");

    Ok(())
}

#[tokio::test]
async fn invalid_permission_name_returns_404() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test tournament").await?;
    let user_id = get_id_of_a_new_user(&app, &Uuid::now_v7().to_string(), "password").await;
    let _admin_token = get_session_token_for_infrastructure_admin(&app).await;
    let organizer_token = get_organizer_token(&app, &tournament_id).await;

    // WHEN
    let response = check_permission(
        &app,
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
async fn multiple_permission_names_returns_400() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test tournament").await?;
    let user_id = get_id_of_a_new_user(&app, &Uuid::now_v7().to_string(), "password").await;
    let organizer_token = get_organizer_token(&app, &tournament_id).await;

    // WHEN
    let response = app.client
        .get(app.url(&format!(
            "/users/{}/tournaments/{}/permissions?permission_name=WriteTeams&permission_name=ReadTeams",
            user_id, tournament_id
        )))
        .bearer_auth(&organizer_token)
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn user_not_assigned_to_tournament_returns_401() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_alpha_id = get_id_of_a_new_tournament(&app, "tournament alpha").await?;
    let tournament_beta_id = get_id_of_a_new_tournament(&app, "tournament beta").await?;
    let user_name = Uuid::now_v7().to_string();
    let user_id = get_id_of_a_new_user(&app, &user_name, "password").await;

    // Assign user to tournament alpha only
    let admin_token = get_session_token_for_infrastructure_admin(&app).await;
    create_roles(
        &app,
        &user_id,
        &tournament_alpha_id,
        vec![Role::Organizer],
        &admin_token,
    )
    .await;

    let user_token =
        crate::common::auth_utils::get_session_token_for(&app, &user_name, "password")
            .await
            .unwrap();

    // WHEN - Try to check permission in tournament they're not assigned to
    let response = check_permission(
        &app,
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
