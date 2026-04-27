use std::collections::HashMap;

use crate::common::test_app::TestApp;

use reqwest::{Response, StatusCode};
use serde_json::{json, Value};
use tau::{omni_error::OmniError};
use uuid::Uuid;

pub async fn get_id_of_a_new_group_phase(
    app:&TestApp,
    tournament_id: &str,
    token: &str,
) -> Result<String, OmniError> {
    match create_phase(&app, tournament_id, &false, &token)
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
    app: &TestApp,
    tournament_id: &str,
    is_finals: &bool,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert(
        "name",
        Value::String(format!("phase_{}", Uuid::now_v7().to_string())),
    );
    request_body.insert("tournament_id", Value::String(tournament_id.to_owned()));
    request_body.insert("status", Value::String("Planned".to_owned()));
    request_body.insert("is_finals", Value::Bool(is_finals.to_owned()));

    app.client
        .post(app.url(&format!(
            "/tournaments/{}/phases",
            tournament_id
        )))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
