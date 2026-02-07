use std::collections::HashMap;

use reqwest::{Client, Response};
use tau::setup::get_socket_addr;

pub async fn create_tournament(
    full_name: &str,
    shortened_name: &str,
    token: &str,
) -> Response {
    let socket_address = get_socket_addr();
    let mut request_body = HashMap::new();
    request_body.insert("full_name", full_name);
    request_body.insert("shortened_name", shortened_name);

    let client = Client::new();
    client
        .post(format!("http://{}/tournament", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
