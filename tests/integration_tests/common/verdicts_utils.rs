use std::collections::HashMap;

use crate::common::test_app::TestApp;

use reqwest::{Response, StatusCode};
use serde_json::Value;
use tau::{omni_error::OmniError};

pub async fn get_id_of_a_new_verdict(
    app: &TestApp,
    tournament_id: &str,
    judge_id: &str,
    debate_id: &str,
    proposition_won: &bool,
    token: &str,
) -> Result<String, OmniError> {
    let response =
        create_verdict(app, tournament_id, judge_id, debate_id, proposition_won, &token).await;
    if response.status() != StatusCode::OK {
        return Err(OmniError::ExplicitError {
            status: response.status(),
            message: format!(
                "Error creating verdict: {}",
                response.text().await.unwrap()
            ),
        });
    }
    match response.json::<serde_json::Value>().await.unwrap()["id"].as_str() {
        Some(id) => Ok(id.to_owned()),
        None => Err(OmniError::ExplicitError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to get an verdict".to_owned(),
        }),
    }
}

pub async fn create_verdict(
    app: &TestApp,
    tournament_id: &str,
    judge_id: &str,
    debate_id: &str,
    proposition_won: &bool,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("judge_user_id", Value::String(judge_id.to_owned()));
    request_body.insert("debate_id", Value::String(debate_id.to_owned()));
    request_body.insert("proposition_won", Value::Bool(proposition_won.to_owned()));

    app.client
        .post(app.url(&format!(
            "/tournaments/{}/debates/{}/verdicts",
            tournament_id, debate_id
        )))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_verdict(
    app: &TestApp,
    id: &str,
    tournament_id: &str,
    debate_id: &str,
    token: &str,
) -> Response {
    app.client
        .get(app.url(&format!(
            "/tournaments/{}/debates/{}/verdicts/{}",
            tournament_id, debate_id, id
        )))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_all_verdicts(
    app: &TestApp,
    judge_id: &str,
    tournament_id: &str,
    token: &str,
) -> Response {
    app.client
        .get(app.url(&format!(
            "/tournaments/{}/debates/{}/verdicts",
            judge_id, tournament_id
        )))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn patch_verdict(
    app: &TestApp,
    verdict_id: &str,
    tournament_id: &str,
    judge_id: &str,
    debate_id: &str,
    proposition_won: &bool,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("id", Value::String(verdict_id.to_owned()));
    request_body.insert("judge_user_id", Value::String(judge_id.to_owned()));
    request_body.insert("debate_id", Value::String(debate_id.to_owned()));
    request_body.insert("proposition_won", Value::Bool(proposition_won.to_owned()));

    app.client
        .patch(app.url(&format!(
            "/tournaments/{}/debates/{}/verdicts/{}",
            tournament_id, debate_id, verdict_id
        )))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn delete_verdict(
    app: &TestApp,
    id: &str,
    tournament_id: &str,
    debate_id: &str,
    token: &str,
) -> Response {
    app.client
        .delete(app.url(&format!(
            "/tournaments/{}/debates/{}/verdicts/{}",
            tournament_id, debate_id, id
        )))
        .header("accept", "text/plain")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
