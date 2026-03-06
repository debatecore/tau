use reqwest::{Client, Response};
use tau::setup::get_socket_addr;
use tau::tournament::roles::Role;

pub async fn grant_roles(
    user_id: &str,
    tournament_id: &str,
    roles: Vec<Role>,
    token: &str,
) -> Response {
    let socket_address = get_socket_addr();
    let client = Client::new();
    let roles_string: String = serde_json::to_string(&roles).unwrap();

    client
        .post(format!(
            "http://{}/user/{}/tournament/{}/roles",
            socket_address, user_id, tournament_id
        ))
        .body(roles_string)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
