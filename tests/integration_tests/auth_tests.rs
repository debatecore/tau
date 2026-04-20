use reqwest::StatusCode;
use serial_test::serial;
use std::future::IntoFuture;
use tau::setup::{self};

use crate::common::{
    test_app::TestApp,
    auth_utils::login_with_credentials, create_app, create_listener,
    prepare_empty_database,
};

#[tokio::test]
#[serial]
async fn login_as_infraadmin_should_work_out_of_the_box() {
    // GIVEN
    let app = TestApp::spawn().await;

    // WHEN
    let res = login_with_credentials(&app, "admin", "admin").await;

    // THEN
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.is_ok(), true);
}
