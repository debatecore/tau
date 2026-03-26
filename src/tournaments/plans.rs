use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Pool, Postgres, Transaction};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    omni_error::OmniError,
};

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
    // QUESTION: Do we need a tournament id here or as tournament always 
    // has only one config it should be embedded right into tournament struct?
    pub tournament_id:      Uuid,
    // Number of rounds for a single group for one phase
    pub group_phase_rounds: Option<i32>,
    // Number of groups of teams participating in tournament
    pub groups_count:       Option<i32>,
    // Number of teams that reached the final phase. Must be a power of 2
    pub advancing_teams:    Option<i32>,
    // Number of total teams participating in tournament.
    // QUESTION: As config (plan) is the most logical place to store this data and 
    // I didn't find any teams number in other files, I make an assumption that I can 
    // add the field, as it is important for tests
    pub total_teams:        Option<i32>
}

#[derive(Deserialize, ToSchema, sqlx::FromRow)]
/// TournamentPlanExternal can be used to patch a TournamentPlan without 
// changing important fields such as ID
// CHANGED: rename occured because we don't want to enter an ID and let it be handled externally
pub struct TournamentPlanExternal {
    // Number of rounds for a single group for one phase
    pub group_phase_rounds: Option<i32>,
    // Number of groups of teams participating in tournament
    pub groups_count:       Option<i32>,
    // Number of teams that reached the final phase. Must be a power of 2
    pub advancing_teams:    Option<i32>,
    // Number of total teams participating in tournament.
    pub total_teams:        Option<i32>
}

impl TournamentPlan {
    pub async fn post(
        tournament_id: Uuid,
        json: TournamentPlan,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentPlan, OmniError> {
        json.validate()?;

        let mut tx = pool.begin().await?;
        let plan = Self::post_tx(&mut tx, tournament_id, json).await?;
        tx.commit().await?;
        Ok(plan)
    }

    /// Transaction-aware create. Use this when part of a larger operation.
    pub async fn post_tx(
        tx: &mut Transaction<'_, Postgres>,
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
        .fetch_one(&mut **tx)
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
        patch: TournamentPlanExternal,
        pool: &Pool<Postgres>,
    ) -> Result<TournamentPlan, OmniError> {
        patch.validate()?;

        let mut tx = pool.begin().await?;
        let plan = self.patch_tx(&mut tx, patch).await?;
        tx.commit().await?;
        Ok(plan)
    }

    /// Transaction-aware patch.
    pub async fn patch_tx(
        self,
        tx: &mut Transaction<'_, Postgres>,
        patch: TournamentPlanExternal,
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
        .fetch_one(&mut **tx)
        .await?;

        Ok(updated)
    }

    /// Standalone delete.
    pub async fn delete(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        let mut tx = pool.begin().await?;
        self.delete_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Transaction-aware delete.
    pub async fn delete_tx(
        self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), OmniError> {
        query!(
            r#"DELETE FROM tournament_plans WHERE id = $1"#,
            self.id
        )
        .execute(&mut **tx)
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
}

impl TournamentPlanExternal {
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
    total_teams:        Option<i32>, 
    group_phase_rounds: Option<i32>, 
    groups_count:       Option<i32>, 
    advancing_teams:    Option<i32>, 
) -> Result<(), OmniError> {
    let total_teams = total_teams.ok_or_else(|| OmniError::ExplicitError {
        status: StatusCode::BAD_REQUEST,
        message: "total_teams must be set".to_owned(),
    })?;

    let group_phase_rounds = group_phase_rounds.ok_or_else(|| OmniError::ExplicitError {
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
            message: "Invalid tournament plan setup: all numbers should be positive".to_owned(),
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
            message: "Number of advancing teams should be a power of 2 (e.g. 4, 8, 16, 32 ...)".to_owned(),
        });
    }

    Ok(())
}