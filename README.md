# **Robo Arc**
Learning Rust project. Yes, im very original and full of ideas, so im remaking what i already did with python, but hopefully better this time.

![Bot profile picture](PFP.png "Bot's profile picture")

This bot is made using [serenity.rs](https://github.com/serenity-rs/serenity/) (currently using the development release [serenity.await](https://github.com/Lakelezz/serenity/blob/await/)), an async discord api wrapper for [Rust](https://www.rust-lang.org/)

## __Running the bot for yourself__

### __**Get the source and prepare it**__:

```bash
git clone git@gitlab.com:nitsuga5124/robo-arc.git # Over SSH
git clone https://gitlab.com/nitsuga5124/robo-arc.git # Over HTTPS

cd robo-arc
mv config.toml.example config.toml
```

### __**Discord token**__:

First, create an application [Here](https://discordapp.com/developers/applications/)
\
Go the thw newly created application and head over the `Bot` tab.
\
In here you create a bot and copy the token. This is what will be put on the `discord` variable inside `config.toml`.
\
> You can also create the invite link for the bot on the OAuth2 tab; Just select bot, the permissions you want the bot to have and copy the invite link.

### __**Other tokens**__:

- __osu!__
\
    Obtain the token [HERE](https://osu.ppy.sh/p/api/)
\
    NOTE: You will need to create an account if you do not already have one.
\
    NOTE: Do not create more than 1 account in the case of already having one, it's a banable offence.

- __Sankaku__
\
    Create an account [HERE](https://idol.sankakucomplex.com/user/signup)
\
    Obtain the "passhash" from the location [THIS](https://forum.sankakucomplex.com/t/channel-api-for-discord-integration/2204/7) image shows.
\
    NOTE: The image shows the Chrome Dev Tools (f12), On firefox the tab is called "Storage", same key.


### __**PostgreSQL Database**__:

You'll need to have a psql server running. If you don't know how, i recommend using docker. [Here's](https://www.youtube.com/watch?v=aHbE3pTyG-Q) a video that will help you with that.

    NOTE: if you are using windows, docker requires the hyper-V module to be enabled, which breaks other virualization software like VirtualBox or VMWare. If you use any of those software's, consider setting the database on the system natively.

With a created database and you connected with a user, youll need to create different tables, required by the bot.
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
```sql
CREATE TABLE public.annoyed_channels (
    channel_id bigint NOT NULL,
    CONSTRAINT annoyed_channels_pkey PRIMARY KEY (channel_id)
)
```
```sql
CREATE TABLE public.best_bg (
    user_id bigint NOT NULL,
    best_boy text COLLATE pg_catalog."default",
    best_girl text COLLATE pg_catalog."default",
    booru text COLLATE pg_catalog."default",
    CONSTRAINT best_bg_pkey PRIMARY KEY (user_id)
)
```
```sql
CREATE TABLE public.prefixes (
    guild_id bigint NOT NULL,
    prefix text COLLATE pg_catalog."default",
    CONSTRAINT prefixes_pkey PRIMARY KEY (guild_id)
)
```
```sql
CREATE TABLE public.new_posts (
    booru_url text COLLATE pg_catalog."default" NOT NULL,
    tags text COLLATE pg_catalog."default" NOT NULL,
    webhook text[] COLLATE pg_catalog."default",
    channel_id bigint[],
    sent_md5 text[] COLLATE pg_catalog."default"
)
```

### __**Eval command**__:
If you'd like to have an eval command, to evaluate python code within discord, you will need to use the `basic_python_bot_for_eval.py` file.

Dependencies:
```
# install python 3.6 or newer and python3-pip

python3 -m pip install pipenv -U --user
pipenv install toml git+https://github.com/Rapptz/discord.py.git#egg=feature-intents
```
Running the file:
```
pipenv run python3 basic_python_bot_for_eval.py
```

### __**Running the bot**__:

It's as simple as just running:

```bash
export DATABASE_URL="postgres://`postgres_username`:`postgres_user_password`@`postgres_ip`:`postgres_port`/`postgres_database`"
# example: postgres://postgres:123456@localhost:5432/arcbot
# this values are the same as the ones you put on config.toml
cargo run --release
```

## Notes

- To get the idol command working, you *may* need to run this command first:
> `curl -H "\x00\x00\x00\x08\x00\x00\x00\x01\x00\x00\x00\x14\x00U\x00s\x00e\x00r\x00-\x00A\x00g\x00e\x00n\x00t\x00\x00\x00\n\x00\x00\x00ä\x00M\x00o\x00z\x00i\x00l\x00l\x00a\x00/\x005\x00.\x000\x00 \x00(\x00W\x00i\x00n\x00d\x00o\x00w\x00s\x00 \x00N\x00T\x00 \x001\x000\x00.\x000\x00;\x00 \x00W\x00i\x00n\x006\x004\x00;\x00 \x00x\x006\x004\x00)\x00 \x00A\x00p\x00p\x00l\x00e\x00W\x00e\x00b\x00K\x00i\x00t\x00/\x005\x003\x007\x00.\x003\x006\x00 \x00(\x00K\x00H\x00T\x00M\x00L\x00,\x00 \x00l\x00i\x00k\x00e\x00 \x00G\x00e\x00c\x00k\x00o\x00)\x00 \x00C\x00h\x00r\x00o\x00m\x00e\x00/\x006\x007\x00.\x000\x00.\x003\x003\x009\x006\x00.\x008\x007\x00 \x00S\x00a\x00f\x00a\x00r\x00i\x00/\x005\x003\x007\x00.\x003\x006" https://iapi.sankakucomplex.com/post/index.json\?page=1\&limit=1\&tags=rating:safe`
