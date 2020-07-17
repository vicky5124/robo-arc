-- Add migration script here
CREATE TABLE reminders(
    id serial NOT NULL PRIMARY KEY,
    date timestamptz NOT NULL,
    message_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    guild_id bigint NOT NULL,
    user_id bigint NOT NULL,
    message text
);
