use crate::{
    VoiceManager,
    Lavalink,
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

use serde_json;
use regex::Regex;

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
            if query.starts_with("http") {
                msg.channel_id.say(&ctx, "Please, put the url between <> so it doesn't embed.").await?;
            }
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
        let data = ctx.data.read().await;
        let lava_client = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");

        let query_information = lava_client.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            msg.channel_id.say(&ctx, "Could not find any video of the search query.").await?;
            return Ok(());
        }

        if let Err(why) = lava_client.play(&handler, &query_information.tracks[0]).await {
            msg.channel_id.say(&ctx, format!("There was an error playing the audio: {}", why)).await?;
            return Ok(());
        };

        msg.channel_id.send_message(&ctx, |m| {
            m.content("Now playing:");
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
        msg.channel_id.say(&ctx, "Please, connect the bot to a voice channel first with `.join`").await?;

        //join(&mut ctx.clone(), msg, args.clone()).await?;
        //play(&mut ctx.clone(), msg, args.clone()).await?;
    }

    Ok(())
}

