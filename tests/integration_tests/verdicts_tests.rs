use std::{future::IntoFuture, vec};

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup, tournaments::roles::Role};

use crate::common::{
    auth_utils::get_session_token_for,
    create_app, create_listener,
    debates_utils::get_id_of_a_new_debate,
    prepare_empty_database,
    roles_utils::create_roles,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{
        create_user, get_id_of_a_new_judge, get_id_of_a_new_user, get_judge_token,
        get_organizer_token,
    },
    verdicts_utils::{
        create_verdict, delete_verdict, get_all_verdicts, get_id_of_a_new_verdict,
        get_verdict, patch_verdict,
    },
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

#[tokio::test]
#[serial]
async fn judges_from_other_tournaments_should_not_be_able_to_submit_verdicts(
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

    let tournament_id_alpha = get_id_of_a_new_tournament("alpha").await?;
    let tournament_id_bravo = get_id_of_a_new_tournament("bravo").await?;
    let judge_id = get_id_of_a_new_judge(&tournament_id_alpha).await?;
    let debate_id = get_id_of_a_new_debate(&tournament_id_alpha).await?;
    let token = get_judge_token(&tournament_id_bravo).await;
    let proposition_won = true;

    // WHEN
    let response = create_verdict(
        &tournament_id_alpha,
        &judge_id,
        &debate_id,
        &proposition_won,
        &token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_get_verdicts() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let judge_username = "judge";
    let judge_password = "dredd";

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;
    let judge_id = get_id_of_a_new_user(judge_username, judge_password).await;
    create_roles(&judge_id, &tournament_id, vec![Role::Judge], &token).await;
    let debate_id = get_id_of_a_new_debate(&tournament_id).await?;
    create_user(judge_username, judge_password, &token).await;
    let token = get_session_token_for(judge_username, judge_password).await?;

    let verdict_id =
        get_id_of_a_new_verdict(&tournament_id, &judge_id, &debate_id, &true, &token)
            .await?;

    // WHEN
    let response = get_verdict(&verdict_id, &tournament_id, &debate_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_list_verdicts() -> Result<(), OmniError> {
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
    let token = get_organizer_token(&tournament_id).await;
    let debate_id = get_id_of_a_new_debate(&tournament_id).await?;

    let judge_username_alpha = "judge";
    let judge_password_alpha = "dredd";
    let judge_id_alpha =
        get_id_of_a_new_user(judge_username_alpha, judge_password_alpha).await;
    create_roles(&judge_id_alpha, &tournament_id, vec![Role::Judge], &token).await;
    let token_alpha =
        get_session_token_for(judge_username_alpha, judge_password_alpha).await?;

    let judge_username_bravo = "anna maria";
    let judge_password_bravo = "wesołowska";
    let judge_id_bravo =
        get_id_of_a_new_user(judge_username_bravo, judge_password_bravo).await;
    create_roles(&judge_id_bravo, &tournament_id, vec![Role::Judge], &token).await;
    let token_bravo =
        get_session_token_for(judge_username_bravo, judge_password_bravo).await?;

    create_verdict(
        &tournament_id,
        &judge_id_alpha,
        &debate_id,
        &true,
        &token_alpha,
    )
    .await;
    create_verdict(
        &tournament_id,
        &judge_id_bravo,
        &debate_id,
        &false,
        &token_bravo,
    )
    .await;

    // WHEN
    let response = get_all_verdicts(&tournament_id, &debate_id, &token_alpha).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.json::<Vec<serde_json::Value>>().await.unwrap();
    assert_eq!(response_body.len(), 2);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_patch_verdicts() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let judge_username = "judge";
    let judge_password = "dredd";
    let initial_verdict = true;

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;
    let judge_id = get_id_of_a_new_user(judge_username, judge_password).await;
    create_roles(&judge_id, &tournament_id, vec![Role::Judge], &token).await;
    let debate_id = get_id_of_a_new_debate(&tournament_id).await?;
    create_user(judge_username, judge_password, &token).await;
    let token = get_session_token_for(judge_username, judge_password).await?;

    let verdict_id = get_id_of_a_new_verdict(
        &tournament_id,
        &judge_id,
        &debate_id,
        &initial_verdict,
        &token,
    )
    .await?;

    // WHEN
    let response = patch_verdict(
        &verdict_id,
        &tournament_id,
        &judge_id,
        &debate_id,
        &!initial_verdict,
        &token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(
        response_body["proposition_won"]
            .as_bool()
            .unwrap()
            .to_owned(),
        !initial_verdict
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn judges_should_be_able_to_delete_verdicts() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let judge_username = "judge";
    let judge_password = "dredd";

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;
    let judge_id = get_id_of_a_new_user(judge_username, judge_password).await;
    create_roles(&judge_id, &tournament_id, vec![Role::Judge], &token).await;
    let debate_id = get_id_of_a_new_debate(&tournament_id).await?;
    create_user(judge_username, judge_password, &token).await;
    let token = get_session_token_for(judge_username, judge_password).await?;

    let verdict_id =
        get_id_of_a_new_verdict(&tournament_id, &judge_id, &debate_id, &true, &token)
            .await?;

    // WHEN
    let response = delete_verdict(&verdict_id, &tournament_id, &debate_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    Ok(())
}
