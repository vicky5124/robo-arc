# Robo Arc
Learning Rust project. Yes, im very original and full of ideas, so im remaking what i already did with python, but hopefully better this time.

This bot is made using `serenity.rs`, a sync discord api wrapper for [Rust](https://www.rust-lang.org/)

### Running the bot for yourself
First, create an application [Here](https://discordapp.com/developers/applications/)
\
Go the thw newly created application and head over the `Bot` tab.
\
In here you create a bot and copy the token. This is what will be put on the Token Env var.
\
- You can also create the invite on the OAuth2 tab, Just select bot, the permissions you want the bot to have and copy the invite link.

*nix:
```bash
export DEV_DISCORD_TOKEN="the bot token here" # it is recommended to put this inside ~/.profile so you don't have to run it every time you want to run the bot.
cargo run
```

Windows
```bash
# Setup the Env Var, google this as idk how.
cargo run
```
