CREATE TABLE IF NOT EXISTS users (
    id               UUID NOT NULL UNIQUE PRIMARY KEY,
    handle           TEXT NOT NULL UNIQUE,
    picture_link     TEXT DEFAULT NULL,

    password_hash    TEXT NOT NULL
    -- attempts         INTEGER NOT NULL DEFAULT 0,
    -- locked           BOOLEAN NOT NULL DEFAULT FALSE,
);

CREATE TABLE IF NOT EXISTS sessions (
    id               UUID NOT NULL UNIQUE PRIMARY KEY,
    token            TEXT NOT NULL UNIQUE,
    user_id          UUID NOT NULL REFERENCES users(id),
    issued           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expiry           TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '1 week',
    last_access      TIMESTAMPTZ DEFAULT NULL
    -- revoked          BOOLEAN NOT NULL DEFAULT FALSE,
    -- revoked_at       TIMESTAMPZ DEFAULT NULL,

    -- ip_address       INET NOT NULL,
    -- user_agent       TEXT NOT NULL,
    -- devices          TEXT[] NOT NULL,
    -- geolocation      TEXT NOT NULL,
    -- countries        TEXT[] NOT NULL,
    -- login_method     TEXT NOT NULL
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
    team_id            UUID NOT NULL REFERENCES teams(id)
);

CREATE TABLE IF NOT EXISTS debates (
    id                UUID NOT NULL UNIQUE PRIMARY KEY,
    motion_id         UUID REFERENCES motions(id),
    marshall_user_id  UUID NOT NULL REFERENCES users(id),
    tournament_id     UUID NOT NULL REFERENCES tournaments(id)
);

CREATE TABLE IF NOT EXISTS debate_teams_assignments (
    id                UUID NOT NULL UNIQUE PRIMARY KEY,
    team_id           UUID NOT NULL REFERENCES teams(id),
    debate_id         UUID NOT NULL REFERENCES debates(id),
    is_proposition    BOOLEAN DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS debate_judge_assignments (
    id                UUID NOT NULL UNIQUE PRIMARY KEY,
    judge_user_id     UUID NOT NULL REFERENCES users(id),
    debate_id         UUID NOT NULL REFERENCES debates(id)
);

CREATE TABLE IF NOT EXISTS locations (
    id                UUID NOT NULL UNIQUE PRIMARY KEY,
    name              TEXT NOT NULL,
    tournament_id     UUID NOT NULL REFERENCES tournaments(id),
    address           TEXT,
    remarks           TEXT
);

CREATE TABLE IF NOT EXISTS rooms (
    id                UUID NOT NULL UNIQUE PRIMARY KEY,
    name              TEXT NOT NULL,
    location_id       UUID NOT NULL REFERENCES locations(id),
    remarks           TEXT,
    is_occupied       BOOLEAN NOT NULL DEFAULT FALSE
)
