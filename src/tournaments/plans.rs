use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Pool, Postgres, Transaction};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::tournaments::{
    debates::Debate,
    phases::{Phase, PhaseStatus},
    rounds::{Round, RoundStatus},
};

use crate::omni_error::OmniError;

#[serde_inline_default]
#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
#[serde(deny_unknown_fields)]
/// TournamentPlans can be used to plan a tournament setting up
/// group phase rounds, groups count and advancing teams.
pub struct TournamentPlan {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    // Tournament ID for a particular plan
    pub tournament_id: Uuid,
    // Number of rounds for a single group for one phase
    pub group_phase_rounds: Option<i32>,
    // Number of groups of teams participating in tournament
    pub groups_count: Option<i32>,
    // Number of teams that reached the final phase. Must be a power of 2
    pub advancing_teams: Option<i32>,
    // Number of total teams participating in tournament.
    pub total_teams: Option<i32>,
}

#[derive(Deserialize, ToSchema, sqlx::FromRow)]
/// TournamentPlanPatch can be used to patch a TournamentPlan without
// changing important fields such as ID
// CHANGED: rename occured because we don't want to enter an ID and let it be handled externally
pub struct TournamentPlanPatch {
    // Number of rounds for a single group for one phase
    pub group_phase_rounds: Option<i32>,
    // Number of groups of teams participating in tournament
    pub groups_count: Option<i32>,
    // Number of teams that reached the final phase. Must be a power of 2
    pub advancing_teams: Option<i32>,
    // Number of total teams participating in tournament.
    pub total_teams: Option<i32>,
}

impl TournamentPlan {
    pub async fn post(
        tournament_id: Uuid,
        json: TournamentPlan,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentPlan, OmniError> {
        json.validate()?;

        let mut transaction = pool.begin().await?;
        let plan =
            Self::post_with_transaction(&mut transaction, tournament_id, json).await?;
        transaction.commit().await?;
        Ok(plan)
    }

    /// Transaction-aware create. Use this when part of a larger operation.
    pub async fn post_with_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        tournament_id: Uuid,
        json: TournamentPlan,
    ) -> Result<TournamentPlan, OmniError> {
        json.validate()?;

        let plan = query_as!(
            TournamentPlan,
            r#"
            INSERT INTO tournament_plans
                (id, tournament_id, group_phase_rounds, groups_count, advancing_teams, total_teams)
            VALUES
                ($1, $2, $3, $4, $5, $6)
            RETURNING
                id, tournament_id, group_phase_rounds, groups_count, advancing_teams, total_teams
            "#,
            json.id,
            tournament_id,
            json.group_phase_rounds,
            json.groups_count,
            json.advancing_teams,
            json.total_teams
        )
        .fetch_one(&mut **transaction)
        .await?;

        plan.post_underlying_structs_with_transaction(transaction)
            .await?;

        Ok(plan)
    }

