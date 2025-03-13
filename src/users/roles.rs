use serde::Deserialize;
use strum::VariantArray;
use utoipa::ToSchema;

use super::permissions::Permission;

#[derive(Debug, PartialEq, Deserialize, ToSchema)]
/// Within a tournament, users must be granted roles for their
/// permissions to be defined. Each role comes with a predefined
/// set of permissions to perform certain operations.
/// By default, a newly created user has no roles.
pub enum Role {
    Organizer,
    Judge,
    Marshall,
}

impl Role {
    pub fn get_role_permissions(&self) -> Vec<Permission> {
        use Permission as P;
        match self {
            Role::Organizer => P::VARIANTS.to_vec(),
            Role::Judge => vec![
                P::ReadAttendees,
                P::ReadDebates,
                P::ReadTeams,
                P::ReadTournament,
                P::SubmitOwnVerdictVote,
                P::ReadLocations,
                P::ReadRooms,
            ],
            Role::Marshall => vec![
                P::ReadDebates,
                P::ReadAttendees,
                P::ReadTeams,
                P::ReadTournament,
                P::ReadLocations,
                P::ReadRooms,
                P::SubmitVerdict,
                P::ChangeRoomOccupationStatus,
            ],
        }
    }
}
