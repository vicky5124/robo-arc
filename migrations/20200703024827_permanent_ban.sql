-- Add migration script here
CREATE TABLE permanent_bans (
    id serial NOT NULL PRIMARY KEY,
    guild_id bigint NOT NULL,
    banner_user_id bigint NOT NULL,
    user_id bigint NOT NULL
);
