use std::{future::IntoFuture, vec};

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup};

use crate::common::{
    create_app, create_listener,
    debates_utils::get_id_of_a_new_debate,
    prepare_empty_database,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{get_id_of_a_new_judge, get_judge_token},
    verdicts_utils::create_verdict,
};

#[tokio::test]
#[serial]
async fn judges_should_be_able_to_make_verdicts_on_debates_within_their_tournaments(
) -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
    let debate_id = get_id_of_a_new_debate(&tournament_id).await?;
    let token = get_judge_token(&tournament_id).await;
    let proposition_won = true;

    // WHEN
    let response = create_verdict(
        &tournament_id,
        &judge_id,
        &debate_id,
        &proposition_won,
        &token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(
        response_body["judge_user_id"].as_str().unwrap().to_owned(),
        judge_id.to_owned()
    );
    assert_eq!(
        response_body["proposition_won"]
            .as_bool()
            .unwrap()
            .to_owned(),
        true
    );
    Ok(())
}

// #[tokio::test]
// #[serial]
// async fn organizers_should_be_able_to_get_verdicts() -> Result<(), OmniError> {
//     // GIVEN
//     setup::read_environmental_variables();
//     setup::check_secret_env_var();
//     let state = setup::create_app_state().await;
//     prepare_empty_database(&state.connection_pool).await;
//     let app = create_app(state).await;
//     let listener = create_listener().await;
//     let server = axum::serve(listener, app).into_future();
//     tokio::spawn(server);

//     let tournament_id = get_id_of_a_new_tournament("test").await?;
//     let token = get_organizer_token(&tournament_id).await;
//     let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
//     let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;

//     let verdict_id = get_id_of_a_new_verdict(&judge_id, &team_id).await?;

//     // WHEN
//     let response = get_verdict(&verdict_id, &judge_id, &token).await;

//     // THEN
//     assert_eq!(response.status(), StatusCode::OK);

//     Ok(())
// }

// #[tokio::test]
// #[serial]
// async fn organizers_should_be_able_to_list_verdicts() -> Result<(), OmniError> {
//     // GIVEN
//     setup::read_environmental_variables();
//     setup::check_secret_env_var();
//     let state = setup::create_app_state().await;
//     prepare_empty_database(&state.connection_pool).await;
//     let app = create_app(state).await;
//     let listener = create_listener().await;
//     let server = axum::serve(listener, app).into_future();
//     tokio::spawn(server);

//     let tournament_id = get_id_of_a_new_tournament("test").await?;
//     let token = get_organizer_token(&tournament_id).await;
//     let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
//     let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;
//     let team_id2 = get_id_of_a_new_team(&tournament_id, "aff2").await;

//     create_verdict(&judge_id, &team_id, &token).await;
//     create_verdict(&judge_id, &team_id2, &token).await;

//     // WHEN
//     let response = get_all_verdicts(&judge_id, &tournament_id, &token).await;

//     // THEN
//     assert_eq!(response.status(), StatusCode::OK);
//     let verdicts = response.json::<Vec<serde_json::Value>>().await.unwrap();
//     assert_eq!(verdicts.len(), 2);

//     Ok(())
// }

// #[tokio::test]
// #[serial]
// async fn organizers_should_be_able_to_patch_verdicts() -> Result<(), OmniError> {
//     // GIVEN
//     setup::read_environmental_variables();
//     setup::check_secret_env_var();
//     let state = setup::create_app_state().await;
//     prepare_empty_database(&state.connection_pool).await;

//     let app = create_app(state).await;
//     let listener = create_listener().await;
//     let server = axum::serve(listener, app).into_future();
//     tokio::spawn(server);

//     let tournament_id = get_id_of_a_new_tournament("test").await?;
//     let token = get_organizer_token(&tournament_id).await;
//     let judge_id = get_id_of_a_new_judge(&tournament_id).await?;

//     let team_id = get_id_of_a_new_team(&tournament_id, "aff").await;
//     let new_team_id = get_id_of_a_new_team(&tournament_id, "aff2").await;

//     let verdict_id = get_id_of_a_new_verdict(&judge_id, &team_id).await?;

//     // WHEN
//     let response = patch_verdict(&verdict_id, &judge_id, &new_team_id, &token).await;

//     // THEN
//     assert_eq!(response.status(), StatusCode::OK);

//     Ok(())
// }

// #[tokio::test]
// #[serial]
// async fn organizers_should_be_able_to_delete_verdicts() -> Result<(), OmniError> {
//     // GIVEN
//     setup::read_environmental_variables();
//     setup::check_secret_env_var();
//     let state = setup::create_app_state().await;
//     prepare_empty_database(&state.connection_pool).await;

//     let app = create_app(state).await;
//     let listener = create_listener().await;
//     let server = axum::serve(listener, app).into_future();
//     tokio::spawn(server);

//     let tournament_id = get_id_of_a_new_tournament("some").await?;
//     let token = get_organizer_token(&tournament_id).await;
//     let team_id = get_id_of_a_new_team(&tournament_id, "team").await;
//     let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
//     let verdict_id = get_id_of_a_new_verdict(&judge_id, &team_id).await?;

//     // WHEN
//     let response = delete_verdict(&verdict_id, &judge_id, &token).await;

//     // THEN
//     assert_eq!(response.status(), StatusCode::NO_CONTENT);

//     Ok(())
// }

// #[tokio::test]
// #[serial]
// async fn verdicts_should_not_be_visible_to_judges_and_marshals() -> Result<(), OmniError>
// {
//     // GIVEN
//     setup::read_environmental_variables();
//     setup::check_secret_env_var();
//     let state = setup::create_app_state().await;
//     prepare_empty_database(&state.connection_pool).await;

//     let app = create_app(state).await;
//     let listener = create_listener().await;
//     let server = axum::serve(listener, app).into_future();
//     tokio::spawn(server);

//     let tournament_id = get_id_of_a_new_tournament("some").await?;
//     let team_id = get_id_of_a_new_team(&tournament_id, "team").await;
//     let judge_id = get_id_of_a_new_judge(&tournament_id).await?;
//     let verdict_id = get_id_of_a_new_verdict(&judge_id, &team_id).await?;

//     // WHEN
//     let roles_to_check = vec![Role::Judge, Role::Marshal];
//     for role in roles_to_check {
//         let token = get_token_for_user_with_roles(vec![role], &tournament_id).await;
//         let response = get_verdict(&verdict_id, &judge_id, &token).await;

//         // THEN
//         assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
//     }

//     Ok(())
// }
