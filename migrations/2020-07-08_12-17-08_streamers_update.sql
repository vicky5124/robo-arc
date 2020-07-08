-- Add migration script here
ALTER TABLE streamer_notification_channel DROP CONSTRAINT streamer_notification_channel_streamer_fkey, ADD CONSTRAINT streamer_notification_channel_streamer_fkey FOREIGN KEY (streamer) REFERENCES streamers (streamer) ON UPDATE CASCADE;
ALTER TABLE streamer_notification_webhook DROP CONSTRAINT streamer_notification_webhook_streamer_fkey, ADD CONSTRAINT streamer_notification_webhook_streamer_fkey FOREIGN KEY (streamer) REFERENCES streamers (streamer) ON UPDATE CASCADE;
