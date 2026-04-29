use reqwest::StatusCode;

use crate::common::{
    test_app::TestApp,
};

#[tokio::test]
async fn test_teapot() {
    // GIVEN
    let app = TestApp::spawn().await;

    // WHEN
    let res = app
        .client
        .get(app.url(&format!("/brew-coffee")))
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(res.status(), StatusCode::IM_A_TEAPOT);
}
