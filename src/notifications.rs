use crate::utils::booru::{SAFE_BANLIST, UNSAFE_BANLIST};

use crate::global_data::*;

use std::{
    sync::Arc,
    //collections::HashMap,
    time::Duration,
};

use reqwest::{header::*, Client as ReqwestClient, Url};
use serde::Deserialize;

use serenity::{
    model::{channel::Embed, id::ChannelId},
    prelude::{Context, RwLock},
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
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let data = sqlx::query!("SELECT * FROM new_posts")
        .fetch_all(&pool)
        .await?;

    for i in data {
        let base_url = i.booru_url;
        let tags = i.tags;
        let webhooks = i.webhook.unwrap_or_default();
        let channels = i.channel_id.unwrap_or_default();
        let mut md5s = i.sent_md5.unwrap_or_default();

        if base_url == "yande.re" {
            let url = Url::parse_with_params(
                "https://yande.re/post/index.json",
                &[("tags", &tags), ("limit", &"100".to_string())],
            )?;
            let resp = reqwest::get(url).await?.json::<Vec<Post>>().await?;

            for post in resp {
                if !md5s.contains(&post.md5) {
                    for channel in &channels {
                        let real_channel = ChannelId(*channel as u64).to_channel(&*ctx).await?;
                        let mut is_unsafe = false;

                        if real_channel.is_nsfw() || real_channel.guild().is_none() {
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
                            if let Err(why) = ChannelId(*channel as u64)
                                .send_message(&ctx, |m| {
                                    m.embed(|e| {
                                        e.title("Original Post");
                                        e.url(format!("https://yande.re/post/show/{}", post.id));
                                        e.image(post.sample_url.clone())
                                    })
                                })
                                .await
                            {
                                eprintln!("Error while sending message >>> {}", why);
                            };
                        }
                    }

                    let allow_hooks = {
                        let data_read = ctx.data.read().await;
                        let config = data_read.get::<Tokens>().unwrap();
                        config.webhook_notifications
                    };

                    if allow_hooks {
                        for webhook in &webhooks {
                            let mut split = webhook.split('/');
                            let id = split.nth(5).unwrap().parse::<u64>()?;
                            let token = split.next().unwrap();

                            let embed = Embed::fake(|e| {
                                e.title("Original Post");
                                e.url(format!("https://yande.re/post/show/{}", post.id));
                                e.image(post.sample_url.clone())
                            });

                            if let Ok(hook) = &ctx.http.get_webhook_with_token(id, token).await {
                                hook.execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                                    .await?;
                            }
                        }
                    }

                    md5s.push(post.md5);

                    sqlx::query!(
                        "UPDATE new_posts SET sent_md5 = $1 WHERE booru_url = $2 AND tags = $3",
                        &md5s,
                        &base_url,
                        &tags
                    )
                    .execute(&pool)
                    .await?;
                }
            }
        }
    }
    Ok(())
}

#[inline]
async fn check_changes(
    data: &TwitchStreamData,
    sent_streams: Arc<RwLock<Vec<TwitchStreamData>>>,
) -> bool {
    for i in sent_streams.read().await.iter() {
        if i.user_id == data.user_id && i != data {
            return true;
        }
    }
    false
}

