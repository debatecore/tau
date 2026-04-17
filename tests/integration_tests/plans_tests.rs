use reqwest::{Client, Response, StatusCode};
use std::future::IntoFuture;
use tau::setup::get_local_socket_addr;
use serial_test::serial;
use serde_json::json;
use tau::{omni_error::OmniError, setup};
use crate::common::{
    create_app, create_listener, prepare_empty_database,
    tournament_utils::get_id_of_a_new_tournament,
    plans_utils::{
        create_plan,
        count_debates,
        count_phases,
        count_plans,
        count_rounds,
        calculate_final_phase_rounds,
        calculate_final_phase_debates
    },
    user_utils::{
        get_organizer_token, 
        get_token_for_user_with_roles, 
        get_token_for_user_with_no_roles
    },
};
use uuid::Uuid;

const TEST_GROUP_PHASE_ROUNDS: i32 = 4;
const TEST_GROUPS_COUNT:       i32 = 8;
const TEST_ADVANCING_TEAMS:    i32 = 4;
const TEST_TOTAL_TEAMS:        i32 = 32;

const TEST_GROUP_PHASE_ROUNDS_PATCH: i32 = 5;
const TEST_GROUPS_COUNT_PATCH:       i32 = 10;
const TEST_ADVANCING_TEAMS_PATCH:    i32 = 4;
const TEST_TOTAL_TEAMS_PATCH:        i32 = 30;

fn expected_counts(group_phase_rounds: i32, groups_count: i32, advancing_teams: i32) -> (i64, i64, i64) {
    let phases  = 2;
    let rounds  = (group_phase_rounds+calculate_final_phase_rounds(advancing_teams)) as i64;
    let debates = (groups_count*group_phase_rounds + calculate_final_phase_debates(advancing_teams)) as i64;
    (phases, rounds, debates)
}

#[tokio::test]
#[serial]
async fn tournament_plan_creation_should_impossible_for_other_users() -> Result<(), OmniError>  {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let token = get_token_for_user_with_no_roles().await;
    let tournament_id = get_id_of_a_new_tournament("test").await?;

    // WHEN
    assert_eq!(
        create_plan(
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
#[serial]
async fn organizers_should_be_able_to_create_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    let pool = state.connection_pool.clone();
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;

    let response = create_plan(
        &tournament_id,
        TEST_ADVANCING_TEAMS,
        TEST_GROUP_PHASE_ROUNDS,
        TEST_GROUPS_COUNT,
        TEST_TOTAL_TEAMS,
        &token,
    ).await;

    let status = response.status();
    let body = response.text().await.unwrap();

    assert_eq!(status, StatusCode::OK);

    let (expected_phases, expected_rounds, expected_debates) =
        expected_counts(TEST_GROUP_PHASE_ROUNDS, TEST_GROUPS_COUNT, TEST_ADVANCING_TEAMS);

    assert_eq!(count_plans(&pool, &tournament_id).await, 1);
    assert_eq!(count_phases(&pool, &tournament_id).await, expected_phases);
    assert_eq!(count_rounds(&pool, &tournament_id).await, expected_rounds);
    assert_eq!(count_debates(&pool, &tournament_id).await, expected_debates);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_get_tournament_plan() -> Result<(), OmniError> {
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

    let create_response = create_plan(
        &tournament_id, 
        TEST_ADVANCING_TEAMS, 
        TEST_GROUP_PHASE_ROUNDS, 
        TEST_GROUPS_COUNT, 
        TEST_TOTAL_TEAMS,
        &token
    ).await;
    
    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    // WHEN
    let response = Client::new()
        .get(format!(
            "http://{}/tournaments/{}/plan/{}",
            get_local_socket_addr(), tournament_id, plan_id
        ))
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_patch_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    let pool = state.connection_pool.clone();
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;

    let create_response = create_plan(
        &tournament_id, 
        TEST_ADVANCING_TEAMS, 
        TEST_GROUP_PHASE_ROUNDS, 
        TEST_GROUPS_COUNT, 
        TEST_TOTAL_TEAMS,
        &token
    ).await;
    
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
    let response = Client::new()
        .patch(format!(
            "http://{}/tournaments/{}/plan/{}",
            get_local_socket_addr(), tournament_id, plan_id
        ))
        .json(&patch_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::OK);

    let (new_expected_phases, new_expected_rounds, new_expected_debates) = 
        expected_counts(TEST_GROUP_PHASE_ROUNDS_PATCH, TEST_GROUPS_COUNT_PATCH, TEST_ADVANCING_TEAMS_PATCH);

    assert_eq!(count_plans(&pool, &tournament_id).await, 1);
    assert_eq!(count_phases(&pool, &tournament_id).await, new_expected_phases);
    assert_eq!(count_rounds(&pool, &tournament_id).await, new_expected_rounds);
    assert_eq!(count_debates(&pool, &tournament_id).await, new_expected_debates);

    Ok(())
}

#[tokio::test]
#[serial]
async fn organizers_should_be_able_to_delete_tournament_plan() -> Result<(), OmniError> {
    // GIVEN
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    let pool = state.connection_pool.clone();
    prepare_empty_database(&state.connection_pool).await;
    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let token = get_organizer_token(&tournament_id).await;

    let create_response = create_plan(
        &tournament_id, 
        TEST_ADVANCING_TEAMS, 
        TEST_GROUP_PHASE_ROUNDS, 
        TEST_GROUPS_COUNT, 
        TEST_TOTAL_TEAMS,
        &token
    ).await;
    
    assert_eq!(create_response.status(), StatusCode::OK);

    let response_body = create_response.json::<serde_json::Value>().await.unwrap();
    let plan_id = response_body["id"].as_str().unwrap();

    // WHEN
    let response = Client::new()
        .delete(format!(
            "http://{}/tournaments/{}/plan/{}",
            get_local_socket_addr(), tournament_id, plan_id
        ))
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap();

    // THEN
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    assert_eq!(count_plans(&pool,   &tournament_id).await, 0);
    assert_eq!(count_phases(&pool,  &tournament_id).await, 0);
    assert_eq!(count_rounds(&pool,  &tournament_id).await, 0);
    assert_eq!(count_debates(&pool, &tournament_id).await, 0);

    Ok(())
}

#[tokio::test]
#[serial]
async fn create_plan_should_rollback_everything_if_underlying_creation_fails() -> Result<(), OmniError> {
    setup::read_environmental_variables();
    setup::check_secret_env_var();
    let state = setup::create_app_state().await;
    prepare_empty_database(&state.connection_pool).await;
    let pool = state.connection_pool.clone();

    let app = create_app(state).await;
    let listener = create_listener().await;
    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server);

    let tournament_id = get_id_of_a_new_tournament("test").await?;
    let tournament_uuid = Uuid::parse_str(&tournament_id).unwrap();

    let plan = tau::tournaments::plans::TournamentPlan {
        id: Uuid::now_v7(),
        tournament_id: tournament_uuid,
        group_phase_rounds: Some(TEST_GROUP_PHASE_ROUNDS),
        groups_count: Some(TEST_GROUPS_COUNT),
        advancing_teams: Some(TEST_ADVANCING_TEAMS),
        total_teams: Some(TEST_TOTAL_TEAMS),
    };

    let mut transaction = pool.begin().await.unwrap();

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

    assert_eq!(count_plans(&pool, &tournament_id).await, 0);
    assert_eq!(count_phases(&pool, &tournament_id).await, 0);
    assert_eq!(count_rounds(&pool, &tournament_id).await, 0);
    assert_eq!(count_debates(&pool, &tournament_id).await, 0);

    Ok(())
}