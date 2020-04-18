use crate::VoiceManager;

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
    voice,
    prelude::Context,
};


#[command]
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
        msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention().await)).await?;
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
    let query = args.message();

    if !query.starts_with("http") {
        msg.channel_id.say(&ctx.http, "Must provide a valid URL").await?;

        return Ok(());
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
        let source = match voice::ytdl(&query).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await?;

                return Ok(());
            },
        };

        handler.play(source);

        msg.channel_id.say(&ctx.http, "Playing song").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await?;
    }

    Ok(())
}

