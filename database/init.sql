CREATE TABLE IF NOT EXISTS debates (
    id               TEXT NOT NULL UNIQUE FOREIGN KEY,
    team1Id          TEXT NOT NULL,
    team2Id          TEXT NOT NULL,
    motionId         TEXT NOT NULL,
    marshallUserId   TEXT NOT NULL,
    propositionTeamAssignment    INTEGER NOT NULL,
    -- 0 ^^ if undecided, otherwise 1 or 2
)

CREATE TABLE IF NOT EXISTS debate_judge (
    id              TEXT NOT NULL UNIQUE PRIMARY KEY,
    judgeUserId     TEXT NOT NULL,
    debateId        TEXT NOT NULL,
)

CREATE TABLE IF NOT EXISTS motions (
    id           TEXT NOT NULL UNIQUE PRIMARY KEY,
    motion       TEXT NOT NULL,
    adinfo       TEXT,  
)

CREATE TABLE IF NOT EXISTS roles (
    id              TEXT NOT NULL UNIQUE PRIMARY KEY,
    tournamentId    TEXT NOT NULL,
    roles           TEXT[] NOT NULL,
)

CREATE TABLE IF NOT EXISTS speakers (
    id                 TEXT NOT NULL UNIQUE PRIMARY KEY,
    name               TEXT NOT NULL,
    position           INTEGER NOT NULL,
    teamId             TEXT NOT NULL,
    individualPoints   INTEGER NOT NULL,
    penaltyPoints      INTEGER NOT NULL,
)

CREATE TABLE IF NOT EXISTS teams (
    id              TEXT NOT NULL UNIQUE PRIMARY KEY,
    fullName        TEXT NOT NULL,
    shortenedName   TEXT NOT NULL,
    tournamentId    TEXT NOT NULL,
)

CREATE TABLE IF NOT EXISTS tournaments (
    id              TEXT NOT NULL UNIQUE PRIMARY KEY,
    fullName        TEXT NOT NULL,
    shortenedName   TEXT NOT NULL,
)

CREATE TABLE IF NOT EXISTS tournament_user (
    id              TEXT NOT NULL UNIQUE PRIMARY KEY,
    userId          TEXT NOT NULL,
    tournamentId    TEXT NOT NULL,
)

CREATE TABLE IF NOT EXISTS users (
    id              TEXT NOT NULL UNIQUE PRIMARY KEY,
    handle          TEXT NOT NULL,
    passwordHash   TEXT NOT NULL,
    pictureLink    TEXT,
)
CREATE TABLE IF NOT EXISTS roles (
    id              TEXT NOT NULL UNIQUE PRIMARY KEY,
    userId          TEXT NOT NULL,
    rolesId         TEXT NOT NULL,
)
