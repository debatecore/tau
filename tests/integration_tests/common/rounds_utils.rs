use std::collections::HashMap;

use reqwest::{Client, Response, StatusCode};
use serde_json::Value;
use tau::{omni_error::OmniError, setup::get_socket_addr};
use uuid::Uuid;

pub async fn get_id_of_a_new_round(
    tournament_id: &str,
    phase_id: &str,
    token: &str,
) -> Result<String, OmniError> {
    match create_round(tournament_id, phase_id, &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
    {
        Some(id) => Ok(id.to_owned()),
        None => Err(OmniError::ExplicitError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to get a round".to_owned(),
        }),
    }
}

pub async fn create_round(tournament_id: &str, phase_id: &str, token: &str) -> Response {
    let socket_address = get_socket_addr();
    let mut request_body = HashMap::new();
    let client = Client::new();
    let phase_name = format!("phase_{}", Uuid::now_v7().to_string());

    request_body.insert("phase_id", Value::String(phase_id.to_owned()));
    request_body.insert("status", Value::String("Planned".to_owned()));
    request_body.insert("name", Value::String(phase_name));

    client
        .post(format!(
            "http://{}/tournaments/{}/phases/{}/rounds",
            socket_address, tournament_id, phase_id
        ))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
