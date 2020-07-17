-- Add migration script here
CREATE TYPE message_type AS ENUM (
    'Regular',
    'GroupRecipientAddition',
    'GroupRecipientRemoval',
    'GroupCallCreation',
    'GroupNameUpdate',
    'GroupIconUpdate',
    'PinsAdd',
    'MemberJoin',
    'NitroBoost',
    'NitroTier1',
    'NitroTier2',
    'NitroTier3'
);
CREATE TABLE log_messages(
    id bigint PRIMARY KEY,
    channel_id bigint NOT NULL,
    guild_id bigint NOT NULL,
    author_id bigint NOT NULL,
    
    content text,
    content_history text[],
    attachments text[],
    attachments_history text[][],

    embeds text[],
    embeds_history text[][],
    
    pinned bool NOT NULL,
    was_pinned bool DEFAULT false,
    kind message_type,
    
    creation_timestamp timestamptz NOT NULL,
    edited_timestamp timestamptz,
    
    tts bool,
    webhook_id bigint
);
