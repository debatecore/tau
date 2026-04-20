use serde::Deserialize;
use strum::{EnumIter, EnumString, VariantArray}; // Added EnumString
use utoipa::ToSchema;

#[derive(Debug, VariantArray, EnumIter, EnumString, Clone, PartialEq, ToSchema)]

/// Permissions gate every operation in the system.
///
/// Each [`Role`] carries a fixed set of permissions. To perform any operation,
/// the acting user must hold at least one role whose permission set includes the
/// required permission.
///
/// The **infrastructure admin** is a special super-user that bypasses all
/// permission checks and is implicitly granted every permission listed below.
///
/// ## Available permissions
///
/// | Permission | Description |
/// |---|---|
/// | `ReadAttendees` | View the list of attendees registered to a tournament. |
/// | `WriteAttendees` | Add, edit, or remove attendees. |
/// | `ReadDebates` | View debate pairings and draw details. |
/// | `WriteDebates` | Create or modify debate pairings. |
/// | `ReadTeams` | View team registrations. |
/// | `WriteTeams` | Create, edit, or disband teams. |
/// | `ReadMotions` | View motions assigned to debates. |
/// | `WriteMotions` | Set or update debate motions. |
/// | `ReadTournament` | View general tournament information. |
/// | `WriteTournament` | Edit tournament settings and metadata. |
/// | `CreateUsersManually` | Manually register new user accounts. |
/// | `CreateUsersWithLink` | Generate invite links that create accounts on use. |
/// | `DeleteUsers` | Remove user accounts from the tournament. |
/// | `ModifyUserRoles` | Assign or revoke roles for users in a tournament. |
/// | `SubmitOwnVerdictVote` | Submit a ballot as a judge for a debate you are assigned to. |
/// | `SubmitVerdict` | Finalise and publish the official verdict of a debate. |
/// | `ReadLocations` | View venue/location data. |
/// | `WriteLocations` | Create or modify venues/locations. |
/// | `ReadRooms` | View room assignments. |
/// | `WriteRooms` | Create or modify room assignments. |
#[derive(Deserialize, Copy)]
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

    ReadPlan,
    WritePlan,
}
