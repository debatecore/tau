use strum::VariantArray;

use super::permissions::Permission;

#[derive(Debug, PartialEq)]
pub enum Role {
    Admin,
    Organizer,
    Judge,
    Marshall,
}

impl Role {
    pub fn get_role_permissions(&self) -> Vec<Permission> {
        use Permission::*;
        use Role::*;
        match self {
            Admin => Permission::VARIANTS.to_vec(), // all permissions
            Organizer => vec![
                CreateUsersManually,
                CreateUsersWithLink,
                DeleteUsers,
                ModifyUserRoles,
            ],
            Judge => vec![SubmitOwnVerdictVote],
            Marshall => vec![SubmitVerdict],
        }
    }
}
