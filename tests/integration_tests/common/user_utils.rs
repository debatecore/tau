use std::collections::HashMap;

use reqwest::{Client, Response};
use tau::{omni_error::OmniError, setup::get_client_socket_addr, tournaments::roles::Role};
use uuid::Uuid;

use crate::common::{
    auth_utils::{get_session_token_for, get_session_token_for_infrastructure_admin},
    roles_utils::create_roles,
};

pub async fn create_user(handle: &str, password: &str, token: &str) -> Response {
    let socket_address = get_client_socket_addr();
    let mut request_body = HashMap::new();
    let client = Client::new();

    request_body.insert("handle", handle);
    request_body.insert("password", password);

    client
        .post(format!("http://{}/users", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_organizer_token(tournament_id: &str) -> String {
    get_token_for_user_with_roles(vec![Role::Organizer], tournament_id).await
}

pub async fn get_marshal_token(tournament_id: &str) -> String {
    get_token_for_user_with_roles(vec![Role::Marshal], tournament_id).await
}

pub async fn get_judge_token(tournament_id: &str) -> String {
    get_token_for_user_with_roles(vec![Role::Judge], tournament_id).await
}

pub async fn get_token_for_user_with_no_roles() -> String {
    let handle = Uuid::now_v7().to_string();
    let password = "password";

    get_session_token_for_infrastructure_admin().await;
    get_id_of_a_new_user(&handle, password).await;
    get_session_token_for(&handle, password).await.unwrap()
}

pub async fn get_token_for_user_with_roles(
    roles: Vec<Role>,
    tournament_id: &str,
) -> String {
    let handle = Uuid::now_v7().to_string();
    let password = "password";

    let token = get_session_token_for_infrastructure_admin().await;
    let user_id = get_id_of_a_new_user(&handle, password).await;
    create_roles(&user_id, tournament_id, roles, &token).await;
    get_session_token_for(&handle, password).await.unwrap()
}

pub async fn get_id_of_a_new_user(handle: &str, password: &str) -> String {
    let token = get_session_token_for_infrastructure_admin().await;
    let response = create_user(handle, password, &token).await;
    response.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub async fn get_id_of_a_new_judge(tournament_id: &str) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin().await;
    let judge_id =
        get_id_of_a_new_user(&Uuid::now_v7().to_string(), "some password").await;
    create_roles(&judge_id, &tournament_id, vec![Role::Judge], &token).await;
    Ok(judge_id)
}

pub async fn check_permission(
    user_id: &str,
    tournament_id: &str,
    permission_name: &str,
    token: &str,
) -> Response {
    let socket_address = get_client_socket_addr();
    let client = Client::new();

    client
        .get(format!(
            "http://{}/users/{}/tournaments/{}/permissions?permission_name={}",
            socket_address, user_id, tournament_id, permission_name
        ))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
