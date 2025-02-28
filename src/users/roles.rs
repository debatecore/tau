use serde::{Deserialize, Serialize};
use strum::VariantArray;
use utoipa::ToSchema;

use super::permissions::Permission;

#[derive(Debug, PartialEq, Deserialize, ToSchema, VariantArray, Clone, Serialize)]
/// Within a tournament, users must be granted roles for their
/// permissions to be defined. Each role comes with a predefined
/// set of permissions to perform certain operations.
/// By default, a newly created user has no roles.
/// Multiple users can have the same role.
pub enum Role {
    /// This role grants all possible permissions within a tournament.
    Organizer,
    /// Judges can submit their verdicts regarding debates they were assigned to.
    Judge,
    /// Marshalls are responsible for conducting debates.
    /// For pragmatic reasons, they can submit verdicts on Judges' behalf.
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
            ],
            Role::Marshall => vec![
                P::ReadDebates,
                P::ReadAttendees,
                P::ReadTeams,
                P::ReadTournament,
                P::SubmitVerdict,
            ],
        }
    }
}
