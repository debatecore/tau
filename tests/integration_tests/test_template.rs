use std::future::IntoFuture;

use serial_test::serial;
use tau::{
    omni_error::OmniError,
    setup::{self, get_socket_addr},
};
use test_env_helpers::skip;

use crate::common::{
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
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let token = get_session_token_for_infrastructure_admin().await;

    // WHEN
    let _tournament_id = get_id_of_a_new_tournament("fancy tournament").await?;

    Ok(())
}
use std::future::IntoFuture;

use serial_test::serial;
use tau::{
    omni_error::OmniError,
    setup::{self, get_client_socket_addr},
};
use test_env_helpers::skip;

use crate::common::{
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
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let token = get_session_token_for_infrastructure_admin().await;

    // WHEN
    let _tournament_id = get_id_of_a_new_tournament("fancy tournament").await?;

    Ok(())
}
