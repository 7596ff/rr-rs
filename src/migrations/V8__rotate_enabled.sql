BEGIN;

ALTER TABLE settings
    ADD COLUMN rotate_enabled BOOL NOT NULL DEFAULT TRUE;

COMMIT;
