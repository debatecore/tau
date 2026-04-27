use std::collections::HashMap;

use crate::common::test_app::TestApp;

use reqwest::{Response, StatusCode};
use serde_json::Value;
use tau::{omni_error::OmniError};
use uuid::Uuid;

pub async fn get_id_of_a_new_round(
    app: &TestApp,
    tournament_id: &str,
    phase_id: &str,
    token: &str,
) -> Result<String, OmniError> {
    match create_round(app, tournament_id, phase_id, &token)
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

pub async fn create_round(app: &TestApp, tournament_id: &str, phase_id: &str, token: &str) -> Response {
    let mut request_body = HashMap::new();
    let phase_name = format!("phase_{}", Uuid::now_v7().to_string());

    request_body.insert("phase_id", Value::String(phase_id.to_owned()));
    request_body.insert("status", Value::String("Planned".to_owned()));
    request_body.insert("name", Value::String(phase_name));

    app.client
        .post(app.url(&format!(
            "/tournaments/{}/phases/{}/rounds",
            tournament_id, phase_id
        )))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
