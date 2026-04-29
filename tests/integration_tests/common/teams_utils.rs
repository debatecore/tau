use std::collections::HashMap;

use crate::common::test_app::TestApp;

use reqwest::{Client, Response};
use tau::setup::get_local_socket_addr;

use crate::common::auth_utils::get_session_token_for_infrastructure_admin;

pub async fn get_id_of_a_new_team(app: &TestApp, tournament_id: &str, handle: &str) -> String {
    let token = get_session_token_for_infrastructure_admin(app).await;
    create_team(app, tournament_id, handle, &handle[0..handle.len() / 5], &token)
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub async fn create_team(
    app: &TestApp,
    tournament_id: &str,
    full_name: &str,
    shortened_name: &str,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("tournament_id", tournament_id);
    request_body.insert("full_name", full_name);
    request_body.insert("shortened_name", shortened_name);

    app.client
        .post(app.url(&format!(
            "/tournaments/{}/teams",
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

pub async fn get_team(app: &TestApp, id: &str, tournament_id: &str, token: &str) -> Response {
    app.client
        .get(app.url(&format!(
            "/tournaments/{}/teams/{}",
            tournament_id, id
        )))
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn patch_team(
    app: &TestApp,
    id: &str,
    tournament_id: &str,
    full_name: &str,
    shortened_name: &str,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("tournament_id", tournament_id);
    request_body.insert("full_name", full_name);
    request_body.insert("shortened_name", shortened_name);

    app.client
        .patch(app.url(&format!(
            "/tournaments/{}/teams/{}",
            tournament_id, id
        )))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn delete_team(app: &TestApp, id: &str, tournament_id: &str, token: &str) -> Response {
    app.client
        .delete(app.url(&format!(
            "/tournaments/{}/teams/{}",
            tournament_id, id
        )))
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
