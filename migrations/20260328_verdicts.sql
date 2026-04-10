CREATE TABLE IF NOT EXISTS verdicts (
    id                   UUID NOT NULL UNIQUE PRIMARY KEY,
    debate_id            UUID NOT NULL REFERENCES debates(id),
    judge_user_id        UUID NOT NULL REFERENCES users(id),
    proposition_won      BOOLEAN NOT NULL
);