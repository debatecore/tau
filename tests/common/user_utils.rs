use std::collections::HashMap;

use reqwest::{Client, Response};
use tau::setup::get_socket_addr;

pub async fn create_user(handle: &str, password: &str, token: &str) -> Response {
    let socket_address = get_socket_addr();
    let mut request_body = HashMap::new();
    let client = Client::new();

    request_body.insert("handle", handle);
    request_body.insert("password", password);

    client
        .post(format!("http://{}/user", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}
