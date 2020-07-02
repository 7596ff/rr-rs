BEGIN;

CREATE TABLE images (
    guild_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    image BYTEA NOT NULL,
    filetype TEXT NOT NULL
);

COMMIT;
