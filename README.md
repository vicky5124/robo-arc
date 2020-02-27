# Robo Arc
Learning Rust project. Yes, im very original and full of ideas, so im remaking what i already did with python, but hopefully better this time.

This bot is made using [serenity.rs](https://github.com/serenity-rs/serenity/), a sync discord api wrapper for [Rust](https://www.rust-lang.org/)

### Running the bot for yourself

+ __**Get the source and prepare it**__

```bash
git clone git@gitlab.com:nitsuga5124/robo-arc.git # Over SSH
git clone https://gitlab.com/nitsuga5124/robo-arc.git # Over HTTPS

cd robo-arc
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

+ __**Other tokens**__:

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

## Notes

- To get the idol command working, you may need to run this command first:
> `curl -H "\x00\x00\x00\x08\x00\x00\x00\x01\x00\x00\x00\x14\x00U\x00s\x00e\x00r\x00-\x00A\x00g\x00e\x00n\x00t\x00\x00\x00\n\x00\x00\x00ä\x00M\x00o\x00z\x00i\x00l\x00l\x00a\x00/\x005\x00.\x000\x00 \x00(\x00W\x00i\x00n\x00d\x00o\x00w\x00s\x00 \x00N\x00T\x00 \x001\x000\x00.\x000\x00;\x00 \x00W\x00i\x00n\x006\x004\x00;\x00 \x00x\x006\x004\x00)\x00 \x00A\x00p\x00p\x00l\x00e\x00W\x00e\x00b\x00K\x00i\x00t\x00/\x005\x003\x007\x00.\x003\x006\x00 \x00(\x00K\x00H\x00T\x00M\x00L\x00,\x00 \x00l\x00i\x00k\x00e\x00 \x00G\x00e\x00c\x00k\x00o\x00)\x00 \x00C\x00h\x00r\x00o\x00m\x00e\x00/\x006\x007\x00.\x000\x00.\x003\x003\x009\x006\x00.\x008\x007\x00 \x00S\x00a\x00f\x00a\x00r\x00i\x00/\x005\x003\x007\x00.\x003\x006" https://iapi.sankakucomplex.com/post/index.json\?page=1\&limit=1\&tags=rating:safe`
