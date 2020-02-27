use crate::{
    utils::booru,
    Tokens,
};

use std::{
    borrow::Cow,
    io::Read,
};

use serenity::{
    prelude::Context,
    model::channel::Message,
    http::AttachmentType,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};

use serde::Deserialize;
use rand::Rng;
use reqwest::{
    blocking::Client as ReqwestClient,
    header::*,
};

#[derive(Deserialize, PartialEq)]
struct IdolData {
    rating: String,
    sample_url: String,
    file_url: String,
    source: String,
    md5: String,
    file_size: i32,
    fav_count: i32,
    id: i32,
}

#[command]
fn idol(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let (login, pass) = {
        let data = ctx.data.read(); // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap(); 

        let login = tokens["sankaku"]["idol_login"].as_str().unwrap().to_owned();
        let pass = tokens["sankaku"]["idol_passhash"].as_str().unwrap().to_owned();
        (login, pass)
    };

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

    let url = format!("https://iapi.sankakucomplex.com/post/index.json?page=1&limit=50&tags={}&login={}&password_hash={}", tags, login, pass);

    let resp = reqwest.get(&url)
        .headers(headers.clone())
        .send()?
        .json::<Vec::<IdolData>>()?;

    if resp.len() == 0 {
        msg.channel_id.say(&ctx, "No posts match the provided tags.")?;
        return Ok(());
    }

    let choice;
    {
        let mut y = 1;
        loop {
            let r = rand::thread_rng().gen_range(0, resp.len());
            let x = &resp[r];
            y += 1;
            if &x.file_size < &8000000 {
                choice = x;
                break;
            } else if &y > &(&resp.len()*2) {
                msg.channel_id.say(&ctx, "All the content matching the requested tags is too big to be sent.")?;
                return Ok(());
            }
        }
    };

    let sample_url = &format!("https:{}", &choice.sample_url).to_owned()[..];
    let file_url = &format!("https:{}", &choice.file_url).to_owned()[..];
    let mut resp = reqwest.get(sample_url)
        .headers(headers.clone())
        .send()?;

    let mut buf: Vec<u8> = vec![];
    &resp.read_to_end(&mut buf)?;

    let fullsize_tagless = &choice.file_url.split("?").nth(0).unwrap();
    let fullsize_split = fullsize_tagless.split("/").collect::<Vec<&str>>();
    let filename = fullsize_split.get(6).unwrap();

    let attachment = AttachmentType::Bytes {
        data: Cow::from(&buf),
        filename: filename.to_string(),
    };

    let rating = match &choice.rating[..] {
        "s" => "Safe".to_string(),
        "q" => "Questionable".to_string(),
        "e" => "Explicit".to_string(),
        _ => String::new(),
    };

    let score = format!("{}", &choice.fav_count);
    let mut fields = vec![
        ("Rating", &rating, true),
        ("Score", &score, true),
        //("MD5", &choice.md5, true),
    ];

    let source_md = format!("[Here]({})", &choice.source);
    if &choice.source != &"".to_string() {
        &fields.push(("Source", &source_md, true));
    }


    msg.channel_id.send_message(&ctx, |m| {
        m.add_file(attachment);
        m.embed(|e| {
            e.image(format!("attachment://{}", filename));
            e.title("Original Post");
            e.description(format!("[Sample]({}) | [Full Size]({})", &sample_url, &file_url));
            e.url(format!("https://idol.sankakucomplex.com/post/show/{}/", &choice.id));
            e.fields(fields);
            e
        });
        m
    })?;

    Ok(())
}

/*
https://iapi.sankakucomplex.com/post/index.json?page=1&limit=1&tags=rating:safe

[
  {
    "width": 800,
    "in_visible_pool": false,
    "rating": "s",
    "preview_url": "//is.sankakucomplex.com/data/preview/63/7d/637d297b733e47d380bb64b8fce6aa02.jpg",
    "file_size": 138180,
    "is_favorited": false,
    "status": "active",
    "sample_url": "//is.sankakucomplex.com/data/63/7d/637d297b733e47d380bb64b8fce6aa02.jpg?e=1582748303&m=_QJX0YETnpwk79PoeIl6Zg",
    "has_comments": true,
    "md5": "637d297b733e47d380bb64b8fce6aa02",
    "vote_count": 1,
    "change": 1099856,
    "recommended_posts": 0,
    "sample_width": 800,
    "source": "http://www.aliexpress.com/item/32999078659.html",
    "author": "just_juan",
    "created_at": {
      "n": 930537000,
      "json_class": "Time",
      "s": 1582656945
    },
    "has_notes": false,
    "height": 800,
    "parent_id": null,
    "sample_height": 800,
    "preview_width": 150,
    "has_children": false,
    "fav_count": 13,
    "id": 734768,
    "preview_height": 150,
    "file_url": "//is.sankakucomplex.com/data/63/7d/637d297b733e47d380bb64b8fce6aa02.jpg?e=1582748303&m=_QJX0YETnpwk79PoeIl6Zg",
    "total_score": 5
    "tags": [
      {
        "type": 0,
        "count": 272382,
        "name": "cosplay",
        "id": 5
      },
      {
        "type": 0,
        "count": 5387,
        "name": "2girls",
        "id": 6532
      },
      {
        "type": 0,
        "count": 96,
        "name": "model",
        "id": 7994
      },
      {
        "type": 8,
        "count": 2451,
        "name": "1:1_aspect_ratio",
        "id": 26191
      },
      {
        "type": 3,
        "count": 1,
        "name": "citrus",
        "id": 44659
      },
      {
        "type": 4,
        "count": 1,
        "name": "aihara_yuzu",
        "id": 44660
      },
      {
        "type": 4,
        "count": 1,
        "name": "aihara_mei",
        "id": 44661
      }
    ],
  }
] */
