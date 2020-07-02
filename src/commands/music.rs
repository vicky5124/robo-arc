use crate::{
    VoiceManager,
    Lavalink,
};

use std::{
    sync::Arc,
    time::Duration,
};

use serenity::{
    client::bridge::voice::ClientVoiceManager,
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
use serenity_lavalink::nodes::Node;

use serde_json;
use regex::Regex;

use failure::Error;
use failure::Fail;

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Fail)]
#[fail(display = "Not in a voice channel.")]
struct JoinError;

pub async fn _join(ctx: &Context, msg: &Message, manager: &mut ClientVoiceManager) -> Result<String, Error> {
    let guild = msg.guild(ctx).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Err(JoinError.into());
        }
    };

    if manager.join(guild_id, connect_to).is_some() {
        let data = ctx.data.read().await;
        let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let mut lava_client = lava_client_lock.write().await;
        Node::new(&mut lava_client, guild_id, msg.channel_id);
    } else {
        msg.channel_id.say(ctx, "Error joining the channel").await?;
        return Err(JoinError.into());
    }
    
    Ok(connect_to.mention())
}

/// Joins me to the voice channel you are currently on.
#[command]
#[aliases("connect")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;

    let channel  = _join(ctx, msg, &mut manager).await?;
    msg.channel_id.say(ctx, &format!("Joined {}", channel)).await?;

    Ok(())
}

/// Shuffles the order of the current queue.
#[command]
#[aliases(randomize)]
async fn shuffle(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let mut lava_client = lava_client_lock.write().await;
    if let Some(node) = lava_client.nodes.get_mut(&msg.guild_id.unwrap()) {
        {
            let mut rng = thread_rng();
            let mut queue = node.queue.clone();
            queue.shuffle(&mut rng);
            node.queue = queue.clone();
        }
        msg.channel_id.say(ctx, "Successfully shuffled the queue.").await?;
    };

    Ok(())
}

/// Skips the current song being played.
#[command]
#[aliases(next)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let mut lava_client = lava_client_lock.write().await;
    if let Some(node) = lava_client.nodes.get_mut(&msg.guild_id.unwrap()) {
        node.skip();
        msg.react(ctx, '✅').await?;
    };

    Ok(())
}

/// Displays the current song queue.
#[command]
#[aliases(que)]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let mut lava_client = lava_client_lock.write().await;
    if let Some(node) = lava_client.nodes.get_mut(&msg.guild_id.unwrap()) {
        if !node.queue.is_empty() {
            let mut queue = String::from("```st\n");
            for (index, track) in node.queue.iter().take(10).enumerate() {
                queue +=  &format!("{}: {}\n", index + 1, track.track.info.title);
            }
            
            if queue.len() > 10 {
                queue += &format!("... {}", queue.len());
            }

            queue += "\n```";

            queue = queue.replace("@", "@​");

            msg.channel_id.say(ctx, &queue).await?;
        } else {
            msg.channel_id.say(ctx, "The queue is empty.").await?;
        }
    };

    Ok(())
}

