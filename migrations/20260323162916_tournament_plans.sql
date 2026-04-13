-- Add migration script here
CREATE TABLE IF NOT EXISTS tournament_plans (
    id                 UUID NOT NULL UNIQUE PRIMARY KEY,
    tournament_id      UUID NOT NULL REFERENCES tournaments(id),
    group_phase_rounds INTEGER,
    groups_count       INTEGER,
    advancing_teams    INTEGER,
    total_teams        INTEGER
)