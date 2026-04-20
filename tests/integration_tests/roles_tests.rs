use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::{
    omni_error::OmniError,
    setup::{self},
    tournaments::roles::Role,
};

use crate::common::{
    test_app::TestApp,
    auth_utils::{get_session_token_for, get_session_token_for_infrastructure_admin},
    create_app, create_listener, prepare_empty_database,
    roles_utils::{create_roles, delete_roles, get_roles, patch_roles},
    tournament_utils::{create_tournament, get_id_of_a_new_tournament},
    user_utils::{create_user, get_id_of_a_new_user, get_organizer_token},
};

#[tokio::test]
#[serial]
async fn admin_should_be_able_to_assign_roles() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;
    // WHEN
    let token = get_session_token_for_infrastructure_admin(&app).await;
    let user_id = create_user(&app, "some marshal", "some password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament(&app, "some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let response = create_roles(
        &app,
        &user_id,
        &tournament_id,
        vec![Role::Judge, Role::Organizer],
        &token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let granted_roles = response.json::<Vec<String>>().await.unwrap();
    assert_eq!(granted_roles.len(), 2);
    assert!(granted_roles.contains(&"Judge".to_string()));
    assert!(granted_roles.contains(&"Organizer".to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_assign_roles() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let user_id = get_id_of_a_new_user(&app, "some marshal", "some password").await;
    let tournament_id = get_id_of_a_new_tournament(&app, "some tournament").await?;
    let organizer_token = get_organizer_token(&app, &tournament_id).await;

    // WHEN
    let response = create_roles(
        &app,
        &user_id,
        &tournament_id,
        vec![Role::Marshal],
        &organizer_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let granted_roles = response.json::<Vec<String>>().await.unwrap();
    assert_eq!(granted_roles.len(), 1);
    assert!(granted_roles.contains(&"Marshal".to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_from_other_tournaments_should_not_be_able_to_assign_roles(
) -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let user_id = get_id_of_a_new_user(&app, "some marshal", "some password").await;
    let tournament_alpha_id = get_id_of_a_new_tournament(&app, "alpha").await?;

    let tournament_beta_id = get_id_of_a_new_tournament(&app, "beta").await?;
    let organizer_token_beta = get_organizer_token(&app, &tournament_beta_id).await;

    // WHEN
    let response = create_roles(
        &app,
        &user_id,
        &tournament_alpha_id,
        vec![Role::Marshal],
        &organizer_token_beta,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
#[serial]
async fn granting_duplicate_roles_should_cause_conflicts() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    // WHEN
    let token = get_session_token_for_infrastructure_admin(&app).await;
    let user_id = create_user(&app, "some marshal", "some other password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament(&app, "some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let first_response =
        create_roles(&app, &user_id, &tournament_id, vec![Role::Organizer], &token).await;
    let second_response =
        create_roles(&app, &user_id, &tournament_id, vec![Role::Marshal], &token).await;

    // THEN
    assert_eq!(first_response.status(), StatusCode::OK);
    assert_eq!(second_response.status(), StatusCode::CONFLICT);

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_be_visible_to_other_tournament_users() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let alice_handle = "some organizer";
    let alice_password = "some password";

    // WHEN
    let token = get_session_token_for_infrastructure_admin(&app).await;
    let alice_id = create_user(&app, alice_handle, alice_password, &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = create_user(&app, "some marshal", "some other password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament(&app, "some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    create_roles(&app, &alice_id, &tournament_id, vec![Role::Organizer], &token).await;
    create_roles(&app, &bob_id, &tournament_id, vec![Role::Marshal], &token).await;
    let alice_token = get_session_token_for(&app, alice_handle, alice_password)
        .await
        .unwrap();
    let response = get_roles(&app, &bob_id, &tournament_id, &alice_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let granted_roles = response.json::<Vec<String>>().await.unwrap();
    assert_eq!(granted_roles.len(), 1);
    assert!(granted_roles.contains(&"Marshal".to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_not_be_visible_to_other_users_from_outside_tournament(
) -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let mallory_handle = "some organizer";
    let mallory_password = "some password";

    // WHEN
    let token = get_session_token_for_infrastructure_admin(&app).await;
    let alice_id = create_user(&app, "a nice", "set of credentials", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    create_user(&app, mallory_handle, mallory_password, &token).await;
    let tournament_id = create_tournament(&app, "some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    create_roles(&app, &alice_id, &tournament_id, vec![Role::Organizer], &token).await;
    let mallory_token = get_session_token_for(&app, mallory_handle, mallory_password)
        .await
        .unwrap();
    let response = get_roles(&app, &alice_id, &tournament_id, &mallory_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_be_modifiable() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    // WHEN
    let token = get_session_token_for_infrastructure_admin(&app).await;
    let user_id = create_user(&app, "some marshal", "some password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament(&app, "some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    create_roles(
        &app, 
        &user_id,
        &tournament_id,
        vec![Role::Judge, Role::Organizer],
        &token,
    )
    .await;

    let new_roles = vec![Role::Marshal];
    let response = patch_roles(&app, &user_id, &tournament_id, new_roles, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let granted_roles = response.json::<Vec<String>>().await.unwrap();
    assert_eq!(granted_roles.len(), 1);
    assert!(granted_roles.contains(&"Marshal".to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_be_deletable() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    // WHEN
    let token = get_session_token_for_infrastructure_admin(&app).await;
    let user_id = create_user(&app, "some marshal", "some password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament(&app, "some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    create_roles(
        &app,
        &user_id,
        &tournament_id,
        vec![Role::Judge, Role::Organizer],
        &token,
    )
    .await;

    let response = delete_roles(&app, &user_id, &tournament_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    Ok(())
}
