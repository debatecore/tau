use reqwest::{Client, Response};
use tau::setup::get_socket_addr;
use tau::tournaments::plans::TournamentPlan;

pub async fn create_plans(
    user_id: &str,
    tournament_id: &str,
    plans: Vec<TournamentPlan>,
    token: &str,
) -> Response {
    let socket_address = get_socket_addr();
    let client = Client::new();
    let plans_string: String = serde_json::to_string(&plans).unwrap();

    client
        .post(format!(
            "http://{}/users/{}/tournaments/{}/plan",
            socket_address, user_id, tournament_id
        ))
        .body(plans_string)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_plans(user_id: &str, tournament_id: &str, token: &str) -> Response {
    let socket_address = get_socket_addr();
    let client = Client::new();

    client
        .get(format!(
            "http://{}/users/{}/tournaments/{}/plan",
            socket_address, user_id, tournament_id
        ))
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn patch_plans(
    user_id: &str,
    tournament_id: &str,
    plans: Vec<TournamentPlan>,
    token: &str,
) -> Response {
    let socket_address = get_socket_addr();
    let client = Client::new();
    let plans_string: String = serde_json::to_string(&plans).unwrap();

    client
        .patch(format!(
            "http://{}/users/{}/tournaments/{}/plans",
            socket_address, user_id, tournament_id
        ))
        .body(plans_string)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn delete_plans(user_id: &str, tournament_id: &str, token: &str) -> Response {
    let socket_address = get_socket_addr();
    let client = Client::new();

    client
        .delete(format!(
            "http://{}/users/{}/tournaments/{}/plan",
            socket_address, user_id, tournament_id
        ))
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
