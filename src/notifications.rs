use crate::{
    ConnectionPool,
    Tokens,
    SentTwitchStreams,
    VoiceManager,
    utils::booru::{
        SAFE_BANLIST,
        UNSAFE_BANLIST,
    },
};
use std::{
    time::Duration,
    sync::Arc,
    //collections::HashMap,
};

use sqlx;
use futures::TryStreamExt;
use futures::stream::StreamExt;
use serde::Deserialize;
use reqwest::{
    Client as ReqwestClient,
    Url,
    header::*,
};

use tracing::{
    info,
    error,
};

use serenity::{
    prelude::{
        Context,
        RwLock,
    },
    model::{
        id::ChannelId,
        channel::Embed,
    }
};

#[derive(Deserialize)]
pub struct Post {
    sample_url: String,
    pub md5: String,
    id: u64,
    tags: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct TwitchStreamData {
    user_id: String,
    user_name: String,
    game_id: String,
    title: String,
}

#[derive(Deserialize, Debug)]
struct TwitchUser {
    profile_image_url: String,
}

#[derive(Deserialize, Debug)]
struct TwitchGame {
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TwitchGameData {
    data: Vec<TwitchGame>,
}

#[derive(Deserialize, Debug)]
struct TwitchStreams {
    data: Vec<TwitchStreamData>,
}

#[derive(Deserialize, Debug)]
struct TwitchUserData {
    data: Vec<TwitchUser>,
}

async fn check_new_posts(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let data_read = ctx.data.read().await;
    let pool = data_read.get::<ConnectionPool>().unwrap();

    let mut data = sqlx::query!("SELECT * FROM new_posts")
        .fetch(pool)
        .boxed();


    while let Some(i) = data.try_next().await? {
        let base_url = i.booru_url;
        let tags = i.tags;
        let webhooks = i.webhook.unwrap_or(Vec::new());
        let channels = i.channel_id.unwrap_or(Vec::new());
        let mut md5s = i.sent_md5.unwrap_or(vec![]);

        if base_url == "yande.re" {
            let url = Url::parse_with_params("https://yande.re/post/index.json",
                                             &[("tags", &tags), ("limit", &"100".to_string())])?;
            let resp = reqwest::get(url)
                .await?
                .json::<Vec<Post>>()
                .await?;

            for post in resp {
                if !md5s.contains(&post.md5) {
                    for channel in &channels {
                        let real_channel = ChannelId(*channel as u64).to_channel(&ctx).await?;
                        let mut is_unsafe = false;

                        if real_channel.is_nsfw().await || real_channel.guild().is_none() {
                            for tag in post.tags.split(' ').into_iter() {
                                if UNSAFE_BANLIST.contains(&tag) {
                                    is_unsafe = true;
                                }
                            }
                        } else {
                            for tag in post.tags.split(' ').into_iter() {
                                if SAFE_BANLIST.contains(&tag) {
                                    is_unsafe = true;
                                }
                            }
                        }

                        if !is_unsafe {
                            if let Err(why) = ChannelId(*channel as u64).send_message(&ctx, |m|{
                                m.embed(|e| {
                                    e.title("Original Post");
                                    e.url(format!("https://yande.re/post/show/{}", post.id));
                                    e.image(post.sample_url.clone())
                                })
                            }).await {
                                eprintln!("Error while sending message >>> {}", why);
                            };
                        }
                    }

                    let allow_hooks = {
                        let read_data = ctx.data.read().await;
                        let config = read_data.get::<Tokens>().unwrap();
                        config["webhook_notifications"].as_bool().unwrap()
                    };

                    if allow_hooks {
                        for webhook in &webhooks {
                            let mut split = webhook.split('/');
                            let id = split.nth(5).unwrap().parse::<u64>()?;
                            let token = split.nth(0).unwrap();

                            let hook = &ctx.http.get_webhook_with_token(id, token).await?;

                            let embed = Embed::fake(|e| {
                                e.title("Original Post");
                                e.url(format!("https://yande.re/post/show/{}", post.id));
                                e.image(post.sample_url.clone())
                            });
                            
                            hook.execute(&ctx.http, false, |m|{
                                m.embeds(vec![embed])
                            }).await?;
                        }
                    }

                    &md5s.push(post.md5);
                    sqlx::query!(
                        "UPDATE new_posts SET sent_md5 = $1 WHERE booru_url = $2 AND tags = $3",
                        &md5s, &base_url, &tags
                    ).execute(pool).await?;
                }
            }
        }
    }
    Ok(())
}

#[inline]
async fn check_changes(data: &TwitchStreamData, sent_streams: Arc<RwLock<Vec<TwitchStreamData>>>) -> bool {
    for i in sent_streams.read().await.iter() {
        if i.user_id == data.user_id && i != data {
            return true;
        }
    }
    false
}

async fn check_twitch_livestreams(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let token = {
        let data_read = ctx.data.read().await;
        let tokens = data_read.get::<Tokens>().unwrap();
        tokens["twitch"].as_str().unwrap().to_string()
    };

    let data_read = ctx.data.read().await;

    let pool = data_read.get::<ConnectionPool>().unwrap();
    let sent_streams = data_read.get::<SentTwitchStreams>().unwrap();

    let mut data = sqlx::query!("SELECT * FROM streamers")
        .fetch(pool)
        .boxed();

    while let Some(i) = data.try_next().await? {
        let reqwest = ReqwestClient::new();
        let url = Url::parse_with_params("https://api.twitch.tv/helix/streams", &[("user_login", &i.streamer)])?;
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());

        let resp = reqwest.get(url)
            .headers(headers.clone())
            .send()
            .await?
            .json::<TwitchStreams>()
            .await?;

        let stream_data = resp.data;
        if !stream_data.is_empty() && i.is_live {
            if check_changes(&stream_data[0], Arc::clone(sent_streams)).await {
                let mut data = sqlx::query!("SELECT * FROM streamer_notification_channel WHERE streamer = $1", &i.streamer)
                    .fetch(pool)
                    .boxed();

                while let Some(notification_place) = data.try_next().await? {
                    let url = format!("https://api.twitch.tv/helix/games?id={}", stream_data[0].game_id);
                    let game_resp = reqwest.get(&url)
                        .headers(headers.clone())
                        .send()
                        .await?
                        .json::<TwitchGameData>()
                        .await?;

                    let url = format!("https://api.twitch.tv/helix/users?id={}", stream_data[0].user_id);
                    let user_resp = reqwest.get(&url)
                        .headers(headers.clone())
                        .send()
                        .await?
                        .json::<TwitchUserData>()
                        .await?;

                    let game_name = game_resp.data[0].name.clone().unwrap_or("No Game".to_string());
                    let streamer_name = notification_place.streamer.clone();

                    if let Ok(mut message) = ctx.http.get_message(notification_place.channel_id.unwrap() as u64, notification_place.message_id.unwrap() as u64).await
                    {
                        let _ = message.edit(&ctx, |m| {
                            if let Some(role_id) = notification_place.role_id.to_owned() {
                                m.content(format!("<@&{}>", role_id));
                            }
                            m.embed( |e| {
                                if !notification_place.use_default {
                                    e.description(notification_place.live_message.unwrap());
                                } else {
                                    e.description(i.live_message.clone().unwrap());
                                }
                                e.author(|a| {
                                    a.name(&i.streamer);
                                    a.icon_url(&user_resp.data[0].profile_image_url);
                                    a.url(format!("https://www.twitch.tv/{}", &i.streamer))
                                });
                                e.url(format!("https://www.twitch.tv/{}", &i.streamer));
                                e.title(stream_data[0].title.to_string());
                                e.field(
                                    "Game",
                                    game_name,
                                    true,
                                )
                            })
                        }).await;
                    }

                    let sent_stream_data = {
                        let new_vec = sent_streams.read().await;
                        new_vec.clone()
                    };
                    for (index, val) in sent_stream_data.iter().enumerate() {
                        if val.user_name == streamer_name {
                            sent_streams.write().await.remove(index);
                        }
                    }
                    sent_streams.write().await.push(stream_data[0].clone());
                }
            }
        } else if !stream_data.is_empty() && !i.is_live {
            let mut data = sqlx::query!("SELECT * FROM streamer_notification_channel WHERE streamer = $1", &i.streamer)
                .fetch(pool)
                .boxed();

            while let Some(notification_place) = data.try_next().await? {
                let url = format!("https://api.twitch.tv/helix/games?id={}", stream_data[0].game_id);
                let game_resp = reqwest.get(&url)
                    .headers(headers.clone())
                    .send()
                    .await?
                    .json::<TwitchGameData>()
                    .await?;

                let url = format!("https://api.twitch.tv/helix/users?id={}", stream_data[0].user_id);
                let user_resp = reqwest.get(&url)
                    .headers(headers.clone())
                    .send()
                    .await?
                    .json::<TwitchUserData>()
                    .await?;

                let game_name = game_resp.data[0].name.clone().unwrap_or("No Game".to_string());
                let streamer_name = notification_place.streamer.clone();

                let message = ChannelId(notification_place.channel_id.unwrap() as u64).send_message(&ctx, |m| {
                    if let Some(role_id) = notification_place.role_id {
                        m.content(format!("<@&{}>", role_id));
                    }
                    m.embed( |e| {
                        if !notification_place.use_default {
                            e.description(notification_place.live_message.unwrap());
                        } else {
                            e.description(i.live_message.clone().unwrap());
                        }
                        e.author(|a| {
                            a.name(&i.streamer);
                            a.icon_url(&user_resp.data[0].profile_image_url);
                            a.url(format!("https://www.twitch.tv/{}", &i.streamer))
                        });
                        e.url(format!("https://www.twitch.tv/{}", &i.streamer));
                        e.title(stream_data[0].title.to_string());
                        e.field(
                            "Game",
                            game_name,
                            true,
                        )
                    })
                }).await;
                if let Ok(message_ok) = message {
                    sqlx::query!("UPDATE streamer_notification_channel SET message_id = $1 WHERE channel_id = $2 AND streamer = $3", message_ok.id.as_u64().to_owned() as i64, message_ok.channel_id.0 as i64, &i.streamer)
                        .execute(pool)
                        .await?;
                }

                let sent_stream_data = {
                    let new_vec = sent_streams.read().await;
                    new_vec.clone()
                };
                for (index, val) in sent_stream_data.iter().enumerate() {
                    if val.user_name == streamer_name {
                        sent_streams.write().await.remove(index);
                    }
                }
                sent_streams.write().await.push(stream_data[0].clone());
            }

            sqlx::query!("UPDATE streamers SET is_live = true WHERE streamer = $1", &i.streamer)
                .execute(pool)
                .await?;


        } else if stream_data.is_empty() && i.is_live {
            let mut data = sqlx::query!("SELECT * FROM streamer_notification_channel WHERE streamer = $1", i.streamer)
                .fetch(pool)
                .boxed();

            while let Some(notification_place) = data.try_next().await? {
                if let Ok(mut message) = ctx.http.get_message(notification_place.channel_id.unwrap_or(0) as u64, notification_place.message_id.unwrap_or(0) as u64).await
                {
                    let _ = message.edit(&ctx, |m| {
                        if let Some(role_id) = notification_place.role_id.to_owned() {
                            m.content(format!("<@&{}>", role_id));
                        }
                        m.embed( |e| {
                            if !notification_place.use_default {
                                e.description(notification_place.not_live_message.unwrap());
                            } else {
                                e.description(i.not_live_message.clone().unwrap());
                            }
                            e.author(|a| {
                                a.name(&i.streamer);
                                a.url(format!("https://www.twitch.tv/{}", &i.streamer))
                            });
                            e.url(format!("https://www.twitch.tv/{}", &i.streamer));
                            e.title("No longer live.")
                        })
                    }).await;
                }
            }
            sqlx::query!("UPDATE streamers SET is_live = false WHERE streamer = $1", i.streamer)
                .execute(pool)
                .await?;
        }
    }
    
