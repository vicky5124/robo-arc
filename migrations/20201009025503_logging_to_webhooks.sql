-- Add migration script here
DELETE FROM logging_channels;
ALTER TABLE logging_channels DROP COLUMN channel_id;
ALTER TABLE logging_channels ADD COLUMN webhook_url text NOT NULL;
