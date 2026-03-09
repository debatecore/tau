use std::{future::IntoFuture, vec};

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup};

use crate::common::{
    affiliations_utils::create_affiliation,
    create_app, create_listener, prepare_empty_database,
    teams_utils::get_id_of_a_new_team,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{get_id_of_a_new_judge, get_organizer_token},
};

mod common;

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_create_affiliations() -> Result<(), OmniError> {
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
    let tournament_id = get_id_of_a_new_tournament("test").await;
    let token = get_organizer_token(&tournament_id).await;
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;
    let response = create_affiliation(&judge_id, &team_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}
