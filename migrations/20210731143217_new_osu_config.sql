-- Add migration script here
CREATE TABLE osu(
    discord_id BIGINT NOT NULL PRIMARY KEY,
    osu_id INTEGER NOT NULL,
    instant_recent BOOLEAN NOT NULL DEFAULT false
);
