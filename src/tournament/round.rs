use std::fmt;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{omni_error::OmniError, tournament::phase::Phase};

#[serde_inline_default]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Round {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub name: String,
    pub phase_id: Uuid,
    pub planned_start_time: Option<DateTime<Utc>>,
    pub planned_end_time: Option<DateTime<Utc>>,
    pub motion_id: Option<Uuid>,
    pub previous_round_id: Option<Uuid>,
    pub status: RoundStatus,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RoundPatch {
    pub name: Option<String>,
    pub phase_id: Option<Uuid>,
    pub planned_start_time: Option<DateTime<Utc>>,
    pub planned_end_time: Option<DateTime<Utc>>,
    pub motion_id: Option<Uuid>,
    pub previous_round_id: Option<Uuid>,
    pub status: Option<RoundStatus>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum RoundStatus {
    Planned,
    Ongoing,
    Finished,
}

impl Round {
    pub async fn post(round: Round, pool: &Pool<Postgres>) -> Result<Round, OmniError> {
        match query!(
            r#"INSERT INTO rounds
            (id, name, phase_id, planned_start_time, planned_end_time, motion_id, previous_round_id, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, phase_id, planned_start_time, planned_end_time, motion_id, previous_round_id, status"#,
            round.id,
            round.name,
            round.phase_id,
            round.planned_start_time,
            round.planned_end_time,
            round.motion_id,
            round.previous_round_id,
            round.status.to_string(),
        )
        .fetch_one(pool)
        .await
        {
            Ok(record) => {
                let round = Round {
                    id: record.id,
                    name: record.name,
                    phase_id: record.phase_id,
                    planned_start_time: record.planned_start_time,
                    planned_end_time: record.planned_end_time,
                    motion_id: record.motion_id,
                    previous_round_id: record.previous_round_id,
                    status: RoundStatus::try_from(record.status)?,
                };
                Ok(round)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(id: Uuid, pool: &Pool<Postgres>) -> Result<Round, OmniError> {
        match query!("SELECT * FROM rounds WHERE id = $1", id)
            .fetch_one(pool)
            .await
        {
            Ok(record) => {
                let round = Round {
                    id: record.id,
                    name: record.name,
                    phase_id: record.phase_id,
                    planned_start_time: record.planned_start_time,
                    planned_end_time: record.planned_end_time,
                    motion_id: record.motion_id,
                    previous_round_id: record.previous_round_id,
                    status: RoundStatus::try_from(record.status)?,
                };
                Ok(round)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        patch: RoundPatch,
        pool: &Pool<Postgres>,
    ) -> Result<Round, OmniError> {
        let new_round = Round {
            id: self.id,
            name: patch.name.unwrap_or(self.name),
            phase_id: patch.phase_id.unwrap_or(self.phase_id),
            planned_start_time: patch.planned_start_time.or(self.planned_start_time),
            planned_end_time: patch.planned_end_time.or(self.planned_end_time),
            motion_id: patch.motion_id.or(self.motion_id),
            previous_round_id: patch.previous_round_id.or(self.previous_round_id),
            status: patch.status.unwrap_or(self.status),
        };
        match query!(
            r#"
                UPDATE rounds SET name = $1, phase_id = $2, planned_start_time = $3,
                planned_end_time = $4, motion_id = $5, previous_round_id = $6,
                status = $7 WHERE id = $8
            "#,
            new_round.name,
            new_round.phase_id,
            new_round.planned_start_time,
            new_round.planned_end_time,
            new_round.motion_id,
            new_round.previous_round_id,
            new_round.status.to_string(),
            new_round.id,
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(new_round),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM rounds WHERE id = $1", self.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }

    pub async fn validate(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        if self.planned_start_time > self.planned_end_time {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "End time cannot occur before start time".to_owned(),
            });
        }
        if self
            .previous_round_is_not_from_the_same_or_previous_phase(pool)
            .await?
        {
            return Err(OmniError::ExplicitError {
                status: StatusCode::CONFLICT,
                message: "Previous round can only be from the same or previous phase"
                    .to_owned(),
            });
        }
        if self
            .previous_round_is_already_declared_as_previous_round_elsewhere(pool)
            .await?
        {
            return Err(OmniError::ExplicitError {
                status: StatusCode::CONFLICT,
                message: format!(
                    "Round {} is already assigned as a previous round elsewhere and therefore cannot be declared as a previous round again",
                    self.previous_round_id.unwrap()
                ).to_owned()
            });
        }
        Ok(())
    }

    async fn previous_round_is_not_from_the_same_or_previous_phase(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        if self.previous_round_id.is_none() {
            return Ok(false);
        }
        let previous_round =
            Round::get_by_id(self.previous_round_id.unwrap(), pool).await?;
        let phase_of_previous_round_id = previous_round.phase_id;
        if self.phase_id == phase_of_previous_round_id {
            return Ok(false);
        }
        let phase = Phase::get_by_id(self.phase_id, pool).await?;
        if phase.previous_phase_id.is_none() {
            return Ok(false);
        }
        if phase.previous_phase_id.unwrap() != phase_of_previous_round_id {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    async fn previous_round_is_already_declared_as_previous_round_elsewhere(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        match query!(
            "SELECT EXISTS (SELECT 1 FROM rounds WHERE previous_round_id = $1)",
            self.previous_round_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(result) => Ok(result.exists.unwrap()),
            Err(e) => Err(e)?,
        }
    }
}

impl fmt::Display for RoundStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RoundStatus::Planned => write!(f, "Planned"),
            RoundStatus::Ongoing => write!(f, "Ongoing"),
            RoundStatus::Finished => write!(f, "Finished"),
        }
    }
}

impl TryFrom<String> for RoundStatus {
    type Error = OmniError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Planned" => Ok(RoundStatus::Planned),
            "Ongoing" => Ok(RoundStatus::Ongoing),
            "Finished" => Ok(RoundStatus::Finished),
            _ => Err(OmniError::PhaseStatusParsingError),
        }
    }
}
