use strum::{EnumIter, VariantArray, EnumString}; // Added EnumString
use utoipa::ToSchema;
use serde::{Deserialize, Serialize} // use serde :: {Deserialize, Serialize} added.

#[derive(Debug, VariantArray, EnumIter, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
#[strum(serialize_all = "PascalCase")] // Ensures parsing matches the variant names
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

    ReadLocations,
    WriteLocations,

    ReadRooms,
    ModifyAllRoomDetails,
    ChangeRoomOccupationStatus,

    ReadAffiliations,
    WriteAffiliations,
    WriteRooms,

    ReadPhases,
    WritePhases,

    ReadRounds,
    WriteRounds,
}
