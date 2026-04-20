use std::collections::HashMap;

use reqwest::{Response, StatusCode};
use tau::omni_error::OmniError;

use crate::common::test_app::TestApp;

pub async fn get_session_token_for_infrastructure_admin(app: &TestApp) -> String {
    get_session_token_for(app, "admin", "admin").await.unwrap()
}

pub async fn get_session_token_for(
    app: &TestApp,
    handle: &str,
    password: &str,
) -> Result<String, OmniError> {
    let response = login_with_credentials(app, handle, password).await;

    match response.status() {
        StatusCode::OK => Ok(response.text().await.unwrap()),
        _ => Err(OmniError::ExplicitError {
            status: response.status(),
            message: response.text().await.unwrap(),
        }),
    }
}

pub async fn login_with_credentials(
    app: &TestApp,
    handle: &str,
    password: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("login", handle);
    request_body.insert("password", password);

    app.client
        .post(app.url("/auth/login"))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap()
}
