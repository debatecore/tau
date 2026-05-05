use debates::Debate;
use locations::Location;
use phases::{Phase, PhaseStatus};
use plans::TournamentPlan;
use rounds::Round;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use teams::Team;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::omni_error::OmniError;

pub(crate) mod affiliations;
pub(crate) mod attendees;
pub(crate) mod debates;
pub(crate) mod locations;
pub(crate) mod motions;
pub(crate) mod phases;
pub mod plans;
pub mod roles;
pub(crate) mod rooms;
pub(crate) mod rounds;
pub(crate) mod teams;
pub mod verdicts;

static DEFAULT_SPEECH_TIME: i32 = 300;
static DEFAULT_END_PROTECTED_TIME: i32 = 30;
static DEFAULT_START_PROTECTED_TIME: i32 = 0;
static DEFAULT_AD_VOCEM_TIME: i32 = 60;
static DEFAULT_DEBATE_TIME_SLOT: i32 = 120;
static DEFAULT_DEBATE_PREPARATION_TIME: i32 = 15;
static DEFAULT_BEEP_ON_SPEECH_END: bool = true;
static DEFAULT_BEEP_ON_PROTECTED_TIME: bool = true;
static DEFAULT_VISUALIZE_PROTECTED_TIME: bool = false;

/// Represents a tournament.
///
/// TO-DO: Rules defined here will be applied to debates started by Marshals.
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Tournament {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    /// Full name of the tournament. Must be unique.
    full_name: String,
    shortened_name: String,
    /// Speech time per speaker during a debate in seconds.
    speech_time: Option<i32>,
    /// Indicates how much time (in seconds) at the end of a speech will be protected time
    /// during which new questions and points of information do not
    /// have to be answered
    end_protected_time: Option<i32>,
    /// Indicates how much time (in seconds) at the start of a speech will be protected time
    /// during which new questions and points of information do not
    /// have to be answered
    start_protected_time: Option<i32>,
    /// Time of a separate ad vocem speech in seconds
    ad_vocem_time: Option<i32>,
    /// In minutes. Indicates how long is the debate expected to last.
    /// TO-DO: A debate scheduled at a particular room will block the room for this time.
    debate_time_slot: Option<i32>,
    /// In minutes. Indicates how much time
    /// should the teams have to prepare, once the sides are drawn
    debate_preparation_time: Option<i32>,
    // Indicates whether to make a sound effect on speech end
    beep_on_speech_end: Option<bool>,
    // Indicates whether to make a sound effect on protected time start/end
    beep_on_protected_time: Option<bool>,
    // Indicates whether to represent protected time visually or not
    visualize_protected_time: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct TournamentPatch {
    /// Full name of the tournament. Must be unique.
    full_name: Option<String>,
    shortened_name: Option<String>,
    /// In seconds
    /// Speech time per speaker during a debate in seconds.
    speech_time: Option<i32>,
    /// Indicates how much time (in seconds) at the end of a speech will be protected time
    /// during which new questions and points of information do not
    /// have to be answered
    end_protected_time: Option<i32>,
    /// Indicates how much time (in seconds) at the start of a speech will be protected time
    /// during which new questions and points of information do not
    /// have to be answered
    start_protected_time: Option<i32>,
    /// Time of a separate ad vocem speech in seconds
    ad_vocem_time: Option<i32>,
    /// In minutes. Indicates how long is the debate expected to last.
    /// TO-DO: A debate scheduled at a particular room will block the room for this time.
    debate_time_slot: Option<i32>,
    /// In minutes. Indicates how much time
    /// should the teams have to prepare, once the sides are drawn
    debate_preparation_time: Option<i32>,
    // Indicates whether to make a sound effect on speech end
    beep_on_speech_end: Option<bool>,
    // Indicates whether to make a sound effect on protected time start/end
    beep_on_protected_time: Option<bool>,
    // Indicates whether to represent protected time visually or not
    visualize_protected_time: Option<bool>,
}

impl Tournament {
    pub async fn post(
        tournament: Tournament,
        pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(
            Tournament,
            r#"INSERT INTO tournaments
            (
                id,
                full_name,
                shortened_name,
                speech_time,
                end_protected_time,
                start_protected_time,
                ad_vocem_time,
                debate_time_slot,
                debate_preparation_time,
                beep_on_speech_end,
                beep_on_protected_time,
                visualize_protected_time
            )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING
                id,
                full_name,
                shortened_name,
                speech_time,
                end_protected_time,
                start_protected_time,
                ad_vocem_time,
                debate_time_slot,
                debate_preparation_time,
                beep_on_speech_end,
                beep_on_protected_time,
                visualize_protected_time"#,
            tournament.id,
            tournament.full_name,
            tournament.shortened_name,
            tournament.speech_time.unwrap_or(DEFAULT_SPEECH_TIME),
            tournament
                .end_protected_time
                .unwrap_or(DEFAULT_END_PROTECTED_TIME),
            tournament
                .start_protected_time
                .unwrap_or(DEFAULT_START_PROTECTED_TIME),
            tournament.ad_vocem_time.unwrap_or(DEFAULT_AD_VOCEM_TIME),
            tournament
                .debate_time_slot
                .unwrap_or(DEFAULT_DEBATE_TIME_SLOT),
            tournament
                .debate_preparation_time
                .unwrap_or(DEFAULT_DEBATE_PREPARATION_TIME),
            tournament
                .beep_on_speech_end
                .unwrap_or(DEFAULT_BEEP_ON_SPEECH_END),
            tournament
                .beep_on_protected_time
                .unwrap_or(DEFAULT_BEEP_ON_PROTECTED_TIME),
            tournament
                .visualize_protected_time
                .unwrap_or(DEFAULT_VISUALIZE_PROTECTED_TIME)
        )
        .fetch_one(pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<Tournament>, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments")
            .fetch_all(pool)
            .await
        {
            Ok(tournaments) => Ok(tournaments),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_by_id(
        id: Uuid,
        pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        match query_as!(Tournament, "SELECT * FROM tournaments WHERE id = $1", id)
            .fetch_one(pool)
            .await
        {
            Ok(tournament) => Ok(tournament),
            Err(e) => {
                error!("Error getting a tournament with id {id}: {e}");
                Err(e)?
            }
        }
    }

    pub async fn patch(
        self,
        patch: TournamentPatch,
        pool: &Pool<Postgres>,
    ) -> Result<Tournament, OmniError> {
        let tournament = Tournament {
            id: self.id,
            full_name: patch.full_name.unwrap_or(self.full_name),
            shortened_name: patch.shortened_name.unwrap_or(self.shortened_name),
            speech_time: patch.speech_time,
            end_protected_time: patch.end_protected_time,
            start_protected_time: patch.start_protected_time,
            ad_vocem_time: patch.ad_vocem_time,
            debate_time_slot: patch.debate_time_slot,
            debate_preparation_time: patch.debate_preparation_time,
            beep_on_speech_end: patch.beep_on_speech_end,
            beep_on_protected_time: patch.beep_on_protected_time,
            visualize_protected_time: patch.visualize_protected_time,
        };
        match query!(
            r#"UPDATE tournaments SET
            full_name = $1,
            shortened_name = $2,
            speech_time = $3,
            end_protected_time = $4,
            start_protected_time = $5,
            ad_vocem_time = $6,
            debate_time_slot = $7,
            debate_preparation_time = $8,
            beep_on_speech_end = $9,
            beep_on_protected_time = $10,
            visualize_protected_time = $11
            WHERE id = $12"#,
            tournament.full_name,
            tournament.shortened_name,
            tournament.speech_time,
            tournament.end_protected_time,
            tournament.start_protected_time,
            tournament.ad_vocem_time,
            tournament.debate_time_slot,
            tournament.debate_preparation_time,
            tournament.beep_on_speech_end,
            tournament.beep_on_protected_time,
            tournament.visualize_protected_time,
            tournament.id,
        )
        .execute(pool)
        .await
        {
            Ok(_) => Ok(tournament),
            Err(e) => Err(e)?,
        }
    }

    pub async fn delete(self, pool: &Pool<Postgres>) -> Result<(), OmniError> {
        match query!("DELETE FROM tournaments WHERE id = $1", self.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_debates(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Debate>, OmniError> {
        match query_as!(
            Debate,
            "SELECT * FROM debates WHERE tournament_id = $1",
            &self.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(debates) => Ok(debates),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_plan(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<TournamentPlan>, OmniError> {
        match query_as!(
            TournamentPlan,
            "SELECT * FROM tournament_plans WHERE tournament_id = $1",
            &self.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(plan) => Ok(plan),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_teams(&self, pool: &Pool<Postgres>) -> Result<Vec<Team>, OmniError> {
        match query_as!(
            Team,
            "SELECT * FROM teams WHERE tournament_id = $1",
            &self.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(debates) => Ok(debates),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_locations(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Location>, OmniError> {
        match query_as!(
            Location,
            "SELECT * FROM locations WHERE tournament_id = $1",
            self.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(locations) => Ok(locations),
            Err(e) => Err(e)?,
        }
    }

    pub async fn get_rounds(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Round>, OmniError> {
        let mut retrieved_rounds: Vec<Round> = vec![];
        let phases = self.get_phases(pool).await?;
        for phase in phases {
            let rounds = phase.get_rounds(pool).await?;
            for round in rounds {
                retrieved_rounds.push(round);
            }
        }
        Ok(retrieved_rounds)
    }

    pub async fn get_phases(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<Phase>, OmniError> {
        let mut phases: Vec<Phase> = vec![];
        let records = query!("SELECT * FROM phases WHERE tournament_id = $1", self.id)
            .fetch_all(pool)
            .await?;
        for record in records {
            let phase = Phase {
                id: record.id,
                name: record.name,
                tournament_id: record.tournament_id,
                is_finals: record.is_finals,
                previous_phase_id: record.previous_phase_id,
                group_size: record.group_size,
                status: PhaseStatus::try_from(record.status)?,
            };
            phases.push(phase);
        }
        Ok(phases)
    }
}
