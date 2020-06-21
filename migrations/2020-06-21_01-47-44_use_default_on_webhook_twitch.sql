-- Add migration script here
ALTER TABLE streamer_notification_webhook
ADD use_default bool;
