use crate::{
    utils::booru,
    Booru,
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
use rand::Rng;

use reqwest::{
    blocking::Client as ReqwestClient,
    header::*,
};
use quick_xml::de::from_str;
use serde::{
    Deserialize,
    Serialize,
};

// defining the Post type to be used for the xml deserialized on the Posts vector.
#[derive(Serialize, Deserialize, PartialEq)]
struct Post {
    score: String, // i32
    source: String,
    rating: String,
    sample_url: String,
    file_url: String,
}

// defining the Posts vector to Deserialize the requested xml list.
#[derive(Deserialize, PartialEq)]
struct Posts {
    post: Vec<Post>,
}

pub fn get_booru(ctx: &mut Context, msg: &Message, booru: &Booru, args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Hypno has no source value
    // danbooru has not sample_url value
    // behoimi needs login

    let channel = &ctx.http.get_channel(msg.channel_id.0)?; // Gets the channel object to be used for the nsfw check.
    // Checks if the command was invoked on a DM
    let dm_channel: bool;
    if msg.guild_id == None {
        dm_channel = true;
    } else {
        dm_channel = false;
    }

    let raw_tags = {
        if channel.is_nsfw() || dm_channel {
            let mut raw_tags = booru::obtain_tags_unsafe(args);
            booru::illegal_check_unsafe(&mut raw_tags)
        } else {
            let mut raw_tags = booru::obtain_tags_safe(args);
            booru::illegal_check_safe(&mut raw_tags)
        }
    };

    let tags = raw_tags.iter().map(|x| format!("{}%20", x)).collect::<String>();

    let reqwest = ReqwestClient::new();
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0".parse().unwrap());

    let page = 1;
    let url;

    if booru.typ == 1 {
        url = format!("https://{}/index.php?page=dapi&s=post&q=index&tags={}&pid={}&limit=50", booru.url, tags, page);
    } else if booru.typ == 2 {
        url = format!("https://{}/post/index.xml?tags={}&page={}&limit=50", booru.url, tags, page);
    } else {
        url = "https://safebooru.org/index.php?page=dapi&s=post&q=index".to_string();
    }

    let resp = reqwest.get(&url)
        .headers(headers.clone())
        .send()?
        .text()?;
    
    let xml: Posts = from_str(&resp.to_owned()[..])?;

    // gets a random post from the vector.
    let r = rand::thread_rng().gen_range(0, xml.post.len()); 
    let choice = &xml.post[r];

    // define both url types.
    let full_size = &choice.file_url;
    let sample_size = &choice.sample_url;
    
    // Check if there's a source to get added to the fields.
    let source_avail: bool;
    if &choice.source == &String::from(""){
        source_avail = false;
    } else {
        source_avail = true;
    }
    let source = &choice.source;
    let source_md = format!("[Here]({})", source);

    // Sets the score and rating for ease of uses
    let score = &choice.score;
    let rating = match &choice.rating[..] {
        "s" => "Safe".to_string(),
        "q" => "Questionable".to_string(),
        "e" => "Explicit".to_string(),
        _ => String::new(),
    };

    // Addes a source field to the embed if available.
    let mut fields = vec![
        ("Rating", &rating, true),
        ("Score", &score, true),
    ];
    if source_avail {
        fields.push(("Source", &source_md, true));
    }

    // https://github.com/serenity-rs/serenity/blob/current/examples/11_create_message_builder/src/main.rs
    msg.channel_id.send_message(&ctx, |m| { // say method doesn't work for the message builder.
        m.embed( |e| {
            e.description(format!("[Sample]({}) | [Full Size]({})", &sample_size, &full_size));
            e.image(sample_size);
            e.fields(fields);

            e
        });

        m
    })?;


    Ok(())
}

#[command]
#[aliases("picture", "pic", "booru")]
#[description = "Sends a random picture from the first page of the specified tags on safebooru."]
pub fn booru_command(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}
