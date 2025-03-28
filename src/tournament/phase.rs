use std::fmt;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use sqlx::{query, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

use super::round::{Round, RoundStatus};

#[serde_inline_default]
#[derive(Serialize, Deserialize, ToSchema, Clone)]
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

#[derive(Deserialize, ToSchema, Clone)]
pub struct PhasePatch {
    pub name: Option<String>,
    pub tournament_id: Option<Uuid>,
    pub is_finals: Option<bool>,
    pub previous_phase_id: Option<Uuid>,
    pub group_size: Option<i32>,
    pub status: Option<PhaseStatus>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, PartialEq)]
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
        new_phase: Phase,
        pool: &Pool<Postgres>,
    ) -> Result<Phase, OmniError> {
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

    pub async fn get_rounds(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Round>, OmniError> {
        let mut rounds = vec![];
        match query!("SELECT * FROM rounds WHERE phase_id = $1", self.id)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => {
                for row in rows {
                    let round = Round {
                        id: row.id,
                        name: row.name,
                        phase_id: row.phase_id,
                        planned_start_time: row.planned_start_time,
                        planned_end_time: row.planned_end_time,
                        motion_id: row.motion_id,
                        previous_round_id: row.previous_round_id,
                        status: RoundStatus::try_from(row.status)?,
                    };
                    rounds.push(round);
                }
                Ok(rounds)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn validate(&self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        if self.is_finals && self.group_size.is_some() {
            return Err(OmniError::ExplicitError {
                status: StatusCode::BAD_REQUEST,
                message: "Group size cannot be defined for a finals phase".to_owned(),
            });
        }
        if self.phase_name_exists_in_tournament(pool).await? {
            return Err(OmniError::ResourceAlreadyExistsError);
        }
        if self
            .previous_phase_is_from_different_tournament(pool)
            .await?
        {
            return Err(OmniError::ExplicitError {
                status: StatusCode::CONFLICT,
                message: "Previous phase must be from the same tournament".to_owned(),
            });
        }
        if self
            .previous_phase_is_already_declared_as_previous_round_elsewhere(pool)
            .await?
        {
            return Err(OmniError::ExplicitError {
                status: StatusCode::CONFLICT,
                message: format!(
                    "Phase {} is already assigned as a previous phase elsewhere and therefore cannot be declared as a previous phase again",
                    self.previous_phase_id.unwrap()
                ).to_owned(),
            });
        }
        if self.phases_are_looped(pool).await? {
            return Err(OmniError::ExplicitError {
                status: StatusCode::CONFLICT,
                message: "Performing this operation would create a phase loop".to_owned(),
            });
        }
        if self.first_phases_are_doubled(pool).await? {
            return Err(OmniError::ExplicitError {
                status: StatusCode::CONFLICT,
                message: "Only one phase within a tournament can have previous_phase_id set to none".to_owned(),
            });
        }
        if self.status == PhaseStatus::Finished
            && self.some_rounds_are_not_finished(pool).await?
        {
            return Err(OmniError::ExplicitError {
                status: StatusCode::CONFLICT,
                message: "Some rounds are not finished. To finish this phase of the tournament, finish all the phases".to_owned(),
            });
        }
        Ok(())
    }

    pub async fn get_next_phase(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Phase, OmniError> {
        let next_phase_record = query!(
            "SELECT id FROM phases WHERE previous_phase_id = $1",
            self.id
        )
        .fetch_one(pool)
        .await
        .ok();
        if next_phase_record.is_none() {
            return Err(OmniError::ResourceNotFoundError);
        } else {
            let next_phase =
                Phase::get_by_id(next_phase_record.unwrap().id, pool).await?;
            return Ok(next_phase);
        }
    }

    pub async fn get_previous_phase(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Phase, OmniError> {
        if self.previous_phase_id.is_none() {
            return Err(OmniError::ResourceNotFoundError);
        }
        return Ok(Phase::get_by_id(self.previous_phase_id.unwrap(), pool).await?);
    }

    pub async fn phase_name_exists_in_tournament(
        &self,
        connection_pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        match query!(
            "SELECT EXISTS(SELECT 1 FROM phases WHERE name = $1 AND tournament_id = $2 AND id != $3)",
            self.name,
            self.tournament_id,
            self.id
        )
        .fetch_one(connection_pool)
        .await
        {
            Ok(result) => Ok(result.exists.unwrap()),
            Err(e) => Err(e)?,
        }
    }

    async fn previous_phase_is_from_different_tournament(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        if self.previous_phase_id.is_none() {
            return Ok(false);
        }
        let previous_phase =
            Phase::get_by_id(self.previous_phase_id.unwrap(), pool).await?;
        if previous_phase.tournament_id != self.tournament_id {
            return Ok(true);
        }
        return Ok(false);
    }

    async fn previous_phase_is_already_declared_as_previous_round_elsewhere(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        match query!(
            "SELECT EXISTS (SELECT 1 FROM phases WHERE previous_phase_id = $1)",
            self.previous_phase_id
        )
        .fetch_one(pool)
        .await
        {
            Ok(result) => Ok(result.exists.unwrap()),
            Err(e) => Err(e)?,
        }
    }

    async fn phases_are_looped(&self, pool: &Pool<Postgres>) -> Result<bool, OmniError> {
        let mut phase_ids: Vec<Uuid> = vec![];
        let mut previous_phase = self.get_previous_phase(pool).await;
        while previous_phase.is_ok() {
            let found_phase = previous_phase.unwrap();
            if phase_ids.contains(&found_phase.id) {
                return Ok(true);
            }
            phase_ids.push(found_phase.id);
            previous_phase = found_phase.get_previous_phase(pool).await;
        }

        if phase_ids.contains(&self.id) {
            return Ok(true);
        }
        phase_ids.push(self.id);

        let mut next_phase = self.get_next_phase(pool).await;
        while next_phase.is_ok() {
            let found_phase = next_phase.unwrap();
            if phase_ids.contains(&found_phase.id) {
                return Ok(true);
            }
            phase_ids.push(found_phase.id);
            next_phase = found_phase.get_previous_phase(pool).await;
        }
        Ok(false)
    }

    async fn first_phases_are_doubled(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        if self.previous_phase_id.is_some() {
            return Ok(false);
        } else {
            match query!("SELECT EXISTS (SELECT 1 FROM phases WHERE previous_phase_id is NULL AND tournament_id = $1)", self.tournament_id).fetch_one(pool).await {
                Ok(result) => Ok(result.exists.unwrap()),
                Err(e) => Err(e)?
            }
        }
    }

    async fn some_rounds_are_not_finished(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<bool, OmniError> {
        for round in self.get_rounds(pool).await? {
            if round.status != RoundStatus::Finished {
                return Ok(true);
            }
        }
        return Ok(false);
    }
}

impl PhasePatch {
    pub fn create_phase_with(self, phase: Phase) -> Phase {
        return Phase {
            id: phase.id,
            name: self.name.unwrap_or(phase.name),
            tournament_id: self.tournament_id.unwrap_or(phase.tournament_id),
            is_finals: self.is_finals.unwrap_or(phase.is_finals),
            previous_phase_id: self.previous_phase_id.or(phase.previous_phase_id),
            group_size: self.group_size.or(phase.group_size),
            status: self.status.unwrap_or(phase.status),
        };
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
