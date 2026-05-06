use std::collections::HashSet;

use reqwest::StatusCode;
use tau::omni_error::OmniError;

use crate::common::{
    plans_utils::{
        calculate_final_phase_debates, calculate_final_phase_rounds, create_plan,
    },
    test_app::TestApp,
    tournament_utils::get_id_of_a_new_tournament,
    user_utils::get_organizer_token,
};

const TEST_GROUP_PHASE_ROUNDS: i32 = 4;
const TEST_GROUPS_COUNT: i32 = 8;
const TEST_ADVANCING_TEAMS: i32 = 4;
const TEST_TOTAL_TEAMS: i32 = 32;

#[tokio::test]
async fn ladder_should_return_consistent_phases_rounds_and_debates(
) -> Result<(), OmniError> {
    let app = TestApp::spawn().await;
    let tournament_id = get_id_of_a_new_tournament(&app, "ladder").await?;
    let token = get_organizer_token(&app, &tournament_id).await;

    let plan_response = create_plan(
        &app,
        &tournament_id,
        TEST_ADVANCING_TEAMS,
        TEST_GROUP_PHASE_ROUNDS,
        TEST_GROUPS_COUNT,
        TEST_TOTAL_TEAMS,
        &token,
    )
    .await;
    assert_eq!(plan_response.status(), StatusCode::OK);

    let ladder_response = app
        .client
        .get(app.url(&format!("/tournaments/{}/ladder", tournament_id)))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    assert_eq!(ladder_response.status(), StatusCode::OK);
    let body = ladder_response.json::<serde_json::Value>().await.unwrap();

    let phases = body["phases"]
        .as_array()
        .expect("phases should be an array");
    let rounds = body["rounds"]
        .as_array()
        .expect("rounds should be an array");
    let debates = body["debates"]
        .as_array()
        .expect("debates should be an array");

    let expected_phase_count = 2usize;
    let expected_round_count = (TEST_GROUP_PHASE_ROUNDS
        + calculate_final_phase_rounds(TEST_ADVANCING_TEAMS))
        as usize;
    let expected_debate_count = (TEST_GROUPS_COUNT * TEST_GROUP_PHASE_ROUNDS
        + calculate_final_phase_debates(TEST_ADVANCING_TEAMS))
        as usize;

    assert_eq!(phases.len(), expected_phase_count);
    assert_eq!(rounds.len(), expected_round_count);
    assert_eq!(debates.len(), expected_debate_count);

    let phase_ids: HashSet<&str> = phases
        .iter()
        .map(|phase| {
            phase["id"]
                .as_str()
                .expect("every phase should have string id")
        })
        .collect();

    let round_ids: HashSet<&str> = rounds
        .iter()
        .map(|round| {
            round["id"]
                .as_str()
                .expect("every round should have string id")
        })
        .collect();

    for round in rounds {
        let parent_phase_id = round["phase_id"]
            .as_str()
            .expect("every round should have phase_id");
        assert!(
            phase_ids.contains(parent_phase_id),
            "round refers to missing phase_id {parent_phase_id}"
        );
    }

    for debate in debates {
        let parent_round_id = debate["round_id"]
            .as_str()
            .expect("every debate should have round_id");
        assert!(
            round_ids.contains(parent_round_id),
            "debate refers to missing round_id {parent_round_id}"
        );
    }

    Ok(())
}
