CREATE TABLE IF NOT EXISTS users (
    id               UUID NOT NULL UNIQUE PRIMARY KEY,
    handle           TEXT NOT NULL UNIQUE,
    password_hash    TEXT NOT NULL,
    picture_link     TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS tournaments (
    id               UUID NOT NULL UNIQUE PRIMARY KEY,
    full_name        TEXT NOT NULL UNIQUE,
    shortened_name   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS roles (
    id              UUID NOT NULL UNIQUE PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id),
    tournament_id   UUID NOT NULL REFERENCES tournaments(id),
    roles           TEXT[] DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS motions (
    id           UUID NOT NULL UNIQUE PRIMARY KEY,
    motion       TEXT NOT NULL UNIQUE,
    adinfo       TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS teams (
    id               UUID NOT NULL UNIQUE PRIMARY KEY,
    full_name        TEXT NOT NULL,
    shortened_name   TEXT NOT NULL,
    tournament_id    UUID NOT NULL REFERENCES tournaments(id)
);

CREATE TABLE IF NOT EXISTS attendees (
    id                 UUID NOT NULL UNIQUE PRIMARY KEY,
    name               TEXT NOT NULL,
    position           INTEGER DEFAULT NULL,
    team_id            UUID DEFAULT NULL REFERENCES teams(id),
    individual_points  INTEGER NOT NULL DEFAULT 0,
    penalty_points     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS debates (
    id                UUID NOT NULL UNIQUE,
    team1_id          UUID NOT NULL REFERENCES teams(id),
    team2_id          UUID NOT NULL REFERENCES teams(id),
    motion_id         UUID NOT NULL REFERENCES motions(id),
    marshall_user_id  UUID NOT NULL REFERENCES users(id),
    proposition_team_assignment INTEGER NOT NULL DEFAULT 0
    -- 0 ^^ if undecided, otherwise 1 or 2
);

CREATE TABLE IF NOT EXISTS debate_judge_assignment (
    id                UUID NOT NULL UNIQUE PRIMARY KEY,
    judge_user_id     UUID NOT NULL REFERENCES users(id),
    debate_id         UUID NOT NULL REFERENCES debates(id)
);
