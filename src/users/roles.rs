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
            ],
            Role::Judge => vec![P::SubmitOwnVerdictVote],
            Role::Marshall => vec![P::SubmitVerdict],
        }
    }
}
