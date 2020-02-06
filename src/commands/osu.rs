use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};
use postgres::{Client, NoTls};
use regex::Regex;
use toml::Value;
use std::{
    fs::File,
    io::prelude::*,
};

use reqwest;
use serde::Deserialize;


#[derive(Deserialize, PartialEq)]
struct OsuUser {
    user_id: String,
}

#[derive(Default)]
struct OsuUserData {
    osu_id: i32,
    name: String,
    old_name: String,
    mode: Option<i32>,
    pp: Option<bool>,
    short_recent: Option<bool>,
}

pub fn get_database() -> Result<Client, Box<dyn std::error::Error>> {
    let mut file = File::open("tokens.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let tokens = contents.parse::<Value>().unwrap();
    let psql_password = tokens["psql_password"].as_str().unwrap();

    let client = Client::connect(
        &format!("host=localhost user=postgres password={} dbname=arcbot", psql_password)
        .to_owned()[..],
        NoTls
    )?;
    Ok(client)
}

fn get_osu_id(name: &String) -> Result<i32, Box<dyn std::error::Error>> {
    let mut file = File::open("tokens.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let tokens = contents.parse::<Value>().unwrap();
    let osu_key = tokens["osu"].as_str().unwrap();

    let re = Regex::new("[^0-9A-Za-z///' ]").unwrap();
    let mut sanitized_name = re.replace(name, "").into_owned();
    sanitized_name = sanitized_name.replace(" ", "%20");

    let url = format!("https://osu.ppy.sh/api/get_user?k={}&u={}&type=string", osu_key, sanitized_name);
    let resp = reqwest::blocking::get(&url)?
        .json::<Vec<OsuUser>>()?;

    if resp.len() != 0 {
        let id: i32 = resp[0].user_id.trim().parse()?;
        return Ok(id);
    } else {
        return Ok(0);
    }
}

#[command]
fn configure_osu(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {
    let mut client = get_database()?;
    let author_id = msg.author.id.as_u64().clone() as i64;
    let data = client.query("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1",
                            &[&author_id])?;
    let empty_data: bool;

    let mut user_data = OsuUserData::default();

    if !data.is_empty() {
        empty_data = false;
        for row in data {
            user_data.osu_id = row.get(0);
            user_data.name = row.get(1);
            user_data.old_name = user_data.name.clone();
            user_data.mode = row.get(3);
            user_data.pp = row.get(2);
            user_data.short_recent = row.get(4);
        }
    } else {
        empty_data = true;
    }

    if arguments.len() > 0 {
        let args = arguments.raw_quoted().collect::<Vec<&str>>();

        for arg in args {
            if arg.starts_with("pp=") {
                let x: &str = arg.split("=").nth(1).unwrap();
                user_data.pp = match x {
                    "y" | "yes" | "true" | "1" => Some(true),
                    _ => Some(false)
                }

            } else if arg.starts_with("short_recent=") || arg.starts_with("recent=") { 
                let x: &str = arg.split("=").nth(1).unwrap();
                user_data.short_recent = match x {
                    "y" | "yes" | "true" | "1" => Some(true),
                    _ => Some(false)
                }

            } else if arg.starts_with("mode=") { 
                let x: &str = arg.split("=").nth(1).unwrap();
                user_data.mode = match x {
                    "0" | "std" | "standard" => Some(0),
                    "1" | "taiko" => Some(1),
                    "2" | "ctb" | "catch" => Some(2),
                    "3" | "mania" => Some(3),
                    _ => None
                }

            } else {
                if empty_data {
                    user_data.name += arg;
                } else {
                    user_data.name = if user_data.name == user_data.old_name {arg.to_string()} else {user_data.name + " " + arg};
                }
            }
        }
    } else {
        if empty_data {
            msg.channel_id.say(&ctx, "send help!")?;
            return Ok(());
        } else {
            let current_conf = format!("
Your current configuration:
```User ID: '{}'
Username: '{}'
Mode ID: '{}'
Show PP? '{}'
Short recent? '{}'```",
                user_data.osu_id, user_data.name, user_data.mode.unwrap(), user_data.pp.unwrap(), user_data.short_recent.unwrap()
            );

            msg.channel_id.say(&ctx, current_conf)?;
            return Ok(());
        }
    }
    user_data.osu_id = get_osu_id(&user_data.name)?;

    let current_conf = format!("
Successfully changed your configuration to this:
```User ID: '{}'
Username: '{}'
Mode ID: '{}'
Show PP? '{}'
Short recent? '{}'```",
        user_data.osu_id, user_data.name, user_data.mode.unwrap(), user_data.pp.unwrap(), user_data.short_recent.unwrap()
    );
    msg.channel_id.say(&ctx, current_conf)?;


    /*
    let name = "Hello";
    let data = None::<&[u8]>;

    client.execute(
        "INSERT INTO person (name, data) VALUES ($1, $2)",
        &[&name, &data],
    )?;
    */

    //msg.channel_id.say(&ctx, format!("{:?}", arguments))?;
    Ok(())
}
