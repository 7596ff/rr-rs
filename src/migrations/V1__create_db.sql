BEGIN;

CREATE TABLE IF NOT EXISTS guilds (
    id   TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    guild_id             TEXT    UNIQUE NOT NULL,
    starboard_channel_id TEXT,
    starboard_emoji      TEXT           NOT NULL DEFAULT '⭐',
    starboard_min_stars  INTEGER        NOT NULL DEFAULT 1,
    movies_repeat_every  INTEGER        NOT NULL DEFAULT 7
);

CREATE TABLE IF NOT EXISTS invite_roles (
    guild_id    TEXT NOT NULL,
    id          TEXT NOT NULL,
    invite_code TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS roleme_roles (
    guild_id    TEXT NOT NULL,
    id          TEXT NOT NULL,
    color       TEXT
);

CREATE TABLE IF NOT EXISTS starboard (
    guild_id   TEXT      NOT NULL,
    member_id  TEXT      NOT NULL,
    channel_id TEXT      NOT NULL,
    message_id TEXT      NOT NULL,
    post_id    TEXT      NOT NULL,
    star_count INTEGER   NOT NULL,
    date       TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS movies (
    guild_id   TEXT NOT NULL,
    member_id  TEXT NOT NULL,
    id         SERIAL,
    title      TEXT NOT NULL,
    url        TEXT,
    watch_date TIMESTAMP,
    nominated  BOOLEAN DEFAULT FALSE,
    UNIQUE (guild_id, member_id, title)
);

CREATE TABLE IF NOT EXISTS movie_votes (
    guild_id  TEXT    NOT NULL,
    member_id TEXT    NOT NULL,
    id        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS movie_dates (
    guild_id   TEXT NOT NULL,
    watch_date TIMESTAMP,
    id         INTEGER
);

COMMIT;

