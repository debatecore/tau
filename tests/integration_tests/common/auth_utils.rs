use std::collections::HashMap;

use reqwest::{Client, Response, StatusCode};
use tau::{omni_error::OmniError, setup::get_socket_addr};

pub async fn get_session_token_for_infrastructure_admin() -> String {
    get_session_token_for("admin", "admin").await.unwrap()
}

pub async fn get_session_token_for(
    handle: &str,
    password: &str,
) -> Result<String, OmniError> {
    let response = login_with_credentials(handle, password).await;
    match response.status() {
        StatusCode::OK => Ok(response.text().await.unwrap()),
        _ => Err(OmniError::ExplicitError {
            status: response.status(),
            message: response.text().await.unwrap(),
        }),
    }
}

pub async fn login_with_credentials(handle: &str, password: &str) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("login", handle);
    request_body.insert("password", password);
    let socket_address = get_socket_addr().to_string().replace("0.0.0.0", "127.0.0.1");

    let client = Client::new();
    client
        .post(format!("http://{}/auth/login", socket_address))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap()
}
