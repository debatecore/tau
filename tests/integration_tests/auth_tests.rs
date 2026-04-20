use reqwest::StatusCode;
use serial_test::serial;
use std::future::IntoFuture;
use tau::setup::{self};

use crate::common::{
    auth_utils::login_with_credentials, create_app, create_listener,
    prepare_empty_database,
};

#[tokio::test]
#[serial]
async fn login_as_infraadmin_should_work_out_of_the_box() {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    // WHEN
    let res = login_with_credentials("admin", "admin").await;

    // THEN
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.is_ok(), true);
}