    pub async fn get_by_id(
        id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentPlan, OmniError> {
        let plan = query_as!(
            TournamentPlan,
            r#"
            SELECT
                id, tournament_id, group_phase_rounds, groups_count, advancing_teams, total_teams
            FROM tournament_plans
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(plan)
    }

    /// Standalone patch.
    pub async fn patch(
        self,
        patch: TournamentPlanPatch,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentPlan, OmniError> {
        patch.validate()?;

        let mut transaction = pool.begin().await?;
        let plan = self.patch_with_transaction(&mut transaction, patch).await?;
        transaction.commit().await?;
        Ok(plan)
    }

    /// Transaction-aware patch.
    pub async fn patch_with_transaction(
        self,
        transaction: &mut Transaction<'_, Postgres>,
        patch: TournamentPlanPatch,
    ) -> Result<TournamentPlan, OmniError> {
        patch.validate()?;

        let updated = query_as!(
            TournamentPlan,
            r#"
            UPDATE tournament_plans
            SET
                group_phase_rounds = $1,
                groups_count = $2,
                advancing_teams = $3,
                total_teams = $4
            WHERE id = $5
            RETURNING
                id, tournament_id, group_phase_rounds, groups_count, advancing_teams, total_teams
            "#,
            patch.group_phase_rounds,
            patch.groups_count,
            patch.advancing_teams,
            patch.total_teams,
            self.id
        )
        .fetch_one(&mut **transaction)
        .await?;

        updated
            .delete_underlying_structs_with_transaction(transaction)
            .await?;
        updated
            .post_underlying_structs_with_transaction(transaction)
            .await?;

        Ok(updated)
    }

    /// Standalone delete.
    pub async fn delete(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        let mut transaction = pool.begin().await?;
        self.delete_with_transaction(&mut transaction).await?;
        transaction.commit().await?;
        Ok(())
    }

    /// Transaction-aware delete.
    pub async fn delete_with_transaction(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), OmniError> {
        self.delete_underlying_structs_with_transaction(transaction)
            .await?;

        query!(r#"DELETE FROM tournament_plans WHERE id = $1"#, self.id)
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub fn validate(&self) -> Result<(), OmniError> {
        validate(
            self.total_teams,
            self.group_phase_rounds,
            self.groups_count,
            self.advancing_teams,
        )
    }

    fn validated_values(&self) -> Result<(i32, i32, i32, i32), OmniError> {
        Ok((
            self.total_teams.ok_or_else(|| OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "total_teams must be set".to_owned(),
            })?,
            self.group_phase_rounds
                .ok_or_else(|| OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "group_phase_rounds must be set".to_owned(),
                })?,
            self.groups_count.ok_or_else(|| OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "groups_count must be set".to_owned(),
            })?,
            self.advancing_teams
                .ok_or_else(|| OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "advancing_teams must be set".to_owned(),
                })?,
        ))
    }

    async fn post_underlying_structs_with_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), OmniError> {
        let (total_teams, group_phase_rounds, groups_count, _advancing_teams) =
            self.validated_values()?;

        // Phase
        let mut previous_phase_id: Option<Uuid> = None;

        for phase_index in 1..=2 {
            let curr_phase_id = Uuid::now_v7();
            let is_finals = phase_index == 2; // is phase count constant (group phase / final phase)?

            Phase::post_with_transaction(
                transaction,
                self.tournament_id,
                Phase {
                    id: curr_phase_id,
                    name: format!("phase_{phase_index}"),
                    tournament_id: self.tournament_id,
                    is_finals,
                    previous_phase_id,
                    group_size: Some(total_teams / groups_count),
                    status: PhaseStatus::Planned,
                },
            )
            .await?;

            let mut previous_round_id: Option<Uuid> = None;

            // Group phase
            if !is_finals {
                // Rounds
                for round_index in 1..=group_phase_rounds {
                    let curr_round_id = Uuid::now_v7();
                    Round::post_with_transaction(
                        transaction,
                        Round {
                            id: curr_round_id,
                            name: format!("round_{round_index}"),
                            phase_id: curr_phase_id,
                            planned_start_time: None,
                            planned_end_time: None,
                            motion_id: None,
                            previous_round_id: previous_round_id,
                            status: RoundStatus::Planned,
                        },
                    )
                    .await?;

                    // Debates
                    for _ in 1..=groups_count {
                        Debate::post_with_transaction(
                            transaction,
                            self.tournament_id,
                            Debate {
                                id: Uuid::now_v7(),
                                motion_id: None,
                                marshal_user_id: None,
                                tournament_id: self.tournament_id,
                                round_id: curr_round_id,
                            },
                        )
                        .await?;
                    }

                    previous_round_id = Some(curr_round_id);
                }
            // Finals phase
            } else {
                let mut remaining_teams = _advancing_teams / 2;
                // Rounds
                for round_index in 1..=calculate_final_phase_rounds(_advancing_teams) {
                    let curr_round_id = Uuid::now_v7();
                    Round::post_with_transaction(
                        transaction,
                        Round {
                            id: curr_round_id,
                            name: format!("round_{round_index}"),
                            phase_id: curr_phase_id,
                            planned_start_time: None,
                            planned_end_time: None,
                            motion_id: None,
                            previous_round_id: previous_round_id,
                            status: RoundStatus::Planned,
                        },
                    )
                    .await?;

                    // Debates
                    for _ in 1..=remaining_teams {
                        Debate::post_with_transaction(
                            transaction,
                            self.tournament_id,
                            Debate {
                                id: Uuid::now_v7(),
                                motion_id: None,
                                marshal_user_id: None,
                                tournament_id: self.tournament_id,
                                round_id: curr_round_id,
                            },
                        )
                        .await?;
                    }

                    remaining_teams /= 2;
                    previous_round_id = Some(curr_round_id);
                }
            }

            previous_phase_id = Some(curr_phase_id);
        }

        Ok(())
    }

