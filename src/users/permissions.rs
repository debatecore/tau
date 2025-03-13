use strum::{EnumIter, VariantArray};
use utoipa::ToSchema;

#[derive(Debug, VariantArray, EnumIter, Clone, PartialEq, ToSchema)]
/// To perform any operation, a user is required to have a corresponding
/// permission. Permissions are predefined for each role.
/// The infrastructure admin has all permissions and is allowed to perform
/// every operation.
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

    WriteRoles,
}
