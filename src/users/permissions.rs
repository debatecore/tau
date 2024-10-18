use strum::{EnumIter, VariantArray};

#[derive(Debug, VariantArray, EnumIter, Clone, PartialEq)]
pub enum Permission {
    CreateUsersManually,
    CreateUsersWithLink,
    DeleteUsers,
    ModifyUserRoles,

    SubmitOwnVerdictVote,
    SubmitAllVerdictVotes,
}
