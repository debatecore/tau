use std::collections::HashMap;

use reqwest::Response;
use tau::{omni_error::OmniError, tournaments::roles::Role};
use uuid::Uuid;

use crate::common::{
    test_app::TestApp,
    auth_utils::{get_session_token_for, get_session_token_for_infrastructure_admin},
    roles_utils::create_roles,
};

pub async fn create_user(
    app: &TestApp,
    handle: &str,
    password: &str,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();

    request_body.insert("handle", handle);
    request_body.insert("password", password);

    app.client
        .post(app.url("/users"))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_organizer_token(app: &TestApp, tournament_id: &str) -> String {
    get_token_for_user_with_roles(app, vec![Role::Organizer], tournament_id).await
}

pub async fn get_marshal_token(app: &TestApp, tournament_id: &str) -> String {
    get_token_for_user_with_roles(app, vec![Role::Marshal], tournament_id).await
}

pub async fn get_judge_token(app: &TestApp, tournament_id: &str) -> String {
    get_token_for_user_with_roles(app, vec![Role::Judge], tournament_id).await
}

pub async fn get_token_for_user_with_no_roles(app: &TestApp) -> String {
    let handle = Uuid::now_v7().to_string();
    let password = "password";

    get_session_token_for_infrastructure_admin(app).await;
    get_id_of_a_new_user(app, &handle, password).await;
    get_session_token_for(app, &handle, password).await.unwrap()
}

pub async fn get_token_for_user_with_roles(
    app: &TestApp,
    roles: Vec<Role>,
    tournament_id: &str,
) -> String {
    let handle = Uuid::now_v7().to_string();
    let password = "password";

    let token = get_session_token_for_infrastructure_admin(app).await;
    let user_id = get_id_of_a_new_user(app, &handle, password).await;
    create_roles(app, &user_id, tournament_id, roles, &token).await;
    get_session_token_for(app, &handle, password).await.unwrap()
}

pub async fn get_id_of_a_new_user(
    app: &TestApp,
    handle: &str,
    password: &str,
) -> String {
    let token = get_session_token_for_infrastructure_admin(app).await;
    let response = create_user(app, handle, password, &token).await;

    response
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub async fn get_id_of_a_new_judge(
    app: &TestApp,
    tournament_id: &str,
) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin(app).await;
    let judge_id = get_id_of_a_new_user(app, &Uuid::now_v7().to_string(), "some password").await;
    create_roles(app, &judge_id, tournament_id, vec![Role::Judge], &token).await;
    Ok(judge_id)
}

pub async fn check_permission(
    app: &TestApp,
    user_id: &str,
    tournament_id: &str,
    permission_name: &str,
    token: &str,
) -> Response {
    app.client
        .get(app.url(&format!(
            "/users/{}/tournaments/{}/permissions?permission_name={}",
            user_id, tournament_id, permission_name
        )))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
