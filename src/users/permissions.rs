use strum::{EnumIter, VariantArray};
use utoipa::ToSchema;

#[derive(Debug, VariantArray, EnumIter, Clone, PartialEq, ToSchema)]
pub enum Permission {
    ReadAttendees,
    WriteAttendees,

    ReadDebates,
    WriteDebates,

    ReadTeams,
    WriteTeams,

    ReadMotions,
    WriteMotions,

    ReadTournament,
    WriteTournament,

    CreateUsersManually,
    CreateUsersWithLink,
    DeleteUsers,
    ModifyUserRoles,

    SubmitOwnVerdictVote,
    SubmitVerdict,
}
