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

Any OS:
```bash
# Rename tokens.toml.example to tokens.toml
# Modify the file with the required data
cargo run
```