    async fn delete_underlying_structs_with_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), OmniError> {
        query!(
            r#"
            DELETE FROM debates
            WHERE round_id IN (
                SELECT r.id
                FROM rounds r
                INNER JOIN phases p ON p.id = r.phase_id
                WHERE p.tournament_id = $1
            )
            "#,
            self.tournament_id
        )
        .execute(&mut **transaction)
        .await?;

        query!(
            r#"
            DELETE FROM rounds
            WHERE phase_id IN (
                SELECT id
                FROM phases
                WHERE tournament_id = $1
            )
            "#,
            self.tournament_id
        )
        .execute(&mut **transaction)
        .await?;

        query!(
            r#"
            DELETE FROM phases
            WHERE tournament_id = $1
            "#,
            self.tournament_id
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

impl TournamentPlanPatch {
    pub fn validate(&self) -> Result<(), OmniError> {
        validate(
            self.total_teams,
            self.group_phase_rounds,
            self.groups_count,
            self.advancing_teams,
        )
    }
}

fn validate(
    total_teams: Option<i32>,
    group_phase_rounds: Option<i32>,
    groups_count: Option<i32>,
    advancing_teams: Option<i32>,
) -> Result<(), OmniError> {
    let total_teams = total_teams.ok_or_else(|| OmniError::ExplicitError {
        status: StatusCode::BAD_REQUEST,
        message: "total_teams must be set".to_owned(),
    })?;

    let group_phase_rounds =
        group_phase_rounds.ok_or_else(|| OmniError::ExplicitError {
            status: StatusCode::BAD_REQUEST,
            message: "group_phase_rounds must be set".to_owned(),
        })?;

    let groups_count = groups_count.ok_or_else(|| OmniError::ExplicitError {
        status: StatusCode::BAD_REQUEST,
        message: "groups_count must be set".to_owned(),
    })?;

    let advancing_teams = advancing_teams.ok_or_else(|| OmniError::ExplicitError {
        status: StatusCode::BAD_REQUEST,
        message: "advancing_teams must be set".to_owned(),
    })?;

    if total_teams <= 1 || group_phase_rounds <= 0 || groups_count <= 0 {
        return Err(OmniError::ExplicitError {
            status: StatusCode::BAD_REQUEST,
            message: "Invalid tournament plan setup: all numbers should be positive"
                .to_owned(),
        });
    }

    if total_teams % group_phase_rounds != 0 {
        return Err(OmniError::ExplicitError {
            status: StatusCode::BAD_REQUEST,
            message: "total_teams must be divisible by group_phase_rounds".to_owned(),
        });
    }

    if groups_count >= total_teams {
        return Err(OmniError::ExplicitError {
            status: StatusCode::BAD_REQUEST,
            message: "Number of groups cannot be higher than or equal to the total number of teams".to_owned(),
        });
    }

    if advancing_teams >= total_teams {
        return Err(OmniError::ExplicitError {
            status: StatusCode::BAD_REQUEST,
            message: "Number of advancing teams cannot be higher than or equal to the total number of teams".to_owned(),
        });
    }

    if advancing_teams <= 0 || (advancing_teams & (advancing_teams - 1)) != 0 {
        return Err(OmniError::ExplicitError {
            status: StatusCode::BAD_REQUEST,
            message:
                "Number of advancing teams should be a power of 2 (e.g. 4, 8, 16, 32 ...)"
                    .to_owned(),
        });
    }

    Ok(())
}

fn calculate_final_phase_rounds(advancing_teams: i32) -> i32 {
    let mut teams = advancing_teams.clone();
    let mut final_phase_rounds = 0;
    if (teams != 0) {
        while (teams & 1) == 0 {
            final_phase_rounds += 1;
            teams >>= 1;
        }
    }
    final_phase_rounds
}
