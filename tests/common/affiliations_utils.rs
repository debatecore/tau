use std::collections::HashMap;

use reqwest::{Client, Response};
use tau::setup::get_socket_addr;

pub async fn create_affiliation(judge_id: &str, team_id: &str, token: &str) -> Response {
    let socket_address = get_socket_addr();
    let mut request_body = HashMap::new();
    let client = Client::new();

    request_body.insert("judge_user_id", judge_id);
    request_body.insert("team_id", team_id);

    client
        .post(format!(
            "http://{}/user/{}/affiliations",
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
