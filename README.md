# Robo Arc
Learning Rust project. Yes, im very original and full of ideas, so im remaking what i already did with python, but hopefully better this time.

This bot is made using [serenity.rs](https://github.com/serenity-rs/serenity/), a sync discord api wrapper for [Rust](https://www.rust-lang.org/)

### Running the bot for yourself

+ __**Get the source and prepare it**__

```bash
git clone git@gitlab.com:nitsuga5124/robo-arc.git # Over SSH
git clone https://gitlab.com/nitsuga5124/robo-arc.git # Over HTTPS

mv config.toml.example config.toml
```

+ __**Discord token**__:

First, create an application [Here](https://discordapp.com/developers/applications/)
\
Go the thw newly created application and head over the `Bot` tab.
\
In here you create a bot and copy the token. This is what will be put on the `discord` variable inside `config.toml`.
\
> You can also create the invite link for the bot on the OAuth2 tab; Just select bot, the permissions you want the bot to have and copy the invite link.


+ __**PostgreSQL Database**__:

You'll need to have a psql server running. If you don't know how, i recommend using docker. [Here's](https://www.youtube.com/watch?v=aHbE3pTyG-Q) a video that will help you with that.

With a created database and you connected with a user, youll need to create the osu_user table.
```sql
CREATE TABLE public.osu_user (
    discord_id bigint NOT NULL,
    osu_id integer NOT NULL,
    osu_username character varying(50) COLLATE pg_catalog."default" NOT NULL,
    pp boolean,
    mode integer,
    short_recent boolean,
    CONSTRAINT osu_user_pkey PRIMARY KEY (discord_id)
)
```

+ __**Running the bot**__

It's as simple as just running:

```bash
cargo run
```
