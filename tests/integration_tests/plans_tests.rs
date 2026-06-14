use crate::common::{
    plans_utils::{
        calculate_final_phase_debates, calculate_final_phase_rounds, count_debates,
        count_phases, count_plans, count_rounds, create_plan, patch_plan,
    },
    test_app::TestApp,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::{get_organizer_token, get_token_for_user_with_no_roles},
};
use reqwest::StatusCode;
use serde_json::json;
use tau::{omni_error::OmniError, tournaments::plans::TournamentPlanPatch};
use uuid::Uuid;

const TEST_GROUP_PHASE_ROUNDS: i32 = 4;
const TEST_GROUPS_COUNT: i32 = 8;
const TEST_ADVANCING_TEAMS: i32 = 4;
const TEST_TOTAL_TEAMS: i32 = 32;

const TEST_GROUP_PHASE_ROUNDS_PATCH: i32 = 5;
const TEST_GROUPS_COUNT_PATCH: i32 = 10;
const TEST_ADVANCING_TEAMS_PATCH: i32 = 4;
const TEST_TOTAL_TEAMS_PATCH: i32 = 30;

fn expected_counts(
    group_phase_rounds: i32,
    groups_count: i32,
    advancing_teams: i32,
) -> (i64, i64, i64) {
    let phases = 2;
    let rounds =
        (group_phase_rounds + calculate_final_phase_rounds(advancing_teams)) as i64;
    let debates = (groups_count * group_phase_rounds
        + calculate_final_phase_debates(advancing_teams)) as i64;
    (phases, rounds, debates)
}

#[tokio::test]
async fn tournament_plan_creation_should_impossible_for_other_users(
) -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let token = get_token_for_user_with_no_roles(&app).await;
    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;

    // WHEN
    assert_eq!(
        create_plan(
            &app,
            &tournament_id,
            TEST_GROUP_PHASE_ROUNDS,
            TEST_GROUPS_COUNT,
            TEST_ADVANCING_TEAMS,
            TEST_TOTAL_TEAMS,
            &token
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );

    Ok(())
}

#[tokio::test]
async fn organizers_should_be_able_to_create_tournament_plan() -> Result<(), OmniError> {
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;

    let response = create_plan(
        &app,
        &tournament_id,
        TEST_ADVANCING_TEAMS,
        TEST_GROUP_PHASE_ROUNDS,
        TEST_GROUPS_COUNT,
        TEST_TOTAL_TEAMS,
        &token,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let (expected_phases, expected_rounds, expected_debates) = expected_counts(
        TEST_GROUP_PHASE_ROUNDS,
        TEST_GROUPS_COUNT,
        TEST_ADVANCING_TEAMS,
    );

    assert_eq!(count_plans(&app.pool, &tournament_id).await, 1);
    assert_eq!(
        count_phases(&app.pool, &tournament_id).await,
        expected_phases
    );
    assert_eq!(
        count_rounds(&app.pool, &tournament_id).await,
        expected_rounds
    );
    assert_eq!(
        count_debates(&app.pool, &tournament_id).await,
        expected_debates
    );

    Ok(())
}

#[tokio::test]
async fn organizers_should_be_able_to_get_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;

    let create_response = create_plan(
        &app,
        &tournament_id,
        TEST_ADVANCING_TEAMS,
        TEST_GROUP_PHASE_ROUNDS,
        TEST_GROUPS_COUNT,
        TEST_TOTAL_TEAMS,
        &token,
    )
    .await;

    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    // WHEN
    let response = app
        .client
        .get(app.url(&format!("/tournaments/{}/plan/{}", tournament_id, plan_id)))
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn organizers_should_be_able_to_patch_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;

    let create_response = create_plan(
        &app,
        &tournament_id,
        TEST_ADVANCING_TEAMS,
        TEST_GROUP_PHASE_ROUNDS,
        TEST_GROUPS_COUNT,
        TEST_TOTAL_TEAMS,
        &token,
    )
    .await;

    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    let patch_data = json!({
        "group_phase_rounds": TEST_GROUP_PHASE_ROUNDS_PATCH,
        "groups_count":       TEST_GROUPS_COUNT_PATCH,
        "advancing_teams":    TEST_ADVANCING_TEAMS_PATCH,
        "total_teams":        TEST_TOTAL_TEAMS_PATCH,
    });

    // WHEN
    let response = app
        .client
        .patch(app.url(&format!("/tournaments/{}/plan/{}", tournament_id, plan_id)))
        .json(&patch_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let (new_expected_phases, new_expected_rounds, new_expected_debates) =
        expected_counts(
            TEST_GROUP_PHASE_ROUNDS_PATCH,
            TEST_GROUPS_COUNT_PATCH,
            TEST_ADVANCING_TEAMS_PATCH,
        );

    assert_eq!(count_plans(&app.pool, &tournament_id).await, 1);
    assert_eq!(
        count_phases(&app.pool, &tournament_id).await,
        new_expected_phases
    );
    assert_eq!(
        count_rounds(&app.pool, &tournament_id).await,
        new_expected_rounds
    );
    assert_eq!(
        count_debates(&app.pool, &tournament_id).await,
        new_expected_debates
    );

    Ok(())
}

#[tokio::test]
async fn organizers_should_be_able_to_delete_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;

    let create_response = create_plan(
        &app,
        &tournament_id,
        TEST_ADVANCING_TEAMS,
        TEST_GROUP_PHASE_ROUNDS,
        TEST_GROUPS_COUNT,
        TEST_TOTAL_TEAMS,
        &token,
    )
    .await;

    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    // WHEN
    let response = app
        .client
        .delete(app.url(&format!("/tournaments/{}/plan/{}", tournament_id, plan_id)))
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    assert_eq!(count_plans(&app.pool, &tournament_id).await, 0);
    assert_eq!(count_phases(&app.pool, &tournament_id).await, 0);
    assert_eq!(count_rounds(&app.pool, &tournament_id).await, 0);
    assert_eq!(count_debates(&app.pool, &tournament_id).await, 0);

    Ok(())
}

