-- Add migration script here
CREATE TABLE public.annoyed_channels
(
    channel_id bigint NOT NULL,
    CONSTRAINT annoyed_channels_pkey PRIMARY KEY (channel_id)
)

TABLESPACE pg_default;

ALTER TABLE public.annoyed_channels
    OWNER to postgres;

CREATE TABLE public.best_bg
(
    user_id bigint NOT NULL,
    best_boy text COLLATE pg_catalog."default",
    best_girl text COLLATE pg_catalog."default",
    booru text COLLATE pg_catalog."default",
    CONSTRAINT best_bg_pkey PRIMARY KEY (user_id)
)

TABLESPACE pg_default;

ALTER TABLE public.best_bg
    OWNER to postgres;

CREATE TABLE public.new_posts
(
    booru_url text COLLATE pg_catalog."default" NOT NULL,
    tags text COLLATE pg_catalog."default" NOT NULL,
    webhook text[] COLLATE pg_catalog."default",
    channel_id bigint[],
    sent_md5 text[] COLLATE pg_catalog."default"
)

TABLESPACE pg_default;

ALTER TABLE public.new_posts
    OWNER to postgres;

CREATE TABLE public.osu_user
(
    discord_id bigint NOT NULL,
    osu_id integer NOT NULL,
    osu_username character varying(50) COLLATE pg_catalog."default" NOT NULL,
    pp boolean,
    mode integer,
    short_recent boolean,
    CONSTRAINT osu_user_pkey PRIMARY KEY (discord_id)
)

TABLESPACE pg_default;

ALTER TABLE public.osu_user
    OWNER to postgres;

CREATE TABLE public.prefixes
(
    guild_id bigint NOT NULL,
    prefix text COLLATE pg_catalog."default",
    disallowed_commands text[] COLLATE pg_catalog."default",
    CONSTRAINT prefixes_pkey PRIMARY KEY (guild_id)
)

TABLESPACE pg_default;

ALTER TABLE public.prefixes
    OWNER to postgres;

CREATE TABLE public.streamers
(
    streamer text COLLATE pg_catalog."default" NOT NULL,
    is_live boolean NOT NULL DEFAULT false,
    use_default boolean NOT NULL DEFAULT false,
    live_message text COLLATE pg_catalog."default" DEFAULT 'I''m live!'::text,
    not_live_message text COLLATE pg_catalog."default" DEFAULT 'I''m no longer live.'::text,
    CONSTRAINT streamers_pkey PRIMARY KEY (streamer)
)

TABLESPACE pg_default;

ALTER TABLE public.streamers
    OWNER to postgres;


CREATE TABLE public.streamer_notification_channel
(
    streamer text COLLATE pg_catalog."default" NOT NULL,
    role_id bigint,
    use_default boolean NOT NULL DEFAULT false,
    live_message text COLLATE pg_catalog."default" DEFAULT 'I''m live!'::text,
    not_live_message text COLLATE pg_catalog."default" DEFAULT 'I''m no longer live.'::text,
    channel_id bigint,
    message_id bigint,
    CONSTRAINT streamer_notification_channel_streamer_fkey FOREIGN KEY (streamer)
        REFERENCES public.streamers (streamer) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
)

TABLESPACE pg_default;

ALTER TABLE public.streamer_notification_channel
    OWNER to postgres;

CREATE TABLE public.streamer_notification_webhook
(
    streamer text COLLATE pg_catalog."default" NOT NULL,
    webhook text COLLATE pg_catalog."default" NOT NULL,
    role_id bigint,
    live_message text COLLATE pg_catalog."default" DEFAULT 'I''m live!'::text,
    not_live_message text COLLATE pg_catalog."default" DEFAULT 'I''m no longer live.'::text,
    message_id bigint,
    CONSTRAINT streamer_notification_webhook_streamer_fkey FOREIGN KEY (streamer)
        REFERENCES public.streamers (streamer) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
)

TABLESPACE pg_default;

ALTER TABLE public.streamer_notification_webhook
    OWNER to postgres;

