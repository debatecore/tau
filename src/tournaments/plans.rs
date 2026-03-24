use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, query_as, Pool, Postgres};
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
/// TournamentPlansPatch can be used to patch a TournamentPlan without 
// changing important fields such as ID ot Tournament ID
pub struct TournamentPlanPatch {
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
        tournament_plan: TournamentPlan,
        connection_pool: &Pool<Postgres>,
    ) -> Result<TournamentPlan, OmniError> {
        match query!(
            r#"INSERT INTO tournament_plans
            (id, tournament_id, group_phase_rounds, groups_count, advancing_teams, total_teams)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tournament_id, group_phase_rounds, groups_count, advancing_teams, total_teams"#,
            tournament_plan.id,
            tournament_plan.tournament_id,
            tournament_plan.group_phase_rounds,
            tournament_plan.groups_count,
            tournament_plan.advancing_teams,
            tournament_plan.total_teams
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(record) => {
                let plan = TournamentPlan {
                    id:                 record.id,
                    tournament_id:      record.tournament_id,
                    group_phase_rounds: record.group_phase_rounds,
                    groups_count:       record.groups_count,
                    advancing_teams:    record.advancing_teams,
                    total_teams:        record.total_teams,
                };
                Ok(plan)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(id: Uuid, connection_pool: &Pool<Postgres>, ) -> Result<TournamentPlan, OmniError> {
        match query!("SELECT * FROM tournament_plans WHERE id = $1", id)
            .fetch_one(connection_pool)
            .await
        {
            Ok(record) => {
                let plan = TournamentPlan {
                    id:                 record.id,
                    tournament_id:      record.tournament_id,
                    group_phase_rounds: record.group_phase_rounds,
                    groups_count:       record.groups_count,
                    advancing_teams:    record.advancing_teams,
                    total_teams:        record.total_teams
                };
                Ok(plan)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(self, patch: TournamentPlanPatch, connection_pool: &Pool<Postgres>, ) -> Result<TournamentPlan, OmniError> {
        match query!(
            r#"
                UPDATE tournament_plans SET 
                   group_phase_rounds = $1, 
                   groups_count = $2, 
                   advancing_teams = $3, 
                   total_teams = $4 
                WHERE id = $5
            "#,
            patch.group_phase_rounds,
            patch.groups_count,
            patch.advancing_teams,
            patch.total_teams,
            self.id
        )
        .execute(connection_pool)
        .await
        {
            Ok(record) => {
                // Return an updated plan in case of success
                let plan = TournamentPlan {
                    id:                 self.id,
                    tournament_id:      self.tournament_id,
                    group_phase_rounds: patch.group_phase_rounds,
                    groups_count:       patch.groups_count,
                    advancing_teams:    patch.advancing_teams,
                    total_teams:        patch.total_teams
                };
                Ok(plan)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, connection_pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!(
            r"DELETE FROM tournament_plans WHERE id = $1",
            self.id
        )
        .execute(connection_pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }

    // QUESTION: just reject or try to substitute numbers? design so it's impossible to mess something up?
    pub async fn validate(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        if self.total_teams <= Some(1) || self.group_phase_rounds <= Some(0) || self.groups_count <= Some(0) {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Invalid tournament plan setup: all numbers should be positive".to_owned(),
            });
        }
        
        if let (Some(total_teams), Some(group_phase_rounds)) =
            (self.total_teams, self.group_phase_rounds)
        {
            if group_phase_rounds == 0 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "Group phase rounds cannot be zero".to_owned(),
                });
            }

            if total_teams % group_phase_rounds != 0 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "total_teams must be divisible by group_phase_rounds".to_owned(),
                });
            }
        } else {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Group phase rounds and total teams cannot be zero".to_owned(),
            });
        }
        
        if self.total_teams <= self.groups_count {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Number of groups cannot be higher than or equal to the total number of teams".to_owned(),
            });
        }

        if self.total_teams <= self.advancing_teams {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Number of advancing teams cannot be higher than or equal to the total number of teams".to_owned(),
            });
        }

        if let Some(advancing_teams) = self.advancing_teams {
            if advancing_teams <= 1 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "advancing_teams must be greater than 1".to_owned(),
                });
            }

            if (advancing_teams & (advancing_teams - 1)) != 0 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "Number of advancing teams should be equal to the power of 2 (e.g. 4, 16, 32 ...)".to_owned(),
                });
            }
        } else {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "advancing_teams must be set".to_owned(),
            });
        }

        Ok(())
    }
}

// Should be one function actually
impl TournamentPlanPatch {
    pub async fn validate(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> { // just reject or try to substitute numbers? design so it's impossible to mess something up?
        if self.total_teams <= Some(1) || self.group_phase_rounds <= Some(0) || self.groups_count <= Some(0) {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Invalid tournament plan setup: all numbers should be positive".to_owned(),
            });
        }
        
        if let (Some(total_teams), Some(group_phase_rounds)) =
            (self.total_teams, self.group_phase_rounds)
        {
            if group_phase_rounds == 0 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "Group phase rounds cannot be zero".to_owned(),
                });
            }

            if total_teams % group_phase_rounds != 0 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "total_teams must be divisible by group_phase_rounds".to_owned(),
                });
            }
        } else {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Group phase rounds and total teams cannot be zero".to_owned(),
            });
        }
        
        if self.total_teams <= self.groups_count {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Number of groups cannot be higher than or equal to the total number of teams".to_owned(),
            });
        }

        if self.total_teams <= self.advancing_teams {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Number of advancing teams cannot be higher than or equal to the total number of teams".to_owned(),
            });
        }

        if let Some(advancing_teams) = self.advancing_teams {
            if advancing_teams <= 1 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "advancing_teams must be greater than 1".to_owned(),
                });
            }

            if (advancing_teams & (advancing_teams - 1)) != 0 {
                return Err(OmniError::ExplicitError {
                    status: StatusCode::BAD_REQUEST,
                    message: "Number of advancing teams should be equal to the power of 2 (e.g. 4, 16, 32 ...)".to_owned(),
                });
            }
        } else {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "advancing_teams must be set".to_owned(),
            });
        }

        Ok(())
    }
}