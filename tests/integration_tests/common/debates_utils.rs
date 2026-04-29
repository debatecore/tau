use std::collections::HashMap;

use crate::common::test_app::TestApp;

use reqwest::{Response, StatusCode};
use tau::{omni_error::OmniError};

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin,
    phases_utils::get_id_of_a_new_group_phase, rounds_utils::get_id_of_a_new_round,
};

pub async fn get_id_of_a_new_debate(app: &TestApp, tournament_id: &str) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin(app).await;
    let phase_id = get_id_of_a_new_group_phase(app, tournament_id, &token).await?;
    let round_id = get_id_of_a_new_round(app, tournament_id, &phase_id, &token).await?;
    match create_debate(app, tournament_id, &round_id, &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
    {
        Some(id) => Ok(id.to_owned()),
        None => Err(OmniError::ExplicitError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to get an debate".to_owned(),
        }),
    }
}

pub async fn create_debate(app: &TestApp, tournament_id: &str, round_id: &str, token: &str) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("round_id", round_id);
    request_body.insert("tournament_id", tournament_id);

    app.client
        .post(app.url(&format!(
            "/tournaments/{}/debates",
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

pub async fn get_debate(app: &TestApp, id: &str, judge_id: &str, token: &str) -> Response {
    app.client
        .get(app.url(&format!(
            "/tournaments/{}/debates/{}",
            judge_id, id
        )))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_all_debates(
    app: &TestApp,
    judge_id: &str,
    tournament_id: &str,
    token: &str,
) -> Response {
    app.client
        .get(app.url(&format!(
            "/users/{}/debates/tournament/{}",
            judge_id, tournament_id
        )))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn patch_debate(
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
            "/users/{}/debates/{}",
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

pub async fn delete_debate(app: &TestApp, id: &str, judge_id: &str, token: &str) -> Response {
    app.client
        .delete(app.url(&format!(
            "/users/{}/debates/{}",
            judge_id, id
        )))
        .header("accept", "text/plain")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
