-- Add migration script here
CREATE TABLE anti_spam (
    guild_id bigint PRIMARY KEY NOT NULL,
    enabled bool NOT NULL
);
