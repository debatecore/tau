use std::collections::HashMap;

use reqwest::{Client, Response, StatusCode};
use tau::{omni_error::OmniError, setup::get_socket_addr};

use crate::common::auth_utils::get_session_token_for_infrastructure_admin;

pub async fn get_id_of_a_new_affiliation(
    judge_id: &str,
    team_id: &str,
) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin().await;
    match create_affiliation(judge_id, team_id, &token)
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

pub async fn create_affiliation(judge_id: &str, team_id: &str, token: &str) -> Response {
    let socket_address = get_socket_addr().to_string().replace("0.0.0.0", "127.0.0.1");
    let mut request_body = HashMap::new();
    let client = Client::new();

    request_body.insert("judge_user_id", judge_id);
    request_body.insert("team_id", team_id);

    client
        .post(format!(
            "http://{}/users/{}/affiliations",
            socket_address, judge_id
        ))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_affiliation(id: &str, judge_id: &str, token: &str) -> Response {
    let socket_address = get_socket_addr().to_string().replace("0.0.0.0", "127.0.0.1");
    let client = Client::new();

    client
        .get(format!(
            "http://{}/users/{}/affiliations/{}",
            socket_address, judge_id, id
        ))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_all_affiliations(
    judge_id: &str,
    tournament_id: &str,
    token: &str,
) -> Response {
    let socket_address = get_socket_addr().to_string().replace("0.0.0.0", "127.0.0.1");
    let client = Client::new();

    client
        .get(format!(
            "http://{}/users/{}/affiliations/tournament/{}",
            socket_address, judge_id, tournament_id
        ))
        .header("accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn patch_affiliation(
    id: &str,
    judge_id: &str,
    team_id: &str,
    token: &str,
) -> Response {
    let socket_address = get_socket_addr().to_string().replace("0.0.0.0", "127.0.0.1");
    let client = Client::new();

    let mut request_body = HashMap::new();
    request_body.insert("judge_user_id", judge_id);
    request_body.insert("team_id", team_id);

    client
        .patch(format!(
            "http://{}/users/{}/affiliations/{}",
            socket_address, judge_id, id
        ))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn delete_affiliation(id: &str, judge_id: &str, token: &str) -> Response {
    let socket_address = get_socket_addr().to_string().replace("0.0.0.0", "127.0.0.1");
    let client = Client::new();

    client
        .delete(format!(
            "http://{}/users/{}/affiliations/{}",
            socket_address, judge_id, id
        ))
        .header("accept", "text/plain")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
