-- Add migration script here
CREATE TABLE logging_channels (
    guild_id bigint PRIMARY KEY,
    channel_id bigint NOT NULL,
    bitwise bigint NOT NULL
);
