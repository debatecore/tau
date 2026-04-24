use reqwest::Response;
use tau::tournaments::roles::Role;

use crate::common::test_app::TestApp;

pub async fn create_roles(
    app: &TestApp,
    user_id: &str,
    tournament_id: &str,
    roles: Vec<Role>,
    token: &str,
) -> Response {
    let roles_string = serde_json::to_string(&roles).unwrap();

    app.client
        .post(app.url(&format!(
            "/users/{}/tournaments/{}/roles",
            user_id, tournament_id
        )))
        .body(roles_string)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_roles(
    app: &TestApp,
    user_id: &str,
    tournament_id: &str,
    token: &str,
) -> Response {
    app.client
        .get(app.url(&format!(
            "/users/{}/tournaments/{}/roles",
            user_id, tournament_id
        )))
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn patch_roles(
    app: &TestApp,
    user_id: &str,
    tournament_id: &str,
    roles: Vec<Role>,
    token: &str,
) -> Response {
    let roles_string = serde_json::to_string(&roles).unwrap();

    app.client
        .patch(app.url(&format!(
            "/users/{}/tournaments/{}/roles",
            user_id, tournament_id
        )))
        .body(roles_string)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn delete_roles(
    app: &TestApp,
    user_id: &str,
    tournament_id: &str,
    token: &str,
) -> Response {
    app.client
        .delete(app.url(&format!(
            "/users/{}/tournaments/{}/roles",
            user_id, tournament_id
        )))
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
