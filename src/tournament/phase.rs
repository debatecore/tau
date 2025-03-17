use std::fmt;

use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

#[serde_inline_default]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Phase {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub name: String,
    pub tournament_id: Uuid,
    pub is_finals: bool,
    pub previous_phase_id: Option<Uuid>,
    pub group_size: Option<i32>,
    pub status: PhaseStatus,
}

#[derive(Deserialize, ToSchema)]
pub struct PhasePatch {
    pub name: Option<String>,
    pub tournament_id: Option<Uuid>,
    pub is_finals: Option<bool>,
    pub previous_phase_id: Option<Uuid>,
    pub group_size: Option<i32>,
    pub status: Option<PhaseStatus>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum PhaseStatus {
    Planned,
    Ongoing,
    Finished,
}

impl Phase {
    pub async fn post(phase: Phase, pool: &Pool<Postgres>) -> Result<Phase, OmniError> {
        match query!(
            r#"INSERT INTO phases
            (id, name, tournament_id, is_finals, previous_phase_id, group_size, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, tournament_id, is_finals, previous_phase_id, group_size, status"#,
            phase.id,
            phase.name,
            phase.tournament_id,
            phase.is_finals,
            phase.previous_phase_id,
            phase.group_size,
            phase.status.to_string(),
        )
        .fetch_one(pool).await
        {
            Ok(record) => {
                let phase = Phase {
                    id: record.id,
                    name: record.name,
                    tournament_id: record.tournament_id,
                    is_finals: record.is_finals,
                    previous_phase_id: record.previous_phase_id,
                    group_size: record.group_size,
                    status: PhaseStatus::try_from(record.status)?,
                };
                Ok(phase)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(id: Uuid, pool: &Pool<Postgres>) -> Result<Phase, OmniError> {
        match query!("SELECT * FROM phases WHERE id = $1", id)
            .fetch_one(pool)
            .await
        {
            Ok(record) => {
                let phase = Phase {
                    id: record.id,
                    name: record.name,
                    tournament_id: record.tournament_id,
                    is_finals: record.is_finals,
                    previous_phase_id: record.previous_phase_id,
                    group_size: record.group_size,
                    status: PhaseStatus::try_from(record.status)?,
                };
                Ok(phase)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn patch(
        self,
        pool: &Pool<Postgres>,
        patch: PhasePatch,
    ) -> Result<Phase, OmniError> {
        let new_phase = Phase {
            id: self.id,
            name: patch.name.unwrap_or(self.name),
            tournament_id: patch.tournament_id.unwrap_or(self.tournament_id),
            is_finals: patch.is_finals.unwrap_or(self.is_finals),
            previous_phase_id: patch.previous_phase_id.or(self.previous_phase_id),
            group_size: patch.group_size.or(self.group_size),
            status: patch.status.unwrap_or(self.status),
        };
        match query!(
            "UPDATE phases SET name = $1, tournament_id = $2, is_finals = $3, previous_phase_id = $4, group_size = $5, status = $6 WHERE id = $7",
            new_phase.name,
            new_phase.tournament_id,
            new_phase.is_finals,
            new_phase.previous_phase_id,
            new_phase.group_size,
            new_phase.status.to_string(),
            new_phase.id,
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(new_phase),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM phases WHERE id = $1", self.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}

impl fmt::Display for PhaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhaseStatus::Planned => write!(f, "Planned"),
            PhaseStatus::Ongoing => write!(f, "Ongoing"),
            PhaseStatus::Finished => write!(f, "Finished"),
        }
    }
}

impl TryFrom<String> for PhaseStatus {
    type Error = OmniError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Planned" => Ok(PhaseStatus::Planned),
            "Ongoing" => Ok(PhaseStatus::Ongoing),
            "Finished" => Ok(PhaseStatus::Finished),
            _ => Err(OmniError::PhaseStatusParsingError),
        }
    }
}
