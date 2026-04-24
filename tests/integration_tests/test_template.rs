use std::future::IntoFuture;

use serial_test::serial;
use tau::{
    omni_error::OmniError,
    setup::{self, get_client_socket_addr},
};
use test_env_helpers::skip;

use crate::common::{
    test_app::TestApp,
    auth_utils::get_session_token_for_infrastructure_admin,
    create_app, create_listener, prepare_empty_database,
    tournament_utils::{create_tournament, get_id_of_a_new_tournament},
};

mod common;

#[tokio::test]
#[serial]
#[skip]
async fn description_of_what_should_happen() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let token = get_session_token_for_infrastructure_admin(&app).await;

    // WHEN
    let _tournament_id = get_id_of_a_new_tournament(&app, "fancy tournament").await?;

    Ok(())
}
