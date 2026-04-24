use std::collections::HashMap;

use reqwest::{Response, StatusCode};
use tau::omni_error::OmniError;

use crate::common::{
    test_app::TestApp,
    auth_utils::get_session_token_for_infrastructure_admin,
};

pub async fn create_tournament(
    app: &TestApp,
    full_name: &str,
    shortened_name: &str,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("full_name", full_name);
    request_body.insert("shortened_name", shortened_name);

    app.client
        .post(app.url("/tournaments"))
        .json(&request_body)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

pub async fn get_id_of_a_new_tournament(
    app: &TestApp,
    full_name: &str,
) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin(app).await;
    let response = create_tournament(
        app,
        full_name,
        &full_name[0..full_name.len() / 5],
        &token,
    )
    .await;

    match response.status() {
        StatusCode::OK => Ok(response
            .json::<serde_json::Value>()
            .await
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned()),
        _ => Err(OmniError::ExplicitError {
            status: response.status(),
            message: format!(
                "Error creating tournament {}: {}",
                full_name,
                response.text().await.unwrap(),
            ),
        }),
    }
}
