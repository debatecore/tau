use std::collections::HashMap;

use reqwest::{Response, StatusCode};
use tau::omni_error::OmniError;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{
    auth_utils::get_session_token_for_infrastructure_admin, test_app::TestApp,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub async fn create_tournament(app: &TestApp, full_name: &str, token: &str) -> Response {
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

pub async fn get_id_of_a_new_tournament(
    app: &TestApp,
    full_name: &str,
) -> Result<String, OmniError> {
    let token = get_session_token_for_infrastructure_admin(app).await;
    let response = create_tournament(app, full_name, &token).await;

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

fn shorten(word: &str) -> String {
    let len = word.chars().count();
    let id = short_id();

    if len <= 5 {
        return capitalize(&format!("{}{}", word, id));
    }

    let first = word.chars().next().unwrap();
    let last = word.chars().last().unwrap();

    capitalize(&format!("{}{}{}{}", first, len - 2, last, id))
}

fn short_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let count = COUNTER.fetch_add(1, Ordering::Relaxed) as u128;

    let value = now ^ count;

    base36(value % 1679616) // 36^4, gives up to 4 chars
}

fn base36(mut value: u128) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    if value == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();

    while value > 0 {
        let index = (value % 36) as usize;
        result.push(CHARS[index] as char);
        value /= 36;
    }

    result.iter().rev().collect()
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();

    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
