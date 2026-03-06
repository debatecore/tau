use std::future::IntoFuture;

use reqwest::{Client, StatusCode};
use serial_test::serial;
use tau::{
    setup::{self, get_socket_addr},
    tournament::roles::Role,
};

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin, create_app, create_listener,
    prepare_empty_database, roles_utils::grant_roles,
    tournament_utils::create_tournament, user_utils::create_user,
};

mod common;

#[tokio::test]
#[serial]
async fn admin_should_be_able_to_assigning_roles() {
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
    let token = get_session_token_for_infrastructure_admin().await.unwrap();
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
    let response = grant_roles(
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
}
