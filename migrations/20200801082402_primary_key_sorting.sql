-- Add migration script here
ALTER TABLE streamer_notification_channel DROP CONSTRAINT streamer_notification_channel_streamer_fkey;
ALTER TABLE streamer_notification_webhook DROP CONSTRAINT streamer_notification_webhook_streamer_fkey;

ALTER TABLE streamers DROP CONSTRAINT streamers_pkey;

ALTER TABLE streamers ADD UNIQUE(streamer);

ALTER TABLE streamers ADD id SERIAL PRIMARY KEY;
ALTER TABLE new_posts ADD id SERIAL PRIMARY KEY;

ALTER TABLE streamer_notification_channel ADD CONSTRAINT streamer_notification_channel_streamer_fkey FOREIGN KEY (streamer) REFERENCES streamers (streamer) ON UPDATE CASCADE;
ALTER TABLE streamer_notification_webhook ADD CONSTRAINT streamer_notification_webhook_streamer_fkey FOREIGN KEY (streamer) REFERENCES streamers (streamer) ON UPDATE CASCADE;
