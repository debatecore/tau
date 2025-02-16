use serde::Deserialize;
use strum::VariantArray;

use super::permissions::Permission;

#[derive(Debug, PartialEq, Deserialize)]
pub enum Role {
    Admin,
    Organizer,
    Judge,
    Marshall,
}

impl Role {
    pub fn get_role_permissions(&self) -> Vec<Permission> {
        use Permission as P;
        match self {
            Role::Admin => Permission::VARIANTS.to_vec(), // all permissions
            Role::Organizer => vec![
                P::CreateUsersManually,
                P::CreateUsersWithLink,
                P::DeleteUsers,
                P::ModifyUserRoles,
                P::ReadAttendees,
                P::WriteAttendees,
                P::ReadDebates,
                P::WriteDebates,
                P::ReadTeams,
                P::WriteTeams,
                P::ReadTournament,
                P::WriteTournament,
                P::SubmitVerdict,
            ],
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
