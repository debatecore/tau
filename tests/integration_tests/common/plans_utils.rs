use reqwest::{Client, Response, StatusCode};
use tau::setup::get_socket_addr;
use tau::tournaments::plans::TournamentPlan;
use serde_json::json;
use sqlx::{query_scalar, Row};
use uuid::Uuid;
use sqlx::{Postgres, Pool};

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

pub async fn count_plans(pool: &Pool<Postgres>, tournament_id: &str) -> i64 {
    let tournament_id = Uuid::parse_str(tournament_id).unwrap();

    query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM tournament_plans WHERE tournament_id = $1"#,
        tournament_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn count_phases(pool: &Pool<Postgres>, tournament_id: &str) -> i64 {
    let tournament_id = Uuid::parse_str(tournament_id).unwrap();

    query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM phases WHERE tournament_id = $1"#,
        tournament_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn count_rounds(pool: &Pool<Postgres>, tournament_id: &str) -> i64 {
    let tournament_id = Uuid::parse_str(tournament_id).unwrap();

    query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM rounds r
        INNER JOIN phases p ON p.id = r.phase_id
        WHERE p.tournament_id = $1
        "#,
        tournament_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn count_debates(pool: &Pool<Postgres>, tournament_id: &str) -> i64 {
    let tournament_id = Uuid::parse_str(tournament_id).unwrap();

    query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM debates d
        INNER JOIN rounds r ON r.id = d.round_id
        INNER JOIN phases p ON p.id = r.phase_id
        WHERE p.tournament_id = $1
        "#,
        tournament_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// The number of rounds is debate_tree_levels-1 or the number of zeros before the advancing_teams bit
// For example, advancing teams = 16 is the same as 00010000 in binary, so the number of rounds is 4
pub fn calculate_final_phase_rounds(advancing_teams: i32) -> i32 {
    let mut teams = advancing_teams.clone();
    assert!(teams > 0);
    let mut final_phase_rounds = 0;
    while (teams & 1) == 0 {
        final_phase_rounds+=1;
        teams >>= 1;
    }
    return final_phase_rounds
}

// The number of debates is the sum of every power of 2 up to 2^(advancing_teams-1)
pub fn calculate_final_phase_debates(advancing_teams: i32) -> i32 {
    let mut final_phase_debates = 1;
    let mut remaining_debates   = advancing_teams/2;
    while remaining_debates > 1 {
        final_phase_debates += remaining_debates;
        remaining_debates /= 2;
    }
    final_phase_debates
}