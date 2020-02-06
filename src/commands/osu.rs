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

    let client = Client::connect(&format!("host=localhost user=postgres password={} dbname=arcbot", psql_password).to_owned()[..], NoTls)?;
    Ok(client)
}

fn get_osu_id(name: &String) -> Result<i32, Box<dyn std::error::Error>> {
    let mut file = File::open("tokens.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let tokens = contents.parse::<Value>().unwrap();
    let osu_key = tokens["osu"].as_str().unwrap();

    let url = format!("https://osu.ppy.sh/api/get_user?k={}&u={}&type=string", osu_key, name);
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
    let data = client.query("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", &[&author_id])?;
    let empty_data: bool;

    let mut oud = OsuUserData::default();
    //println!("{} {} {} {:?} {:?} {:?}", oud.osu_id, oud.name, oud.old_name, oud.mode, oud.pp, oud.short_recent);

    if !data.is_empty() {
        empty_data = false;
        for row in data {
            oud.osu_id = row.get(0);
            oud.name = row.get(1);
            oud.old_name = oud.name.clone();
            oud.mode = row.get(3);
            oud.pp = row.get(2);
            oud.short_recent = row.get(4);
        }
    } else {
        empty_data = true;
    }

    if arguments.len() > 0 {
        let args = arguments.raw_quoted().collect::<Vec<&str>>();

        for arg in args {
            if arg.starts_with("pp=") {
                let x: Vec<&str> = arg.split("=").collect();
                oud.pp = if x[1] == "true" || x[1] == "1" || x[1] == "yes" || x[1] == "y" {Some(true)} else {Some(false)};
            } else if arg.starts_with("short_recent=") || arg.starts_with("recent=") { 
                let x: Vec<&str> = arg.split("=").collect();
                oud.short_recent = if x[1] == "true" || x[1] == "1" || x[1] == "yes" || x[1] == "y" {Some(true)} else {Some(false)};
            } else if arg.starts_with("mode=") { 
                let x: Vec<&str> = arg.split("=").collect();
                if x[1] == "0" || x[1] == "std" || x[1] == "standard" {
                    oud.mode = Some(0);
                } else if x[1] == "1" || x[1] == "taiko" {
                    oud.mode = Some(1);
                } else if x[1] == "2" || x[1] == "ctb" || x[1] == "catch" {
                    oud.mode = Some(2);
                } else if x[1] == "3" || x[1] == "mania" {
                    oud.mode = Some(3);
                }
            } else {
                if empty_data {
                    oud.name += arg;
                } else {
                    oud.name = if oud.name == oud.old_name {arg.to_string()} else {oud.name + " " + arg};
                }
            }
        }
    } else {
        if empty_data {
            msg.channel_id.say(&ctx, "send help!")?;
            return Ok(());
        } else if !empty_data {
            let current_conf = format!(
                "Your current configuration:\n```User ID: '{}'\nUsername: '{}'\nMode ID: '{}'\nShow PP? '{}'\nShort recent? '{}'```",
                oud.osu_id, oud.name, oud.mode.unwrap(), oud.pp.unwrap(), oud.short_recent.unwrap()
            );

            msg.channel_id.say(&ctx, current_conf)?;
            return Ok(());
        }
    }
    oud.osu_id = get_osu_id(&oud.name)?;

    let current_conf = format!(
        "Successfully changed your configuration to this:\n```User ID: '{}'\nUsername: '{}'\nMode ID: '{}'\nShow PP? '{}'\nShort recent? '{}'```",
        oud.osu_id, oud.name, oud.mode.unwrap(), oud.pp.unwrap(), oud.short_recent.unwrap()
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