async fn check_twitch_livestreams(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let (pool, sent_streams, token, client_id) = {
        let data_read = ctx.data.read().await;

        let tokens = data_read.get::<Tokens>().unwrap();
        let pool = data_read.get::<DatabasePool>().unwrap();
        let sent_streams = data_read.get::<SentTwitchStreams>().unwrap();

        let token = tokens.twitch.to_string();
        let client_id = tokens.twitch_client_id.to_string();

        (pool.clone(), sent_streams.clone(), token, client_id)
    };

    let data = sqlx::query!("SELECT * FROM streamers")
        .fetch_all(&pool)
        .await?;

    for i in data {
        let reqwest = ReqwestClient::new();
        let url = Url::parse_with_params(
            "https://api.twitch.tv/helix/streams",
            &[("user_login", &i.streamer)],
        )?;
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        headers.insert("Client-ID", client_id.to_string().parse().unwrap());

        let resp = match reqwest
            .get(url)
            .headers(headers.clone())
            .send()
            .await?
            .json::<TwitchStreams>()
            .await
        {
            Ok(x) => x,
            Err(why) => {
                error!("Error quering Helix API: {}", &why);
                continue;
            }
        };

        let stream_data = resp.data;
        if !stream_data.is_empty() && i.is_live {
            if check_changes(&stream_data[0], Arc::clone(&sent_streams)).await {
                let data = sqlx::query!(
                    "SELECT * FROM streamer_notification_channel WHERE streamer = $1",
                    &i.streamer
                )
                .fetch_all(&pool)
                .await?;

                for notification_place in data {
                    let url = format!(
                        "https://api.twitch.tv/helix/games?id={}",
                        stream_data[0].game_id
                    );
                    let game_resp = reqwest
                        .get(&url)
                        .headers(headers.clone())
                        .send()
                        .await?
                        .json::<TwitchGameData>()
                        .await?;

                    let url = format!(
                        "https://api.twitch.tv/helix/users?id={}",
                        stream_data[0].user_id
                    );
                    let user_resp = reqwest
                        .get(&url)
                        .headers(headers.clone())
                        .send()
                        .await?
                        .json::<TwitchUserData>()
                        .await?;

                    let game_name = game_resp.data[0]
                        .name
                        .clone()
                        .unwrap_or_else(|| "No Game".to_string());
                    let streamer_name = notification_place.streamer.clone();

                    if let Ok(mut message) = ctx
                        .http
                        .get_message(
                            notification_place.channel_id.unwrap() as u64,
                            notification_place.message_id.unwrap() as u64,
                        )
                        .await
                    {
                        let _ = message
                            .edit(&*ctx, |m| {
                                if let Some(role_id) = notification_place.role_id.to_owned() {
                                    m.content(format!("<@&{}>", role_id));
                                }
                                m.embed(|e| {
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
                                    e.field("Game", game_name, true)
                                })
                            })
                            .await;
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
            let data = sqlx::query!(
                "SELECT * FROM streamer_notification_channel WHERE streamer = $1",
                &i.streamer
            )
            .fetch_all(&pool)
            .await?;

            for notification_place in data {
                let url = format!(
                    "https://api.twitch.tv/helix/games?id={}",
                    stream_data[0].game_id
                );
                let game_resp = reqwest
                    .get(&url)
                    .headers(headers.clone())
                    .send()
                    .await?
                    .json::<TwitchGameData>()
                    .await?;

                let url = format!(
                    "https://api.twitch.tv/helix/users?id={}",
                    stream_data[0].user_id
                );
                let user_resp = reqwest
                    .get(&url)
                    .headers(headers.clone())
                    .send()
                    .await?
                    .json::<TwitchUserData>()
                    .await?;

                let game_data = game_resp.data.get(0);
                let game_name = if let Some(x) = game_data {
                    x.name.clone().unwrap_or_else(|| "No Game".to_string())
                } else {
                    "Unknown game".to_string()
                };
                let streamer_name = notification_place.streamer.clone();

                let message = ChannelId(notification_place.channel_id.unwrap() as u64)
                    .send_message(&ctx, |m| {
                        if let Some(role_id) = notification_place.role_id {
                            m.content(format!("<@&{}>", role_id));
                        }
                        m.embed(|e| {
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
                            e.field("Game", game_name, true)
                        })
                    })
                    .await;
                if let Ok(message_ok) = message {
                    sqlx::query!("UPDATE streamer_notification_channel SET message_id = $1 WHERE channel_id = $2 AND streamer = $3", message_ok.id.as_u64().to_owned() as i64, message_ok.channel_id.0 as i64, &i.streamer)
                        .execute(&pool)
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

            sqlx::query!(
                "UPDATE streamers SET is_live = true WHERE streamer = $1",
                &i.streamer
            )
            .execute(&pool)
            .await?;
        } else if stream_data.is_empty() && i.is_live {
            let data = sqlx::query!(
                "SELECT * FROM streamer_notification_channel WHERE streamer = $1",
                i.streamer
            )
            .fetch_all(&pool)
            .await?;

            for notification_place in data {
                if let Ok(mut message) = ctx
                    .http
                    .get_message(
                        notification_place.channel_id.unwrap_or(0) as u64,
                        notification_place.message_id.unwrap_or(0) as u64,
                    )
                    .await
                {
                    let _ = message
                        .edit(&*ctx, |m| {
                            if let Some(role_id) = notification_place.role_id.to_owned() {
                                m.content(format!("<@&{}>", role_id));
                            }
                            m.embed(|e| {
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
                        })
                        .await;
                }
            }
            sqlx::query!(
                "UPDATE streamers SET is_live = false WHERE streamer = $1",
                i.streamer
            )
            .execute(&pool)
            .await?;
        }
    }

    Ok(())
}

//async fn _check_empty_vc(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
//    let manager_lock = ctx.data.read().await
//        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
//    let user_id = ctx.cache.current_user().await.id;
//
//    for guild_id in &ctx.cache.guilds().await {
//        let mut manager = manager_lock.lock().await;
//        let has_handler = manager.get(guild_id).is_some();
//
//        if has_handler {
//            let guild = ctx.cache.guild(guild_id).await.unwrap();
//            if let Some(channel) = guild.voice_states.get(&user_id)
//                .and_then(|v| v.channel_id) {
//                    let guild_channel = ctx.cache.guild_channel(channel).await.unwrap();
//
//                    if let Ok(members) = guild_channel.members(&ctx).await {
//                        if members.len() == 1 {
//                            manager.remove(guild_id);
//
//                            let data = ctx.data.read().await;
//                            let lava_client = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
//                            lava_client.write().await.destroy(guild_id).await?;
//                        }
//                    }
//            };
//        }
//    }
//
//    Ok(())
//}

async fn reminder_check(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let reminders = sqlx::query!("SELECT * FROM reminders")
        .fetch_all(&pool)
        .await?;

    for row in reminders {
        if row.date < chrono::offset::Utc::now() {
            let _ = ChannelId(row.channel_id as u64)
                .send_message(&ctx, |m| {
                    m.content(format!("<@!{}>: Reminder!", row.user_id));
                    m.embed(|e| {
                        e.description(if let Some(x) = &row.message {
                            x
                        } else {
                            "No Message."
                        });
                        e.field(
                            "Original Message",
                            format!(
                                "[Jump](https://discord.com/channels/{}/{}/{})",
                                if row.guild_id == 0 {
                                    "@me".to_string()
                                } else {
                                    row.guild_id.to_string()
                                },
                                &row.channel_id,
                                &row.message_id,
                            ),
                            true,
                        )
                    })
                })
                .await;

            sqlx::query!("DELETE FROM reminders WHERE id = $1", row.id)
                .execute(&pool)
                .await?;
        }
    }

    Ok(())
}
async fn unmute_check(ctx: Arc<Context>) -> Result<(), Box<dyn std::error::Error>> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let muted_members = sqlx::query!("SELECT * FROM muted_members")
        .fetch_all(&pool)
        .await?;

    for row in muted_members {
        if row.date < chrono::offset::Utc::now() {
            sqlx::query!("DELETE FROM muted_members WHERE id = $1", row.id)
                .execute(&pool)
                .await?;

            let mut member = if let Ok(x) = ctx
                .http
                .get_member(row.guild_id as u64, row.user_id as u64)
                .await
            {
                x
            } else {
                if let Err(why) = ChannelId(row.channel_id as u64).say(
                    &ctx,
                    format!("Unable to unmute <@{}> from temporal mute.", row.user_id),
                ).await {
                    error!("Unable to send message A: {}", why);
                }
                continue;
            };

            let role_id = {
                let role_row = sqlx::query!(
                    "SELECT role_id FROM muted_roles WHERE guild_id = $1",
                    row.guild_id
                )
                .fetch_optional(&pool)
                .await?;

                if let Some(role_row) = role_row {
                    role_row.role_id as u64
                } else {
                    if let Err(why) = ChannelId(row.channel_id as u64).say(&ctx, format!("Unable to unmute <@{}> from temporal mute because there's no configured role.", row.user_id)).await {
                        error!("Unable to send message B: {}", why);
                    }
                    continue;
                }
            };

            if member.remove_role(&ctx, role_id).await.is_err() {
                if let Err(why) = ChannelId(row.channel_id as u64).say(
                    &ctx,
                    format!("Unable to unmute <@{}> from temporal mute.", row.user_id),
                ).await {
                    error!("Unable to send message C: {}", why);
                }
                continue;
            }

            if let Err(why) = ChannelId(row.channel_id as u64)
                .send_message(&ctx, |m| {
                    m.content(format!("<@!{}> has been unmuted.", row.user_id));
                    m.embed(|e| {
                        e.description(if let Some(x) = &row.message {
                            format!("Mute Reason: {}", x)
                        } else {
                            "No Message.".to_string()
                        });
                        e.field(
                            "Original Message",
                            format!(
                                "[Jump](https://discord.com/channels/{}/{}/{})",
                                if row.guild_id == 0 {
                                    "@me".to_string()
                                } else {
                                    row.guild_id.to_string()
                                },
                                &row.channel_id,
                                &row.message_id,
                            ),
                            true,
                        )
                    })
                })
                .await {
                    error!("Unable to send message D: {}", why);
                }
        }
    }

    Ok(())
}

pub async fn notification_loop(ctx: Arc<Context>) {
    let ctx = Arc::clone(&ctx);
    let ctx_clone = Arc::clone(&ctx);

    tokio::spawn(async move {
        loop {
            info!("Notification loop started.");
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                if let Err(why) = check_new_posts(Arc::clone(&ctx1)).await {
                    error!("check_new_posts :: {}", why);
                    error!(
                        "An error occurred while running check_new_posts() >>> {}",
                        why
                    );
                }
            });

            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                if let Err(why) = check_twitch_livestreams(Arc::clone(&ctx2)).await {
                    error!("check_twitch_livestreams :: {}", why);
                    error!(
                        "An error occurred while running check_twitch_livestreams() >>> {}",
                        why
                    );
                }
            });

            //let ctx3 = Arc::clone(&ctx);
            //tokio::spawn(async move {
            //    if let Err(why) = check_empty_vc(Arc::clone(&ctx3)).await {
            //        error!("check_empty_vc :: {}", why);
            //        eprintln!("An error occurred while running check_empty_vc() >>> {}", why);
            //    }
            //});
            debug!("Notification loop finished.");

            tokio::time::sleep(Duration::from_secs(120)).await;
        }
    });

    tokio::spawn(async move {
        loop {
            let ctx1 = Arc::clone(&ctx_clone);
            tokio::spawn(async move {
                if let Err(why) = reminder_check(Arc::clone(&ctx1)).await {
                    error!("remider_check :: {}", why);
                    error!(
                        "An error occurred while running reminder_check() >>> {}",
                        why
                    );
                }
            });

            let ctx2 = Arc::clone(&ctx_clone);
            tokio::spawn(async move {
                if let Err(why) = unmute_check(Arc::clone(&ctx2)).await {
                    error!("unmute_check :: {}", why);
                    error!("An error occurred while running unmute_check() >>> {}", why);
                }
            });
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });
}