#[tokio::test]
async fn create_plan_should_rollback_everything_if_underlying_creation_fails(
) -> Result<(), OmniError> {
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let tournament_uuid = Uuid::parse_str(&tournament_id).unwrap();

    let plan = tau::tournaments::plans::TournamentPlan {
        id: Uuid::now_v7(),
        tournament_id: tournament_uuid,
        group_phase_rounds: Some(TEST_GROUP_PHASE_ROUNDS),
        groups_count: Some(TEST_GROUPS_COUNT),
        advancing_teams: Some(TEST_ADVANCING_TEAMS),
        total_teams: Some(TEST_TOTAL_TEAMS),
    };

    let mut transaction = app.pool.begin().await.unwrap();

    let result: Result<(), OmniError> = async {
        let _created = tau::tournaments::plans::TournamentPlan::post_with_transaction(
            &mut transaction,
            tournament_uuid,
            plan,
        )
        .await?;

        Err(OmniError::ExplicitError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "forced failure".to_owned(),
        })
    }
    .await;

    assert!(result.is_err());

    transaction.rollback().await.unwrap();

    assert_eq!(count_plans(&app.pool, &tournament_id).await, 0);
    assert_eq!(count_phases(&app.pool, &tournament_id).await, 0);
    assert_eq!(count_rounds(&app.pool, &tournament_id).await, 0);
    assert_eq!(count_debates(&app.pool, &tournament_id).await, 0);

    Ok(())
}

#[tokio::test]
async fn plans_should_be_validated_before_creation() -> Result<(), OmniError> {
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let incorrect_plans = get_incorrect_tournament_plans();

    for plan in incorrect_plans.iter() {
        let response = create_plan(
            &app,
            &tournament_id,
            plan.advancing_teams.unwrap(),
            plan.group_phase_rounds.unwrap(),
            plan.groups_count.unwrap(),
            plan.total_teams.unwrap(),
            &token,
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    Ok(())
}

#[tokio::test]
async fn plans_should_be_validated_before_patching() -> Result<(), OmniError> {
    let app = TestApp::spawn().await;

    let tournament_id = get_id_of_a_new_tournament(&app, "test").await?;
    let token = get_organizer_token(&app, &tournament_id).await;
    let incorrect_plans = get_incorrect_tournament_plans();

    for plan in incorrect_plans.iter() {
        let response = patch_plan(
            &app,
            &tournament_id,
            plan.advancing_teams,
            plan.group_phase_rounds,
            plan.groups_count,
            plan.total_teams,
            &token,
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    Ok(())
}

fn get_incorrect_tournament_plans() -> Vec<TournamentPlanPatch> {
    vec![
        get_plan_with_negative_values(),
        get_plan_with_more_advancing_than_total_teams(),
        get_plan_with_advancing_teams_not_power_of_two(),
        get_plan_with_parameter_set_to_zero(),
        get_plan_with_more_groups_than_teams(),
    ]
}

fn get_plan_with_negative_values() -> TournamentPlanPatch {
    TournamentPlanPatch {
        total_teams: Some(16),
        advancing_teams: Some(4),
        group_phase_rounds: Some(-1),
        groups_count: Some(5),
    }
}

fn get_plan_with_parameter_set_to_zero() -> TournamentPlanPatch {
    TournamentPlanPatch {
        total_teams: Some(16),
        advancing_teams: Some(4),
        group_phase_rounds: Some(0),
        groups_count: Some(5),
    }
}

fn get_plan_with_more_advancing_than_total_teams() -> TournamentPlanPatch {
    TournamentPlanPatch {
        total_teams: Some(16),
        advancing_teams: Some(32),
        group_phase_rounds: Some(4),
        groups_count: Some(4),
    }
}

fn get_plan_with_advancing_teams_not_power_of_two() -> TournamentPlanPatch {
    TournamentPlanPatch {
        total_teams: Some(16),
        advancing_teams: Some(3),
        group_phase_rounds: Some(4),
        groups_count: Some(4),
    }
}

fn get_plan_with_more_groups_than_teams() -> TournamentPlanPatch {
    TournamentPlanPatch {
        total_teams: Some(12),
        advancing_teams: Some(4),
        group_phase_rounds: Some(4),
        groups_count: Some(15),
    }
}
