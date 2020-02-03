use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};
use rand::Rng;

use reqwest;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize)]
struct Data {
    image: String,
    hash: String,
    id: u32,
    directory: String,
}

#[command]
#[aliases("picture", "pic")]
#[description = "Sends a random picture from the first page of the specified tags on safebooru."]
pub fn safebooru(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {
    let mut tags = vec!["rating:Safe"];
    if arguments.len() > 0 {
        let mut args = arguments.raw_quoted().collect::<Vec<&str>>();
        let channel = &ctx.http.get_channel(msg.channel_id.0)?;

        let dm_channel: bool;
        if msg.guild_id == None {
            dm_channel = true;
        } else {
            dm_channel = false;

        }

        if channel.is_nsfw() || dm_channel {
            if args[0] == "-x" {
                &tags.remove(0);
                &tags.push("rating:Explicit");
                &args.remove(0);
            } else if args[0] == "-q" {
                &tags.remove(0);
                &tags.push("rating:Questionable");
                &args.remove(0);

            } else if args[0] == "-n" {
                &tags.remove(0);
                let choices = ["rating:Questionable", "rating:Explicit"];
                let r = rand::thread_rng().gen_range(0, choices.len());

                let choice = &choices[r];
                &tags.push(choice);
                &args.remove(0);

            } else if args[0] == "-r" {
                &tags.remove(0);
                &args.remove(0);
            }
        }

        for arg in args {
            &tags.push(arg);
        }
    }

    let stringified_tags: String = tags.iter().map(|x| format!("{}%20", x)).collect();
    
    let url = format!("https://safebooru.org/index.php?page=dapi&s=post&q=index&json=1&tags={}", stringified_tags);
    let resp = reqwest::blocking::get(&url)?
        .json::<Vec<Data>>()?;

    let r = rand::thread_rng().gen_range(0, resp.len());

    let choice = &resp[r];
    
    let full_size = format!("https://safebooru.org//images/{}/{}", choice.directory, choice.image);
    let _ = msg.channel_id.say(&ctx, full_size);

    Ok(())
}
