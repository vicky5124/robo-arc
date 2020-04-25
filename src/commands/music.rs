use crate::{
    VoiceManager,
    LavalinkSocket,
    Tokens,
};

use serenity::{
    framework::{
        standard::{
            Args, CommandResult,
            macros::command,
        },
    },
    model::{
        channel::Message,
        misc::Mentionable
    },
    prelude::Context,
};

use futures::prelude::*;
use async_tungstenite::tungstenite::Message as TungsteniteMessage;
use reqwest::{
    Client as ReqwestClient,
    header::*,
};

use serde::Deserialize;
use serde_json;
use regex::Regex;

#[derive(Deserialize)]
struct TrackInformation {
    author: String,
    length: u128,
    title: String,
    uri: String,
    identifier: String,
}

#[derive(Deserialize)]
struct Track {
    track: String,
    info:  TrackInformation,
}

#[derive(Deserialize)]
struct VideoData {
    tracks: Vec<Track>,
}

#[command]
#[aliases("connect")]
async fn join(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(&ctx.cache).await {
        Some(guild) => guild,
        None => {
            msg.channel_id.say(&ctx.http, "DMs not supported").await?;

            return Ok(());
        }
    };

    let guild_id = guild.read().await.id;

    let channel_id = guild
        .read()
        .await
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(&ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().await.
        get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;

    if manager.join(guild_id, connect_to).is_some() {
        msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await?;
    } else {
        msg.channel_id.say(&ctx.http, "Error joining the channel").await?;
    }

    Ok(())
}

#[command]
#[aliases("stop", "skip")]
async fn leave(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().await.guild_channel(msg.channel_id) {
        Some(channel) => channel.read().await.guild_id,
        None => {
            msg.channel_id.say(&ctx.http, "DMs not supported").await?;

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);
    } else {
        msg.reply(&ctx, "Not in a voice channel").await?;
    }

    Ok(())
}

#[command]
#[min_args(1)]
async fn play(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut embeded = false;
    let mut query = args.message().to_string();

    if query.starts_with('<') && query.ends_with('>') {
        embeded = true;
        let re = Regex::new("[<>]").unwrap();
        query = re.replace_all(&query, "").into_owned();
    }

    if !embeded {
        if let Err(_) = ctx.http.edit_message(msg.channel_id.0, msg.id.0, &serde_json::json!({"flags" : 4})).await  {
            msg.channel_id.say(&ctx, "Please, put the url between <> so it doesn't embed.").await?;
        }
    }

    let guild_id = match ctx.cache.read().await.guild_channel(msg.channel_id) {
        Some(channel) => channel.read().await.guild_id,
        None => {
            msg.channel_id.say(&ctx.http, "Error finding channel info").await?;

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;

    if let Some(handler) = manager.get_mut(guild_id) {
        let bot_id = {
            let cache_read = ctx.cache.read().await;
            cache_read.user.id.to_string()
        };
        let (host, port, password) = {
            let data_read = ctx.data.read().await;
            let configuration = data_read.get::<Tokens>().unwrap();
            
            let host = configuration["lavalink"]["host"].as_str().unwrap();
            let port = configuration["lavalink"]["port"].as_integer().unwrap();
            let password = configuration["lavalink"]["password"].as_str().unwrap();

            (host.to_string(), port.to_string(), password.to_string())
        };

        let search = {
            if Regex::new(r"(?:https?://)?(?:www\.)?youtu(?:(?:\.be)|(?:be\.com))/(?:watch\?v=)?([^&\s\?]+)").unwrap().is_match(&query) {
                Regex::new(r"(https?://)|(www\.)|(youtu\.?be)|([/])|(\.com?)|(watch\?v=)|(&.*)").unwrap().replace_all(&query, "").to_string()
                
            } else {
                format!("ytsearch:{}", &query)
            }
        };

        let reqwest = ReqwestClient::new();
        let url = &format!("ws://{}:{}/loadtracks?identifier={}", &host, &port, &search);

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", password.parse()?);
        headers.insert("Num-Shards", "1".parse()?);
        headers.insert("User-Id", bot_id.parse()?);

        let raw_resp = reqwest.get(url)
            .headers(headers.clone())
            .send()
            .await?
            .json::<VideoData>()
            .await?;

        if raw_resp.tracks.is_empty() {
            msg.channel_id.say(&ctx, "Could not find any video of the search query.").await?;
            return Ok(());
        }

        let event = format!("{{ 'token' : '{}', 'guild_id' : '{}', 'endpoint' : '{}' }}", handler.token.as_ref().unwrap(), handler.guild_id.0, handler.endpoint.as_ref().unwrap());

        let lava_socket =  {
            let read_data = ctx.data.read().await;
            read_data.get::<LavalinkSocket>().cloned().unwrap()
        };

        let payload = format!("{{ 'op' : 'voiceUpdate', 'guildId' : '{}', 'sessionId' : '{}', 'event' : {} }}", handler.guild_id.0, handler.session_id.as_ref().unwrap(), event);
        {
            let mut ws = lava_socket.lock().await;
            ws.send(TungsteniteMessage::text(payload)).await?;
        }
        let payload = format!("{{ 'op' : 'play', 'guildId' : '{}', 'track' : '{}' }}", handler.guild_id.0, raw_resp.tracks[0].track);
        {
            let mut ws = lava_socket.lock().await;
            ws.send(TungsteniteMessage::text(payload)).await?;
        }
        msg.channel_id.send_message(&ctx, |m| {
            m.content("Now playing:");
            m.embed(|e| {
                e.title(&raw_resp.tracks[0].info.title);
                e.thumbnail(format!("https://i.ytimg.com/vi/{}/hq720.jpg", raw_resp.tracks[0].info.identifier));
                e.url(&raw_resp.tracks[0].info.uri);
                e.footer(|f| f.text(format!("Submited by {}", &msg.author.name)));
                e.field("Uploader", &raw_resp.tracks[0].info.author, true);
                e.field("Length", format!("{}:{}",
                    raw_resp.tracks[0].info.length / 1000  % 3600 /  60,
                    {
                        let x = raw_resp.tracks[0].info.length / 1000 % 3600 % 60;
                        if x < 10 {
                            format!("0{}", x)
                        } else {
                            x.to_string()
                        }
                    }),
                true);
                e
            })
        }).await?;
    } else {
        msg.channel_id.say(&ctx, "Please, connect the bot to a voice channel first with `.join`").await?;

        //join(&mut ctx.clone(), msg, args.clone()).await?;
        //play(&mut ctx.clone(), msg, args.clone()).await?;
    }

    Ok(())
}

