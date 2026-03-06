use std::collections::HashMap;

use reqwest::{Client, Response};
use tau::setup::get_socket_addr;

use crate::common::auth_utils::{
    get_session_token_for, get_session_token_for_infrastructure_admin,
};

pub async fn create_user(handle: &str, password: &str, token: &str) -> Response {
    let socket_address = get_socket_addr();
    let mut request_body = HashMap::new();
    let client = Client::new();

    request_body.insert("handle", handle);
    request_body.insert("password", password);

    client
        .post(format!("http://{}/user", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_organizer_token() -> String {
    let handle = "organizer";
    let password = "password";

    let token = get_session_token_for_infrastructure_admin().await.unwrap();
    create_user(handle, password, &token).await;
    get_session_token_for(handle, password).await.unwrap()
}
