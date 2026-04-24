use reqwest::{Client, Response};
use serde_json::json;
use sqlx::{query_scalar};
use sqlx::{Pool, Postgres};
use tau::setup::get_local_socket_addr;
use uuid::Uuid;

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

    Client::new()
        .post(format!(
            "http://{}/tournaments/{}/plan",
            get_local_socket_addr(),
            tournament_id
        ))
        .json(&plan_data)
        .bearer_auth(token)
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

pub fn calculate_final_phase_rounds(advancing_teams: i32) -> i32 {
    let mut teams = advancing_teams.clone();
    let mut final_phase_rounds = 0;
    if teams != 0 {
        while (teams & 1) == 0 {
            final_phase_rounds += 1;
            teams >>= 1;
        }
    }
    return final_phase_rounds;
}

pub fn calculate_final_phase_debates(advancing_teams: i32) -> i32 {
    let mut final_phase_debates = 1;
    let mut remaining_debates = advancing_teams / 2;
    while remaining_debates > 1 {
        final_phase_debates += remaining_debates;
        remaining_debates /= 2;
    }
    final_phase_debates
}

#[cfg(test)]
mod test_debates_calculation {
    use crate::common::plans_utils::calculate_final_phase_debates;
    #[test]
    fn test_finals_debates_calculation() {
        assert_eq!(calculate_final_phase_debates(32), 16 + 8 + 4 + 2 + 1);
        assert_eq!(calculate_final_phase_debates(16), 8 + 4 + 2 + 1);
        assert_eq!(calculate_final_phase_debates(8), 4 + 2 + 1);
        assert_eq!(calculate_final_phase_debates(4), 2 + 1);
    }
}

#[cfg(test)]
mod test_rounds_calculation {
    use crate::common::plans_utils::calculate_final_phase_rounds;
    #[test]
    fn test_finals_rounds_calculation() {
        assert_eq!(calculate_final_phase_rounds(32), 5);
        assert_eq!(calculate_final_phase_rounds(16), 4);
        assert_eq!(calculate_final_phase_rounds(8), 3);
        assert_eq!(calculate_final_phase_rounds(4), 2);
    }
}
