use std::collections::HashMap;

use reqwest::{Response, StatusCode};
use tau::{omni_error::OmniError};

use crate::common::test_app::TestApp;

use crate::common::auth_utils::get_session_token_for_infrastructure_admin;

pub async fn get_id_of_a_new_affiliation(
    app: &TestApp,
    judge_id: &str,
    team_id: &str,
) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin(app).await;
    match create_affiliation(app, judge_id, team_id, &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
    {
        Some(id) => Ok(id.to_owned()),
        None => Err(OmniError::ExplicitError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to get an affiliation".to_owned(),
        }),
    }
}

pub async fn create_affiliation(app: &TestApp, judge_id: &str, team_id: &str, token: &str) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("judge_user_id", judge_id);
    request_body.insert("team_id", team_id);

    app.client
        .post(app.url(&format!(
            "/users/{}/affiliations",
             judge_id
        )))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_affiliation(app: &TestApp, id: &str, judge_id: &str, token: &str) -> Response {
    app.client
        .get(app.url(&format!(
            "/users/{}/affiliations/{}",
            judge_id, id
        )))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_all_affiliations(
    app: &TestApp,
    judge_id: &str,
    tournament_id: &str,
    token: &str,
) -> Response {
    app.client
        .get(app.url(&format!(
            "/users/{}/affiliations/tournament/{}",
            judge_id, tournament_id
        )))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn patch_affiliation(
    app: &TestApp,
    id: &str,
    judge_id: &str,
    team_id: &str,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("judge_user_id", judge_id);
    request_body.insert("team_id", team_id);

    app.client
        .patch(app.url(&format!(
            "/users/{}/affiliations/{}",
            judge_id, id
        )))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn delete_affiliation(app: &TestApp, id: &str, judge_id: &str, token: &str) -> Response {
    app.client
        .delete(app.url(&format!(
            "/users/{}/affiliations/{}",
            judge_id, id
        )))
        .header("accept", "text/plain")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
