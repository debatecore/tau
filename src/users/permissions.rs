use strum::{EnumIter, VariantArray};

#[derive(Debug, VariantArray, EnumIter, Clone, PartialEq)]
pub enum Permission {
    ReadAttendees,
    WriteAttendees,

    ReadDebates,
    WriteDebates,

    ReadTeams,
    WriteTeams,

    ReadMotions,
    WriteMotions,

    ReadTournaments,
    WriteTournaments,

    CreateUsersManually,
    CreateUsersWithLink,
    DeleteUsers,
    ModifyUserRoles,

    SubmitOwnVerdictVote,
    SubmitVerdict,
}
