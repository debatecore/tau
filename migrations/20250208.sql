ALTER TABLE debates ADD COLUMN tournament_id UUID NOT NULL REFERENCES tournaments(id);
ALTER TABLE debates ALTER COLUMN motion_id DROP NOT NULL;
