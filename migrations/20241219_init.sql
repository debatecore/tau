CREATE TABLE IF NOT EXISTS users (
    id              UUID NOT NULL UNIQUE PRIMARY KEY,
    handle          TEXT NOT NULL UNIQUE,
    passwordHash    TEXT NOT NULL,
    pictureLink     TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS tournaments (
    id              UUID NOT NULL UNIQUE PRIMARY KEY,
    fullName        TEXT NOT NULL UNIQUE,
    shortenedName   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS roles (
    id              UUID NOT NULL UNIQUE PRIMARY KEY,
    userId          UUID NOT NULL REFERENCES users(id),
    tournamentId    UUID NOT NULL REFERENCES tournaments(id),
    roles           TEXT[] DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS motions (
    id           UUID NOT NULL UNIQUE PRIMARY KEY,
    motion       TEXT NOT NULL UNIQUE,
    adinfo       TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS teams (
    id              UUID NOT NULL UNIQUE PRIMARY KEY,
    fullName        TEXT NOT NULL,
    shortenedName   TEXT NOT NULL,
    tournamentId    UUID NOT NULL REFERENCES tournaments(id)
);

CREATE TABLE IF NOT EXISTS attendees (
    id                 UUID NOT NULL UNIQUE PRIMARY KEY,
    name               TEXT NOT NULL,
    position           INTEGER DEFAULT NULL,
    teamId             UUID DEFAULT NULL REFERENCES teams(id),
    individualPoints   INTEGER NOT NULL DEFAULT 0,
    penaltyPoints      INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS debates (
    id               UUID NOT NULL UNIQUE,
    team1Id          UUID NOT NULL REFERENCES teams(id),
    team2Id          UUID NOT NULL REFERENCES teams(id),
    motionId         UUID NOT NULL REFERENCES motions(id),
    marshallUserId   UUID NOT NULL REFERENCES users(id),
    propositionTeamAssignment INTEGER NOT NULL DEFAULT 0
    -- 0 ^^ if undecided, otherwise 1 or 2
);

CREATE TABLE IF NOT EXISTS debate_judge_assignment (
    id              UUID NOT NULL UNIQUE PRIMARY KEY,
    judgeUserId     UUID NOT NULL REFERENCES users(id),
    debateId        UUID NOT NULL REFERENCES debates(id)
);
