use std::{future::IntoFuture, vec};

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup, tournaments::roles::Role};
use uuid::Uuid;

use crate::common::{
    test_app::TestApp,
    auth_utils::get_session_token_for,
    create_app, create_listener,
    debates_utils::get_id_of_a_new_debate,
    get_response_json, prepare_empty_database,
    roles_utils::create_roles,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{
        create_user, get_id_of_a_new_judge, get_id_of_a_new_user, get_judge_token,
        get_marshal_token, get_organizer_token,
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
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let debate_id = get_id_of_a_new_debate(&app, &tournament_id).await?;
    let token = get_judge_token(&app, &tournament_id).await;
    let proposition_won = true;

    // WHEN
    let response = create_verdict(
        &app,
        &tournament_id,
        &judge_id,
        &debate_id,
        &proposition_won,
        &token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = get_response_json(response).await?;
    assert_eq!(response_body["judge_user_id"], judge_id.to_owned());
    assert_eq!(response_body["proposition_won"], true);
    Ok(())
}

#[tokio::test]
#[serial]
async fn making_verdicts_should_be_only_allowed_on_existing_debates(
) -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let debate_id = Uuid::now_v7().to_string();
    let token = get_judge_token(&app, &tournament_id).await;
    let proposition_won = true;

    // WHEN
    let response = create_verdict(
        &app,
        &tournament_id,
        &judge_id,
        &debate_id,
        &proposition_won,
        &token,
    )
    .await;

    // THEN
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
#[serial]
async fn judges_from_other_tournaments_should_not_be_able_to_submit_verdicts(
) -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id_alpha = get_id_of_a_new_tournament(&app, "alpha").await?;
    let tournament_id_bravo = get_id_of_a_new_tournament(&app, "bravo").await?;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id_alpha).await?;
    let debate_id = get_id_of_a_new_debate(&app, &tournament_id_alpha).await?;
    let token = get_judge_token(&app, &tournament_id_bravo).await;
    let proposition_won = true;

    // WHEN
    let response = create_verdict(
        &app,
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
    let app = TestApp::spawn().await;

    let judge_username = "judge";
    let judge_password = "dredd";

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let judge_id = get_id_of_a_new_user(&app, judge_username, judge_password).await;
    create_roles(&app, &judge_id, &tournament_id, vec![Role::Judge], &token).await;
    let debate_id = get_id_of_a_new_debate(&app, &tournament_id).await?;
    create_user(&app, judge_username, judge_password, &token).await;
    let token = get_session_token_for(&app, judge_username, judge_password).await?;

    let verdict_id =
        get_id_of_a_new_verdict(&app, &tournament_id, &judge_id, &debate_id, &true, &token)
            .await?;

    // WHEN
    let response = get_verdict(&app, &verdict_id, &tournament_id, &debate_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[serial]
async fn anyone_should_be_able_to_list_verdicts() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let organizer_token = get_organizer_token(&app, &tournament_id).await;
    let debate_id = get_id_of_a_new_debate(&app, &tournament_id).await?;

    let judge_username_alpha = "judge";
    let judge_password_alpha = "dredd";
    let judge_id_alpha =
        get_id_of_a_new_user(&app, judge_username_alpha, judge_password_alpha).await;
    create_roles(
        &app, 
        &judge_id_alpha,
        &tournament_id,
        vec![Role::Judge],
        &organizer_token,
    )
    .await;
    let token_alpha =
        get_session_token_for(&app, judge_username_alpha, judge_password_alpha).await?;

    let judge_username_bravo = "anna maria";
    let judge_password_bravo = "wesołowska";
    let judge_id_bravo =
        get_id_of_a_new_user(&app, judge_username_bravo, judge_password_bravo).await;
    create_roles(
        &app, 
        &judge_id_bravo,
        &tournament_id,
        vec![Role::Judge],
        &organizer_token,
    )
    .await;
    let token_bravo =
        get_session_token_for(&app, judge_username_bravo, judge_password_bravo).await?;

    let tokens_to_test = vec![
        get_marshal_token(&app, &tournament_id).await,
        token_alpha.clone(),
        token_bravo.clone(),
        organizer_token,
    ];

    create_verdict(
        &app,
        &tournament_id,
        &judge_id_alpha,
        &debate_id,
        &true,
        &token_alpha,
    )
    .await;
    create_verdict(
        &app,
        &tournament_id,
        &judge_id_bravo,
        &debate_id,
        &false,
        &token_bravo,
    )
    .await;

    for token in tokens_to_test {
        // WHEN
        let response = get_all_verdicts(&app, &tournament_id, &debate_id, &token).await;

        // THEN
        assert_eq!(response.status(), StatusCode::OK);

        let response_body = response.json::<Vec<serde_json::Value>>().await.unwrap();
        assert_eq!(response_body.len(), 2);
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn judges_should_be_able_to_patch_verdicts() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let judge_username = "judge";
    let judge_password = "dredd";
    let initial_verdict = true;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let judge_id = get_id_of_a_new_user(&app, judge_username, judge_password).await;
    create_roles(&app, &judge_id, &tournament_id, vec![Role::Judge], &token).await;
    let debate_id = get_id_of_a_new_debate(&app, &tournament_id).await?;
    create_user(&app, judge_username, judge_password, &token).await;
    let token = get_session_token_for(&app, judge_username, judge_password).await?;

    let verdict_id = get_id_of_a_new_verdict(
        &app,
        &tournament_id,
        &judge_id,
        &debate_id,
        &initial_verdict,
        &token,
    )
    .await?;

    // WHEN
    let response = patch_verdict(
        &app,
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

    let response_body = get_response_json(response).await?;
    assert_eq!(response_body["proposition_won"], !initial_verdict);
    Ok(())
}

#[tokio::test]
#[serial]
async fn judges_should_be_able_to_delete_verdicts() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let judge_username = "judge";
    let judge_password = "dredd";

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let judge_id = get_id_of_a_new_user(&app, judge_username, judge_password).await;
    create_roles(&app, &judge_id, &tournament_id, vec![Role::Judge], &token).await;
    let debate_id = get_id_of_a_new_debate(&app, &tournament_id).await?;
    create_user(&app, judge_username, judge_password, &token).await;
    let token = get_session_token_for(&app, judge_username, judge_password).await?;

    let verdict_id =
        get_id_of_a_new_verdict(&app, &tournament_id, &judge_id, &debate_id, &true, &token)
            .await?;

    // WHEN
    let response = delete_verdict(&app, &verdict_id, &tournament_id, &debate_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    Ok(())
}
