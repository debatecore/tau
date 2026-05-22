use std::collections::HashMap;

use reqwest::{Response, StatusCode};
use tau::omni_error::OmniError;

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin, test_app::TestApp,
};

use tau::tournaments::shorten;

pub async fn create_tournament(
    app: &TestApp,
    full_name: &str,
    shortened_name: Option<std::string::String>,
    token: &str,
) -> Response {
    if shortened_name == None {
        return create_tournament_without_shortened_name(app, full_name, token).await;
    } else {
        return create_tournament_with_shortened_name(
            app,
            full_name,
            &shortened_name.unwrap_or(shorten(full_name)),
            token,
        )
        .await;
    }
}

pub async fn create_tournament_without_shortened_name(
    app: &TestApp,
    full_name: &str,
    token: &str,
) -> Response {
    let mut request_body = HashMap::new();
    request_body.insert("full_name", full_name);
    let shortened = shorten(&full_name);
    request_body.insert("shortened_name", &shortened);

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

pub async fn create_tournament_with_shortened_name(
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
    let response = create_tournament(app, full_name, None, &token).await;

    match response.status() {
        StatusCode::OK => Ok(response.json::<serde_json::Value>().await.unwrap()["id"]
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

#[cfg(test)]
mod test_shortened_name {
    use tau::tournaments::shorten;
    #[test]
    fn test_shortened_tournament_name_derivation() {
        assert_eq!(shorten("Alpha Bravo Delta Tournament"), "ABD");
        assert_eq!(shorten("Alpha Bravo Delta"), "ABD");
        assert_eq!(shorten("Alpha Bravo"), "ALB");
        assert_eq!(shorten("Alpha"), "ALP");
    }
}
