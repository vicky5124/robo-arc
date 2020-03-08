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
    source: Option<String>,
    rating: String,
    sample_url: Option<String>,
    file_url: String,
}

// defining the Posts vector to Deserialize the requested xml list.
#[derive(Deserialize, PartialEq)]
struct Posts {
    post: Vec<Post>,
}

pub fn get_booru(ctx: &mut Context, msg: &Message, booru: &Booru, args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: behoimi needs login

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
    } else if booru.typ == 3 {
        url = format!("http://{}/post/index.xml?tags={}&page={}&limit=50", booru.url, tags, page);
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
    let sample_size = if let Some(u) = &choice.sample_url {
        u.to_owned()
    } else {
        full_size.clone()
    };
    
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

    // Check if there's a source to get added to the fields.
    let text;
    if let Some(s) = &choice.source {
        if s != &"".to_string() {
            text = format!("[Here]({})", &s);
            &fields.push(("Source", &text, true));
        }
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
#[aliases("picture", "pic", "booru", "boorus")]
#[description = "Sends a random picture from the first page of the booru selected with the specific tags.
Usage: `(prefix)booru_name tag tag tag`

```
.idol feet -x stockings
.e621 paws
.yandere -x
.r32
```

The currently available boorus are:
__Working:__
`SafeBooru` - Safe only booru.
`Chan` - Largest, most popular booru.
`GelBooru` - One of the most popular boorus.
`KonaChan` - Quality Moderated, Girls only booru.
`YandeRe` - Quality Moderated booru.
`Rule34` - If it exist, there's porn of it.
`DanBooru` - Very popular booru, limited to only 2 tags.
`HypnoBooru` - A booru that hosts all sorts of hypno based content.
`FurryBooru` - Second largest Furry booru.
`RealBooru` - Very large IRL booru.
`Idol` - Largest IRL booru, very asian based.

__Broken:__
`e621` - Largest Furry booru.
`Behoimi` - IRL, Mostly cosplays booru.

Available parameters:
`-x` Explicit
`-q` Questionable
`-s` Safe. 
`-n` Non Safe (Random between E or Q)

Inspired by -GN's [WaifuBot](https://github.com/isakvik/waifubot/)"]
pub fn booru_command(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}
