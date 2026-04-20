use std::collections::HashMap;

use reqwest::{Client, Response, StatusCode};
use serde_json::{json, Value};
use tau::{omni_error::OmniError, setup::get_local_socket_addr};
use uuid::Uuid;

pub async fn get_id_of_a_new_group_phase(
    tournament_id: &str,
    token: &str,
) -> Result<String, OmniError> {
    match create_phase(tournament_id, &false, &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
    {
        Some(id) => Ok(id.to_owned()),
        None => Err(OmniError::ExplicitError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to get a phase".to_owned(),
        }),
    }
}

pub async fn create_phase(
    tournament_id: &str,
    is_finals: &bool,
    token: &str,
) -> Response {
    let socket_address = get_local_socket_addr();
    let mut request_body = HashMap::new();
    let client = Client::new();

    request_body.insert(
        "name",
        Value::String(format!("phase_{}", Uuid::now_v7().to_string())),
    );
    request_body.insert("tournament_id", Value::String(tournament_id.to_owned()));
    request_body.insert("status", Value::String("Planned".to_owned()));
    request_body.insert("is_finals", Value::Bool(is_finals.to_owned()));

    client
        .post(format!(
            "http://{}/tournaments/{}/phases",
            socket_address, tournament_id
        ))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
