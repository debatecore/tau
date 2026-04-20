use std::future::IntoFuture;

use reqwest::StatusCode;
use serial_test::serial;
use tau::setup;

use crate::common::{create_app, create_listener, prepare_empty_database, test_app::TestApp};

#[tokio::test]
#[serial]
async fn test_teapot() {
    // GIVEN
    let app = TestApp::spawn().await;

    // WHEN
    let res = app.client
        .get(app.url(&format!("/brew-coffee")))
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(res.status(), StatusCode::IM_A_TEAPOT);
}
