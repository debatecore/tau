use std::{future::IntoFuture, vec};

use reqwest::StatusCode;
use serial_test::serial;
use tau::{omni_error::OmniError, setup, tournaments::roles::Role};

use crate::common::{
    test_app::TestApp,
    affiliations_utils::{
        create_affiliation, delete_affiliation, get_affiliation, get_all_affiliations,
        get_id_of_a_new_affiliation, patch_affiliation,
    },
    create_app, create_listener, prepare_empty_database,
    teams_utils::get_id_of_a_new_team,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{
        get_id_of_a_new_judge, get_organizer_token, get_token_for_user_with_roles,
    },
};

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_create_affiliations() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let team_id = get_id_of_a_new_team(&app, &tournament_id, "aff").await;

    // WHEN
    let response = create_affiliation(&app, &judge_id, &team_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(
        response_body["judge_user_id"].as_str().unwrap().to_owned(),
        judge_id.to_owned()
    );
    assert_eq!(
        response_body["team_id"].as_str().unwrap().to_owned(),
        team_id
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_get_affiliations() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let team_id = get_id_of_a_new_team(&app, &tournament_id, "aff").await;

    let affiliation_id = get_id_of_a_new_affiliation(&app, &judge_id, &team_id).await?;

    // WHEN
    let response = get_affiliation(&app, &affiliation_id, &judge_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_list_affiliations() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let team_id = get_id_of_a_new_team(&app, &tournament_id, "aff").await;
    let team_id2 = get_id_of_a_new_team(&app, &tournament_id, "aff2").await;

    create_affiliation(&app, &judge_id, &team_id, &token).await;
    create_affiliation(&app, &judge_id, &team_id2, &token).await;

    // WHEN
    let response = get_all_affiliations(&app, &judge_id, &tournament_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    let affiliations = response.json::<Vec<serde_json::Value>>().await.unwrap();
    assert_eq!(affiliations.len(), 2);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_patch_affiliations() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;

    let team_id = get_id_of_a_new_team(&app, &tournament_id, "aff").await;
    let new_team_id = get_id_of_a_new_team(&app, &tournament_id, "aff2").await;

    let affiliation_id = get_id_of_a_new_affiliation(&app, &judge_id, &team_id).await?;

    // WHEN
    let response =
        patch_affiliation(&app, &affiliation_id, &judge_id, &new_team_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_delete_affiliations() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "some").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let team_id = get_id_of_a_new_team(&app, &tournament_id, "team").await;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let affiliation_id = get_id_of_a_new_affiliation(&app, &judge_id, &team_id).await?;

    // WHEN
    let response = delete_affiliation(&app, &affiliation_id, &judge_id, &token).await;

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    Ok(())
}

#[tokio::test]
#[serial]
async fn affiliations_should_not_be_visible_to_judges_and_marshals(
) -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "some").await?;
    let team_id = get_id_of_a_new_team(&app, &tournament_id, "team").await;
    let judge_id = get_id_of_a_new_judge(&app, &tournament_id).await?;
    let affiliation_id = get_id_of_a_new_affiliation(&app, &judge_id, &team_id).await?;

    // WHEN
    let roles_to_check = vec![Role::Judge, Role::Marshal];
    for role in roles_to_check {
        let token = get_token_for_user_with_roles(&app, vec![role], &tournament_id).await;
        let response = get_affiliation(&app, &affiliation_id, &judge_id, &token).await;

        // THEN
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    Ok(())
}