/// Displays the information about the currently playing song.
#[command]
#[aliases(np, nowplaying)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let lava_client = lava_client_lock.read().await;

    if let Some(node) = lava_client.nodes.get(&msg.guild_id.unwrap()) {
        let track = node.now_playing.as_ref();
        if let Some(x) = track {
            let track_info = &x.track.info;
            msg.channel_id.send_message(ctx, |m| {
                m.content("Now playing:");
                m.embed(|e| {
                    e.title(&track_info.title);
                    e.thumbnail(format!("https://i.ytimg.com/vi/{}/hq720.jpg", track_info.identifier));
                    e.url(&track_info.uri);
                    e.footer(|f| f.text(format!("Submited by unknwon")));
                    e.field("Uploader", &track_info.author, true);
                    e.field("Length", format!("{}:{}",
                        track_info.length / 1000  % 3600 /  60,
                        {
                            let x = track_info.length / 1000 % 3600 % 60;
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
            msg.channel_id.say(ctx, "Nothing is playing at the moment.").await?;
        }
    } else {
        msg.channel_id.say(ctx, "Nothing is playing at the moment.").await?;
    }

    Ok(())
}

/// Jumps to the specific time in seconds to the currently playing song.
#[command]
#[min_args(1)]
#[aliases(jump_to, jumpto, scrub)]
async fn seek(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = if let Ok(x) = args.single::<u64>() { x } else {
        msg.reply(&ctx.http, "Provide a valid number of seconds.").await?;
        return Ok(());
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let manager = manager_lock.lock().await;
    let has_handler = manager.get(msg.guild_id.unwrap()).is_some();

    if has_handler {
        let data = ctx.data.read().await;
        let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let lava_client_read = lava_client_lock.read().await.clone();
        let mut lava_client = lava_client_lock.write().await;
        let node = lava_client.nodes.get_mut(&msg.guild_id.unwrap()).unwrap();

        node.seek(&lava_client_read, &msg.guild_id.unwrap(), Duration::from_secs(num)).await?;
        msg.react(ctx, '✅').await?;
    } else {
        msg.reply(&ctx.http, "Not in a voice channel").await?;
    }

    Ok(())
}

/// Stops the current player.
#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let mut lava_client = lava_client_lock.write().await;
    if let Some(node) = lava_client.nodes.get_mut(&msg.guild_id.unwrap()) {
        node.clone().stop(&mut lava_client, &msg.guild_id.unwrap()).await?;
        msg.react(ctx, '✅').await?;
    } else {
        msg.channel_id.say(ctx, "Nothing to stop.").await?;
    };

    Ok(())
}

/// Pauses or resumes the current player.
#[command]
#[aliases(resume, unpause)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let mut lava_client = lava_client_lock.write().await;
    let lava_client_clone = lava_client.clone();
    if let Some(node) = lava_client.nodes.get_mut(&msg.guild_id.unwrap()) {
        let p = !node.clone().is_paused;
        node.set_pause(&lava_client_clone, &msg.guild_id.unwrap(), p).await?;
        msg.react(ctx, '✅').await?;
    } else {
        msg.channel_id.say(ctx, "Nothing to stop.").await?;
    };

    Ok(())
}

/// Disconnects me from the voice channel if im in one.
#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;
    let has_handler = manager.get(msg.guild_id.unwrap()).is_some();

    if has_handler {
        manager.remove(msg.guild_id.unwrap());
        {
            let data = ctx.data.read().await;
            let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
            let mut lava_client = lava_client_lock.write().await;
            let node = lava_client.nodes.get(&msg.guild_id.unwrap()).unwrap().clone();

            node.destroy(&mut lava_client, &msg.guild_id.unwrap()).await?;
        }
        msg.react(ctx, '✅').await?;
    } else {
        msg.reply(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}

/// Adds a song to the queue.
///
/// Usage: `play starmachine2000`
/// or `play https://www.youtube.com/watch?v=dQw4w9WgXcQ`
#[command]
#[min_args(1)]
#[aliases(p)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut embeded = false;
    let mut query = args.message().to_string();

    if query.starts_with('<') && query.ends_with('>') {
        embeded = true;
        let re = Regex::new("[<>]").unwrap();
        query = re.replace_all(&query, "").into_owned();
    }

    if !embeded {
        if let Err(_) = ctx.http.edit_message(msg.channel_id.0, msg.id.0, &serde_json::json!({"flags" : 4})).await  {
            if query.starts_with("http") {
                msg.channel_id.say(ctx, "Please, put the url between <> so it doesn't embed.").await?;
            }
        }
    }

    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id.say(ctx, "Error finding channel info").await?;

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;

    if let Some(handler) = manager.get_mut(guild_id) {
        let data = ctx.data.read().await;
        let lava_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let mut lava_client = lava_lock.write().await;

        let mut iter = 0;
        let query_information = loop {
            iter += 1;
            let res = lava_client.auto_search_tracks(&query).await?;

            if res.tracks.is_empty() {
                if iter == 5 {
                    msg.channel_id.say(&ctx, "Could not find any video of the search query.").await?;
                    return Ok(());
                }
            } else {
                if query.starts_with("http") && res.tracks.len() > 1 {
                    msg.channel_id.say(ctx, "If you'd like to play the entire playlist, use `play_playlist` instead.").await?;
                }
                break res;
            }
        };

        {
            let node = lava_client.nodes.get_mut(&guild_id).unwrap();

            node.play(query_information.tracks[0].clone())
                .queue();
        }
        let node = lava_client.nodes.get(&guild_id).unwrap();

        if !lava_client.loops.contains(&guild_id) {
            node.start_loop(Arc::clone(lava_lock), Arc::new(handler.clone())).await;
        }

        msg.channel_id.send_message(ctx, |m| {
            m.content("Added to queue:");
            m.embed(|e| {
                e.title(&query_information.tracks[0].info.title);
                e.thumbnail(format!("https://i.ytimg.com/vi/{}/hq720.jpg", query_information.tracks[0].info.identifier));
                e.url(&query_information.tracks[0].info.uri);
                e.footer(|f| f.text(format!("Submited by {}", &msg.author.name)));
                e.field("Uploader", &query_information.tracks[0].info.author, true);
                e.field("Length", format!("{}:{}",
                    query_information.tracks[0].info.length / 1000  % 3600 /  60,
                    {
                        let x = query_information.tracks[0].info.length / 1000 % 3600 % 60;
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
        msg.channel_id.say(ctx, "Please, connect the bot to the voice channel you are currently on first with the `join` command.").await?;

        //join(ctx, msg, args.clone()).await?;
        //play(ctx, msg, args.clone()).await?;
    }

    Ok(())
}

/// Adds an entire playlist to the queue.
///
/// Usage: `play https://www.youtube.com/playlist?list=PLTktV6LgA75yif8RR7yUiSttZD7GKtl_5`
#[command]
#[min_args(1)]
#[aliases(playlist, playplaylist, play_list, pl, playl, plist)]
async fn play_playlist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut embeded = false;
    let mut query = args.message().to_string();

    if query.starts_with('<') && query.ends_with('>') {
        embeded = true;
        let re = Regex::new("[<>]").unwrap();
        query = re.replace_all(&query, "").into_owned();
    }
    if !query.starts_with("http") {
        msg.reply(ctx, "Provide a valid URL").await?;
        return Ok(());
    }

    if !embeded {
        if let Err(_) = ctx.http.edit_message(msg.channel_id.0, msg.id.0, &serde_json::json!({"flags" : 4})).await  {
            msg.channel_id.say(ctx, "Please, put the url between <> so it doesn't embed.").await?;
        }
    }

    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id.say(ctx, "Error finding channel info").await?;

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;

    if let Some(handler) = manager.get_mut(guild_id) {
        let data = ctx.data.read().await;
        let lava_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let mut lava_client = lava_lock.write().await;

        let query_information = lava_client.get_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            msg.channel_id.say(&ctx, "Could not find any video of the search query.").await?;
            return Ok(());
        }

        {
            let node = lava_client.nodes.get_mut(&guild_id).unwrap();

            for track in &query_information.tracks {
                node.play(track.clone()).queue();
            }
        }
        let node = lava_client.nodes.get(&guild_id).unwrap();

        if !lava_client.loops.contains(&guild_id) {
            node.start_loop(Arc::clone(lava_lock), Arc::new(handler.clone())).await;
        }

        msg.channel_id.send_message(ctx, |m| {
            m.content("Added playlist to queue.");
            m.embed(|e| {
                e.title("Playlist link");
                e.url(query);
                e.footer(|f| f.text(format!("Submited by {}", &msg.author.name)))
            })
        }).await?;
    } else {
        msg.channel_id.say(ctx, "Please, connect the bot to the voice channel you are currently on first with the `join` command.").await?;

        //join(ctx, msg, args.clone()).await?;
        //play(ctx, msg, args.clone()).await?;
    }

    Ok(())
}
