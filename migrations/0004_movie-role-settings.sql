BEGIN;

ALTER TABLE settings
    ADD COLUMN movies_role TEXT,
    DROP COLUMN movies_repeat_every;

COMMIT;
