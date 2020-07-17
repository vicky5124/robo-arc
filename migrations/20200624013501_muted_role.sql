-- Add migration script here
CREATE TABLE muted_roles (
    guild_id bigint PRIMARY KEY,
    role_id bigint NOT NULL
);
