use std::collections::HashMap;

use reqwest::{Client, Response, StatusCode};
use tau::{omni_error::OmniError, setup::get_socket_addr};

use crate::common::auth_utils::get_session_token_for_infrastructure_admin;

pub async fn create_tournament(
    full_name: &str,
    shortened_name: &str,
    token: &str,
) -> Response {
    let socket_address = get_socket_addr()
        .to_string()
        .replace("0.0.0.0", "127.0.0.1");
    let mut request_body = HashMap::new();
    request_body.insert("full_name", full_name);
    request_body.insert("shortened_name", shortened_name);

    let client = Client::new();
    client
        .post(format!("http://{}/tournaments", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_id_of_a_new_tournament(full_name: &str) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin().await;
    let response =
        create_tournament(full_name, &full_name[0..full_name.len() / 5], &token).await;
    match response.status() {
        StatusCode::OK => Ok(response.json::<serde_json::Value>().await.unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned()),
        _ => Err(OmniError::ExplicitError {
            status: response.status(),
            message: format!(
                "Error creating tournament {}: {}",
                full_name,
                response.text().await.unwrap(),
            ),
        }),
    }
}
use std::collections::HashMap;

use reqwest::{Client, Response, StatusCode};
use tau::{omni_error::OmniError, setup::get_local_socket_addr};

use crate::common::auth_utils::get_session_token_for_infrastructure_admin;

pub async fn create_tournament(
    full_name: &str,
    shortened_name: &str,
    token: &str,
) -> Response {
    let socket_address = get_local_socket_addr();
    let mut request_body = HashMap::new();
    request_body.insert("full_name", full_name);
    request_body.insert("shortened_name", shortened_name);

    let client = Client::new();
    client
        .post(format!("http://{}/tournaments", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_id_of_a_new_tournament(full_name: &str) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin().await;
    let response =
        create_tournament(full_name, &full_name[0..full_name.len() / 5], &token).await;
    match response.status() {
        StatusCode::OK => Ok(response.json::<serde_json::Value>().await.unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned()),
        _ => Err(OmniError::ExplicitError {
            status: response.status(),
            message: format!(
                "Error creating tournament {}: {}",
                full_name,
                response.text().await.unwrap(),
            ),
        }),
    }
}
