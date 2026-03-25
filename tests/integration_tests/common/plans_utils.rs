use reqwest::{Client, Response, StatusCode};
use tau::setup::get_socket_addr;
use tau::tournaments::plans::TournamentPlan;
use serde_json::json;

pub async fn create_plan(
    tournament_id: &str,
    advancing_teams: i32,
    group_phase_rounds: i32,
    groups_count: i32,
    total_teams: i32,
    token: &str,
) -> Response {
    let plan_data = json!({
        "tournament_id": tournament_id,
        "group_phase_rounds": group_phase_rounds,
        "groups_count": groups_count,
        "advancing_teams": advancing_teams,
        "total_teams": total_teams,
    });

    // WHEN
    Client::new()
        .post(format!(
            "http://{}/tournaments/{}/plan",
            get_socket_addr(), tournament_id
        ))
        .json(&plan_data)
        .bearer_auth(token.clone())
        .send()
        .await
        .unwrap()
}