    Ok(())
}

async fn check_empty_vc(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let user_id = ctx.cache.read().await.user.id;

    for (guild_id, guild_lock) in &ctx.cache.read().await.guilds {
        let mut manager = manager_lock.lock().await;
        let has_handler = manager.get(guild_id).is_some();

        if has_handler {
            let guild = guild_lock.read().await;
            if let Some(channel) = guild.voice_states.get(&user_id)
                .and_then(|v| v.channel_id) {
                    let guild_channel_lock = {
                        let cache = ctx.cache.read().await;
                        cache.guild_channel(channel).unwrap()
                    };
                    let guild_channel = guild_channel_lock.read().await;

                    if let Ok(members) = guild_channel.members(&ctx).await {
                        if members.len() == 1 {
                            manager.remove(guild_id);
                        }
                    }
            };
        }
    }

    Ok(())
}

pub async fn notification_loop(ctx: Arc<Context>) {
    let ctx = Arc::new(ctx);
    loop {
        info!("Notification loop started.");
        let ctx1 = Arc::clone(&ctx);
        tokio::spawn(async move {
            if let Err(why) = check_new_posts(Arc::clone(&ctx1)).await {
                error!("check_new_posts :: {}", why);
                eprintln!("An error occurred while running check_new_posts() >>> {}", why);
            }
        });

        let ctx2 = Arc::clone(&ctx);
        tokio::spawn(async move {
            if let Err(why) = check_twitch_livestreams(Arc::clone(&ctx2)).await {
                error!("check_twitch_livestreams :: {}", why);
                eprintln!("An error occurred while running check_twitch_livestreams() >>> {}", why);
            }
        });

        let ctx3 = Arc::clone(&ctx);
        tokio::spawn(async move {
            if let Err(why) = check_empty_vc(Arc::clone(&ctx3)).await {
                error!("check_empty_vc :: {}", why);
                eprintln!("An error occurred while running check_empty_vc() >>> {}", why);
            }
        });
        info!("Notification loop finished.");

        tokio::time::delay_for(Duration::from_secs(120)).await;
    }
}

