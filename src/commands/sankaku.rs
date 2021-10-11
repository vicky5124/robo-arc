use crate::{
    global_data::Tokens,
    utils::booru,
    utils::booru::{SAFE_BANLIST, UNSAFE_BANLIST},
};

use std::borrow::Cow;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    http::AttachmentType,
    model::channel::Message,
    prelude::Context,
};

use rand::Rng;
use reqwest::{header::*, Client as ReqwestClient, Url};
use serde::Deserialize;

#[derive(Deserialize, PartialEq)]
struct Tag {
    name: Option<String>,
}

#[derive(Deserialize, PartialEq)]
struct SankakuData {
    rating: String,
    sample_url: Option<String>,
    file_url: Option<String>,
    source: Option<String>,
    md5: String,
    file_size: i32,
    fav_count: i32,
    id: i32,
    tags: Vec<Tag>,
}

#[command]
#[aliases(idol_complex, idolcomplex, sankaku_idol, sankakuidol)]
pub async fn idol(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (login, pass) = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap();

        (
            tokens.sankaku.idol_login.to_string(),
            tokens.sankaku.idol_passhash.to_string(),
        )
    };

    let channel = ctx.http.get_channel(msg.channel_id.0).await?; // Gets the channel object to be used for the nsfw check.
                                                                 // Checks if the command was invoked on a DM
    let dm_channel = msg.guild_id == None;

    let raw_tags = {
        if channel.is_nsfw() || dm_channel {
            let mut raw_tags = booru::obtain_tags_unsafe(args).await;
            booru::illegal_check_unsafe(&mut raw_tags).await
        } else {
            let mut raw_tags = booru::obtain_tags_safe(args).await;
            booru::illegal_check_safe(&mut raw_tags).await
        }
    };

    let tags = raw_tags
        .iter()
        .map(|x| format!("{} ", x))
        .collect::<String>();

    let reqwest = ReqwestClient::new();
    let mut headers = HeaderMap::new();

    headers.insert(
        ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (X11; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0"
            .parse()
            .unwrap(),
    );

    let url = Url::parse_with_params(
        "https://iapi.sankakucomplex.com/post/index.json",
        &[
            ("page", "1"),
            ("limit", "50"),
            ("tags", &tags),
            ("login", &login),
            ("password_hash", &pass),
        ],
    )?;

    let resp = reqwest
        .get(url)
        .headers(headers.clone())
        .send()
        .await?
        .json::<Vec<SankakuData>>()
        .await?;

    if resp.is_empty() {
        msg.channel_id
            .say(ctx, "No posts match the provided tags.")
            .await?;
        return Ok(());
    }

    let choice;
    {
        let mut y = 1;
        loop {
            let r = rand::thread_rng().gen_range(0..resp.len());
            let x = &resp[r];
            y += 1;

            if x.sample_url.is_none() || x.file_url.is_none() {
                continue;
            }

            // 8MB
            if x.file_size < 8_000_000 {
                let mut is_unsafe = false;
                if channel.is_nsfw() || dm_channel {
                    for tag in &x.tags {
                        if UNSAFE_BANLIST
                            .contains(&tag.name.as_ref().unwrap_or(&"gore".to_string()).as_str())
                        {
                            is_unsafe = true;
                        }
                    }
                } else {
                    for tag in &x.tags {
                        if SAFE_BANLIST
                            .contains(&tag.name.as_ref().unwrap_or(&"gore".to_string()).as_str())
                            || &x.rating != "s"
                        {
                            is_unsafe = true;
                        }
                    }
                }
                if !is_unsafe {
                    choice = x;
                    break;
                }
            }
            if y > (&resp.len() * 2) {
                msg.channel_id.say(ctx, "All the content matching the requested tags is either too large, unsafe or illegal to be sent.").await?;
                return Ok(());
            }
        }
    };

    let sample_url = &format!("https:{}", &choice.sample_url.as_ref().unwrap()).to_owned()[..];
    let file_url = &format!("https:{}", &choice.file_url.as_ref().unwrap()).to_owned()[..];
    let buf = reqwest
        .get(sample_url)
        .headers(headers.clone())
        .send()
        .await?
        .bytes()
        .await?
        .into_iter()
        .collect::<Vec<u8>>();

    let fullsize_tagless = &choice.file_url.as_ref().unwrap().split('?').next().unwrap();
    let fullsize_split = fullsize_tagless.split('/').collect::<Vec<&str>>();
    let filename = fullsize_split.get(6).unwrap();

    let attachment = AttachmentType::Bytes {
        data: Cow::from(buf),
        filename: (*filename).to_string(),
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

    let source_md = format!("[Here]({})", &choice.source.as_ref().unwrap());
    if choice.source.as_ref().unwrap() != &"".to_string() {
        fields.push(("Source", &source_md, true));
    }

    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file(attachment);
            m.embed(|e| {
                e.image(format!("attachment://{}", filename));
                e.title("Original Post");
                e.description(format!(
                    "[Sample]({}) | [Full Size]({})",
                    &sample_url, &file_url
                ));
                e.url(format!(
                    "https://idol.sankakucomplex.com/post/show/{}/",
                    &choice.id
                ));
                e.fields(fields);
                e
            });
            m
        })
        .await?;

    Ok(())
}

