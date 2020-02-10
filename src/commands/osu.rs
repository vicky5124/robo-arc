use crate::{
    DatabaseConnection,
    Tokens
};
use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};
use regex::Regex;

use reqwest;
use serde::Deserialize;



// JSON Structure of the osu! user API request.
#[derive(Deserialize, PartialEq)]
struct OsuUser {
    user_id: String,
}

// Data Structure of the data obtained on the database.
#[derive(Default)] // Default is a trait that sets the default value for each type.
struct OsuUserData {
    osu_id: i32, // 0
    name: String, // String::new()
    old_name: String,
    mode: Option<i32>, // None
    pp: Option<bool>,
    short_recent: Option<bool>,
}

// This function simply calls the osu! api to get the id of the user from a username.
fn get_osu_id(name: &String, osu_key: String) -> Result<i32, Box<dyn std::error::Error>> {
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
#[description = "Command to configure an osu! user for the bot to know about your prefferences.
This supports various keyword parameters, this are:
`pp=` To show or not show any pp related features for your account.
`mode=` To set your osu! gamemode.
`short_recent=` To display the short version of the recent command with less information, but more cozy.

- Everything else that is not keyworded will become your username.
- Keyword arguments are not required, they will default to `true, std, true` respectively.

Example usages:
`n!osuc Majorowsky`
`n!osuc nitsuga5124 pp=false short_recent=yes`
`n!osuc [ Frost ] mode=mania pp=yes recent=false`"]
#[aliases("osuc", "config_osu", "configosu", "configureosu", "configo", "setosu", "osuset", "set_osu", "osu_set")]
fn configure_osu(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {
    let client;
    let osu_key;
    {
        let data = ctx.data.read(); // set inmutable global data.
        osu_key = data.get::<Tokens>().unwrap().clone(); // get the osu! api token from the global data.
    }
    let mut data = ctx.data.write(); // set mutable global data.
    client = data.get_mut::<DatabaseConnection>().unwrap(); // get the database connection from the global data.

    let author_id = msg.author.id.as_u64().clone() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let data = client.query("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", // query the SQL to the database.
                            &[&author_id])?; // The arguments on this array will go to the respective calls as $ in the database (arrays start at 1 in this case reeeeee)
    let empty_data: bool;

    let mut user_data = OsuUserData::default(); // generate a basic structure with the default values.

    if !data.is_empty() { // if the data is not empty, aka if the user is on the database already
        empty_data = false;
        // Parses the database result into each of the pieces of data on the structure.
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
    
    // if there where arguments on the command (aka the user wants to modify a value)
    if arguments.len() > 0 {
        // Transforms the given arguments as a vector
        let args = arguments.raw_quoted().collect::<Vec<&str>>();
        
        // iterates over all the arguments on the list
        for arg in args {
            // if the argument is the keyword PP
            if arg.starts_with("pp=") {
                // Split the argument on the first = and get everything after it.
                let x: &str = arg.split("=").nth(1).unwrap();
                // Match the text after =
                user_data.pp = match x {
                    // if x == "n" || x == "no" ... {user_data.pp = Some(false)}
                    "n" | "no" | "false" | "0" => Some(false),
                    // else Some(true)
                    _ => Some(true)
                }

            // if the argument starts with the keyword short_recent OR recent
            } else if arg.starts_with("short_recent=") || arg.starts_with("recent=") { 
                let x: &str = arg.split("=").nth(1).unwrap();
                user_data.short_recent = match x {
                    "n" | "no" | "false" | "0" => Some(false),
                    _ => Some(true)
                }

            } else if arg.starts_with("mode=") { 
                let x: &str = arg.split("=").nth(1).unwrap();
                user_data.mode = match x {
                    "0" | "std" | "standard" => Some(0),
                    "1" | "taiko" => Some(1),
                    "2" | "ctb" | "catch" => Some(2),
                    "3" | "mania" => Some(3),
                    _ => Some(0)
                }
            
            // this triggers if the argument was not a keyword argument and adds the argument to
            // the username adding a space.
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
            // sends the help of the command
            msg.channel_id.say(&ctx, "send help!")?;
            return Ok(());
        } else {
            // gets the current configuration of the user
            let current_conf = format!("
Your current configuration:
```User ID: '{}'
Username: '{}'
Mode ID: '{}'
Show PP? '{}'
Short recent? '{}'```",
                user_data.osu_id, user_data.name, user_data.mode.unwrap(), user_data.pp.unwrap(), user_data.short_recent.unwrap()
            );
            // and sends it.
            msg.channel_id.say(&ctx, current_conf)?;
            return Ok(());
        }
    }
    // calls the get_osu_id function to get the id of the user.
    user_data.osu_id = get_osu_id(&user_data.name, osu_key)?;

    // applies the default values in case of being not specified.
    user_data.pp = match &user_data.pp {
        None => Some(true),
        Some(b) => Some(b.clone()),
    };
    user_data.mode = match &user_data.mode {
        None => Some(0),
        Some(b) => Some(b.clone()),
    };
    user_data.short_recent = match &user_data.short_recent {
        None => Some(true),
        Some(b) => Some(b.clone()),
    };

    if empty_data {
        // inserts the data because the user is new.
        client.execute(
            "INSERT INTO osu_user (osu_id, osu_username, pp, mode, short_recent, discord_id) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&user_data.osu_id, &user_data.name, &user_data.pp.unwrap(), &user_data.mode.unwrap(), &user_data.short_recent.unwrap(), &author_id]
        )?;

    } else {
        // updates the database with the new user data.
        client.execute(
            "UPDATE osu_user SET osu_id = $1, osu_username = $2, pp = $3, mode = $4, short_recent = $5 WHERE discord_id = $6",
            &[&user_data.osu_id, &user_data.name, &user_data.pp.unwrap(), &user_data.mode.unwrap(), &user_data.short_recent.unwrap(), &author_id]
        )?;
    }
   
    // if the id obtained is 0, it means the user doesn't exist.
    if user_data.osu_id == 0 {
        msg.channel_id.say(&ctx, "It looks like your osu ID is 0, Is the Username correct?")?;
    }

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

    Ok(())
}
