use reqwest::StatusCode;

use crate::common::{
    auth_utils::login_with_credentials, test_app::TestApp,
};

#[tokio::test]
async fn login_as_infraadmin_should_work_out_of_the_box() {
    // GIVEN
    let app = TestApp::spawn().await;

    // WHEN
    let res = login_with_credentials(&app, "admin", "admin").await;

    // THEN
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.is_ok(), true);
}