#[command]
#[aliases(
    sankaku,
    complex,
    sc,
    sankakuchan,
    sankakublack,
    sankakuwhite,
    sankaku_chan,
    sankaku_black,
    sankaku_white,
    sankaku_complex
)]
pub async fn chan(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let channel = ctx.http.get_channel(msg.channel_id.0).await?; // Gets the channel object to be used for the nsfw check.
                                                                 // Checks if the command was invoked on a DM
    let dm_channel = msg.guild_id == None;

    let raw_tags = {
        if channel.is_nsfw() || dm_channel {
            let mut raw_tags = booru::obtain_tags_unsafe(args).await;
            booru::illegal_check_unsafe(&mut raw_tags).await
        } else {
            let mut raw_tags = booru::obtain_tags_safe(args).await;
            booru::illegal_check_safe(&mut raw_tags).await
        }
    };

    let mut tags = raw_tags
        .iter()
        .map(|x| format!("{} ", x))
        .collect::<String>();
    tags.pop();

    let reqwest = ReqwestClient::new();
    let mut headers = HeaderMap::new();

    headers.insert(
        ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (X11; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0"
            .parse()
            .unwrap(),
    );

    //page=1&limit=50&tags={}
    let url = Url::parse_with_params(
        "https://capi-v2.sankakucomplex.com/posts",
        &[("page", "1"), ("limit", "50"), ("tags", &tags)],
    )?;

    let raw_resp = reqwest
        .get(url)
        .headers(headers.clone())
        .send()
        .await?
        .json::<Vec<SankakuData>>()
        .await;

    let resp = match raw_resp {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "There's a 4 tag limit to the requests.\nWhat counts as a tag? Most of the tags that have `:` on the name don't count as a tag.").await?;
            return Ok(());
        }
    };

    if resp.is_empty() {
        msg.channel_id
            .say(ctx, "No posts match the provided tags.")
            .await?;
        return Ok(());
    }

    let choice;
    {
        let mut y = 1;
        loop {
            let r = rand::thread_rng().gen_range(0..resp.len());
            let x = &resp[r];
            y += 1;

            if x.sample_url.is_none() || x.file_url.is_none() {
                continue;
            }

            // 8MB
            if x.file_size < 8_000_000 {
                let mut is_unsafe = false;
                if channel.is_nsfw() || dm_channel {
                    for tag in &x.tags {
                        if UNSAFE_BANLIST
                            .contains(&tag.name.as_ref().unwrap_or(&"gore".to_string()).as_str())
                        {
                            is_unsafe = true;
                        }
                    }
                } else {
                    for tag in &x.tags {
                        if SAFE_BANLIST
                            .contains(&tag.name.as_ref().unwrap_or(&"gore".to_string()).as_str())
                            || x.rating != "s"
                        {
                            is_unsafe = true;
                        }
                    }
                }
                if !is_unsafe {
                    choice = x;
                    break;
                }
            }
            if y > (&resp.len() * 2) {
                msg.channel_id.say(ctx, "All the content matching the requested tags is too big to be sent or illegal.").await?;
                return Ok(());
            }
        }
    };

    let sample_url = &choice.sample_url.as_ref().unwrap();
    let file_url = &choice.file_url.as_ref().unwrap();

    let buf = reqwest
        .get(*sample_url)
        .headers(headers.clone())
        .send()
        .await?
        .bytes()
        .await?
        .into_iter()
        .collect::<Vec<u8>>();

    let fullsize_tagless = &choice.file_url.as_ref().unwrap().split('?').next().unwrap();
    let fullsize_split = fullsize_tagless.split('/').collect::<Vec<&str>>();
    let filename = fullsize_split.get(6).unwrap();

    let attachment = AttachmentType::Bytes {
        data: Cow::from(&buf),
        filename: (*filename).to_string(),
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

    let text;
    if let Some(s) = &choice.source {
        if s != &"".to_string() {
            text = format!("[Here]({})", &s);
            fields.push(("Source", &text, true));
        }
    }

    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file(attachment);
            m.embed(|e| {
                e.image(format!("attachment://{}", filename));
                e.title("Original Post");
                e.description(format!(
                    "[Sample]({}) | [Full Size]({})",
                    &sample_url, &file_url
                ));
                e.url(format!(
                    "https://chan.sankakucomplex.com/post/show/{}/",
                    &choice.id
                ));
                e.fields(fields);
                e
            });
            m
        })
        .await?;

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
