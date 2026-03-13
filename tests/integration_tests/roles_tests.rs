use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::{
    omni_error::OmniError,
    setup::{self},
    tournament::roles::Role,
};

use crate::common::{
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
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let user_id = create_user("some marshall", "some password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament("some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let response = create_roles(
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
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let user_id = get_id_of_a_new_user("some marshall", "some password").await;
    let tournament_id = get_id_of_a_new_tournament("some tournament").await?;
    let organizer_token = get_organizer_token(&tournament_id).await;

    // WHEN
    let response = create_roles(
        &user_id,
        &tournament_id,
        vec![Role::Marshall],
        &organizer_token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let granted_roles = response.json::<Vec<String>>().await.unwrap();
    assert_eq!(granted_roles.len(), 1);
    assert!(granted_roles.contains(&"Marshall".to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_from_other_tournaments_should_not_be_able_to_assign_roles(
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

    let user_id = get_id_of_a_new_user("some marshall", "some password").await;
    let tournament_alpha_id = get_id_of_a_new_tournament("alpha").await?;

    let tournament_beta_id = get_id_of_a_new_tournament("beta").await?;
    let organizer_token_beta = get_organizer_token(&tournament_beta_id).await;

    // WHEN
    let response = create_roles(
        &user_id,
        &tournament_alpha_id,
        vec![Role::Marshall],
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
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let user_id = create_user("some marshall", "some other password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament("some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let first_response =
        create_roles(&user_id, &tournament_id, vec![Role::Organizer], &token).await;
    let second_response =
        create_roles(&user_id, &tournament_id, vec![Role::Marshall], &token).await;

    // THEN
    assert_eq!(first_response.status(), StatusCode::OK);
    assert_eq!(second_response.status(), StatusCode::CONFLICT);

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_be_visible_to_other_tournament_users() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let alice_handle = "some organizer";
    let alice_password = "some password";

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let alice_id = create_user(alice_handle, alice_password, &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = create_user("some marshall", "some other password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament("some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    create_roles(&alice_id, &tournament_id, vec![Role::Organizer], &token).await;
    create_roles(&bob_id, &tournament_id, vec![Role::Marshall], &token).await;
    let alice_token = get_session_token_for(alice_handle, alice_password)
        .await
        .unwrap();
    let response = get_roles(&bob_id, &tournament_id, &alice_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let granted_roles = response.json::<Vec<String>>().await.unwrap();
    assert_eq!(granted_roles.len(), 1);
    assert!(granted_roles.contains(&"Marshall".to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_not_be_visible_to_other_users_from_outside_tournament(
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

    let mallory_handle = "some organizer";
    let mallory_password = "some password";

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let alice_id = create_user("a nice", "set of credentials", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    create_user(mallory_handle, mallory_password, &token).await;
    let tournament_id = create_tournament("some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    create_roles(&alice_id, &tournament_id, vec![Role::Organizer], &token).await;
    let mallory_token = get_session_token_for(mallory_handle, mallory_password)
        .await
        .unwrap();
    let response = get_roles(&alice_id, &tournament_id, &mallory_token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_be_modifiable() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let user_id = create_user("some marshall", "some password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament("some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    create_roles(
        &user_id,
        &tournament_id,
        vec![Role::Judge, Role::Organizer],
        &token,
    )
    .await;

    let new_roles = vec![Role::Marshall];
    let response = patch_roles(&user_id, &tournament_id, new_roles, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let granted_roles = response.json::<Vec<String>>().await.unwrap();
    assert_eq!(granted_roles.len(), 1);
    assert!(granted_roles.contains(&"Marshall".to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn roles_should_be_deletable() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    // WHEN
    let token = get_session_token_for_infrastructure_admin().await;
    let user_id = create_user("some marshall", "some password", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let tournament_id = create_tournament("some tournament", "st", &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    create_roles(
        &user_id,
        &tournament_id,
        vec![Role::Judge, Role::Organizer],
        &token,
    )
    .await;

    let response = delete_roles(&user_id, &tournament_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    Ok(())
}
