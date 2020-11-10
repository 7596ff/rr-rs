BEGIN;

CREATE TABLE emojis (
    datetime   BIGINT      NOT NULL,
    guild_id   VARCHAR(24) NOT NULL,
    message_id VARCHAR(24) NOT NULL,
    member_id  VARCHAR(24) NOT NULL,
    emoji_id   VARCHAR(24) NOT NULL,
    reaction   BOOLEAN     NOT NULL DEFAULT false
);

COMMIT;
