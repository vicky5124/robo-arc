use crate::utils::basic_functions::capitalize_first;

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

#[command]
#[aliases("picture", "pic")]
#[description = "Sends a random picture from the first page of the specified tags on safebooru."]
// safebooru command.
// Minimum args is not specified as the arguments are optional.
pub fn safebooru(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {
    let mut tags = vec!["rating:Safe"]; // Setting the default tag to safe rating, to, you know, be safe.
    if arguments.len() > 0 { // There's no point in checking for nsfw status if it's going to be a safe result.
        let mut args = arguments.raw_quoted().collect::<Vec<&str>>(); // Transforms the arguments into a vector for ease of manipulation.
        let channel = &ctx.http.get_channel(msg.channel_id.0)?; // Gets the channel object to be used for the nsfw check.

        // Checks if the command was invoked on a DM
        let dm_channel: bool;
        if msg.guild_id == None {
            dm_channel = true;
        } else {
            dm_channel = false;

        }

        // Allows using the command parameters only if the channel is NSFW of it's a DM.
        if channel.is_nsfw() || dm_channel {
            if args[0] == "-x" {
                &tags.remove(0); // removes the default safe rating
                &tags.push("rating:Explicit"); // adds the explicit rating to the tag list
                &args.remove(0); // removes the parameter from the list of arguments, so it doesn't get added to the tag list later.
            } else if args[0] == "-q" {
                &tags.remove(0);
                &tags.push("rating:Questionable");
                &args.remove(0);

            } else if args[0] == "-n" {
                &tags.remove(0);
                let choices = ["rating:Questionable", "rating:Explicit"];
                let r = rand::thread_rng().gen_range(0, choices.len()); // Generates a random number between 0 and the length of the array. (so either a 0 or a 1)

                let choice = &choices[r]; // indexes the array with the randomly generated number as a random choice of the list.
                &tags.push(choice);
                &args.remove(0);

            } else if args[0] == "-r" {
                &tags.remove(0);
                &args.remove(0);
            }
        }
        
        // Adds every argument that's left to the tag list.
        for arg in args {
            &tags.push(arg);
        }
    }

    // transforms the list of tags into an html friendly string.
    let stringified_tags: String = tags.iter().map(|x| format!("{}%20", x)).collect();
    
    // requests the safebooru api with the specified tags.
    let url = format!("https://safebooru.org/index.php?page=dapi&s=post&q=index&tags={}", stringified_tags);
    let resp = reqwest::blocking::get(&url)?
        .text()?; // serializes the data into a vector with the Data struct type.
    
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
    let rating = capitalize_first(&choice.rating.to_owned()[..]);

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
            e.title("Full Size image.");
            e.url(&full_size);
            e.image(sample_size);
            e.fields(fields);

            e
        });

        m
    })?;

    Ok(())
}
