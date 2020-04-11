use crate::{
    ConnectionPool,
    AnnoyedChannels,
};

use sqlx;
use futures::TryStreamExt;
use futures::stream::StreamExt;

use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    utils::{
        content_safe,
        ContentSafeOptions,
    },
};

async fn set_best_tags(sex: &str, ctx: &mut Context, msg: &Message, mut tags: String) -> Result<(), Box<dyn std::error::Error>> {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let user_id = msg.author.id.0 as i64;

    let data = sqlx::query!("SELECT best_boy, best_girl FROM best_bg WHERE user_id = $1", user_id)
        .fetch_optional(pool)
        .await?;

    if let None = data {
        if sex == "boy" {
            // insert +1boy
            tags += " 1boy";

            sqlx::query!("INSERT INTO best_bg (best_boy, user_id) VALUES ($1, $2)", &tags, user_id)
                .execute(pool)
                .await?;

            msg.reply(&ctx, format!("Successfully set your husbando to `{}`", &tags)).await?;
        } else if sex == "girl" {
            // insert +1girl
            tags += " 1girl";

            sqlx::query!("INSERT INTO best_bg (best_girl, user_id) VALUES ($1, $2)", &tags, user_id)
                .execute(pool)
                .await?;

            msg.reply(&ctx, format!("Successfully set your waifu to `{}`", &tags)).await?;
        }
    } else if sex == "boy" {
        // update +1boy
        tags += " 1boy";

        sqlx::query!("UPDATE best_bg SET best_boy = $1 WHERE user_id = $2", &tags, user_id)
            .execute(pool)
            .await?;

        msg.reply(&ctx, format!("You successfully broke up with your old husbando, now your husbando is `{}`", &tags)).await?;
    } else if sex == "girl" {
        // update +1girl
        tags += " 1girl";

        sqlx::query!("UPDATE best_bg SET best_girl = $1 WHERE user_id = $2", &tags, user_id)
            .execute(pool)
            .await?;

        msg.reply(&ctx, format!("You successfully broke up with your old waifu, now your waifu is `{}`", &tags)).await?;
    }

    Ok(())
}

/// Configures aspects of the bot tied to your account.
///
/// Configurable aspects:
/// `best_girl`: Toggles the annoying features on or off.
/// `best_boy`: Toggles the annoying features on or off.
/// `booru`: Sets the booru to be used for the best_X commands ~~and `picture`~~
#[command]
#[sub_commands(best_boy, best_girl, booru)]
async fn user(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[aliases(husbando, husband)]
async fn best_boy(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("boy", ctx, msg, args.message().to_string()).await?)
}

#[command]
#[aliases(waifu, wife)]
async fn best_girl(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("girl", ctx, msg, args.message().to_string()).await?)
}

#[command]
async fn booru(ctx: &mut Context, msg: &Message, raw_args: Args) -> CommandResult {
    let booru = raw_args.message().to_lowercase();

    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let user_id = msg.author.id.0 as i64;

    let data = sqlx::query!("SELECT best_boy, best_girl FROM best_bg WHERE user_id = $1", user_id)
        .fetch_optional(pool)
        .boxed()
        .await?;

    if let None = data { 
        if booru.as_str() == "" {
            msg.reply(&ctx, "Please, specify the booru to set as your default.").await?;
            return Ok(());
        }
        sqlx::query!("INSERT INTO best_bg (booru, user_id) VALUES ($1, $2)", &booru, user_id)
            .execute(pool)
            .await?;

        msg.reply(&ctx, format!("Successfully set your main booru to `{}`", &booru)).await?;
    } else {
        if booru.as_str() == "" {return Ok(());}

        sqlx::query!("UPDATE best_bg SET booru = $1 WHERE user_id = $2", &booru, user_id)
            .execute(pool)
            .await?;

        msg.reply(&ctx, format!("Successfully edited your main booru to `{}`", &booru)).await?;
    }
    Ok(())
}


/// Configures the bot for the channel it was invoked on.
///
/// Configurable aspects:
/// `annoy`: Toggles the annoying features on or off.
#[command]
#[required_permissions(MANAGE_CHANNELS)]
#[only_in("guilds")]
#[sub_commands(annoy, notifications)]
async fn channel(_ctx: &mut Context, _message: &Message, _args: Args) -> CommandResult {
    Ok(())
}

//#[derive(Default)]
//struct NewPosts<'a> {
//    booru_url: &'a str,
//    tags: &'a str,
//    remove_hook: bool,
//    hook: &'a str,
//    remove_channel: bool,
//    channel: u64,
//}

/// Configure the notifications of the channel.
/// WIP
#[command]
async fn notifications(_ctx: &mut Context, _message: &Message, _args: Args) -> CommandResult {
//    // TODO: change this to more defined presets lol
//    let mut msg = message.channel_id.send_message(&ctx, |m| {
//        m.content(format!("<@{}>", message.author.id));
//        m.embed(|e| {
//            e.title("Say the number of option that you want");
//            e.description("__Choose what notification type you want:__\n1: yande.re")
//        })
//    }).await?;
//    // change this to an enum when i add other services.
//    let mut data = NewPosts::default();
//    println!("1");
//    loop {
//        println!("2");
//        match message.author.await_reply(&ctx).timeout(Duration::from_secs(120)).await {
//            None => break,
//            Some(answ) => {
//                println!("3");
//                if answ.content == "1" {
//                    println!("4");
//                    data.booru_url = "yande.re";
//                    msg.edit(&ctx, |m| {
//                        m.content(format!("<@{}>", message.author.id));
//                        m.embed(|e| {
//                            e.title("Say the numbner of option that you want");
//                            e.description("__What delivery system do you want to use:__\n1: Webhook\n2: Bot")
//                        })
//                    }).await?;
//                    match message.author.await_reply(&ctx).timeout(Duration::from_secs(120)).await {
//                        None => break,
//                        Some(answ) => {
//                            if answ.content == "1" {
//                                msg.edit(&ctx, |m| {
//                                    m.content(format!("<@{}>", message.author.id));
//                                    m.embed(|e| {
//                                        e.title("Say the numbner of option that you want");
//                                        e.description("__What delivery system do you want to use:__\n1: Create hook.\n2: Remove current hook from this channel.")
//                                    })
//                                }).await?;
//                                match message.author.await_reply(&ctx).timeout(Duration::from_secs(120)).await {
//                                    None => break,
//                                    Some(answ) => {
//                                        msg.channel_id.say(&ctx, &answ.content).await?;
//                                        break;
//                                    },
//                                }
//                            } else if answ.content == "2" {
//                                msg.edit(&ctx, |m| {
//                                    m.content(format!("<@{}>", message.author.id));
//                                    m.embed(|e| {
//                                        e.title("Say the numbner of option that you want");
//                                        e.description("__What delivery system do you want to use:__\n1: Not yet implemented.")
//                                    })
//                                }).await?;
//                                break;
//                                //match message.author.await_reply(&ctx).timeout(Duration::from_secs(120)).await {
//                                //    None => break,
//                                //    Some(answ) => {
//                                //        msg.channel_id.say(&ctx, &answ.content).await?;
//                                //        break;
//                                //    },
//                                //}
//                            }
//                        }
//                    }
//                }
//            }
//        }
//    }
    Ok(())
}

/// Toggles the annoying features on or off.
#[command]
async fn annoy(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let channel_id = msg.channel_id.0 as i64;

    let data = sqlx::query!("SELECT channel_id FROM annoyed_channels WHERE channel_id = $1", channel_id)
        .fetch_optional(pool)
        .await?;

    if let Some(_) = data {
        sqlx::query!("DELETE FROM annoyed_channels WHERE channel_id IN ($1)", channel_id)
            .execute(pool)
            .await?;

        msg.channel_id.say(&ctx, format!("Successfully removed `{}` from the list of channels that allows the bot to do annoying features.", msg.channel_id.name(&ctx).await.unwrap())).await?;

    } else {
        sqlx::query!("INSERT INTO annoyed_channels (channel_id) VALUES ($1)", channel_id)
            .execute(pool)
            .await?;

        msg.channel_id.say(&ctx, format!("Successfully added `{}` to the list of channels that allows the bot to do annoying features.", msg.channel_id.name(&ctx).await.unwrap())).await?;
    }

    {
        let mut raw_annoyed_channels = sqlx::query!("SELECT channel_id from annoyed_channels")
            .fetch(pool)
            .boxed();

        let channels_lock = rdata.get::<AnnoyedChannels>().unwrap();
        let mut channels = channels_lock.write().await;
        channels.clear();

        while let Some(row) = raw_annoyed_channels.try_next().await? {
            channels.insert(row.channel_id as u64);
        } 
    }
    Ok(())
}

/// Configures the bot for the guild/server it was invoked on.
///
/// Configurable aspects:
/// `prefix`: Changes the bot prefix for the guild.
#[command]
#[required_permissions(MANAGE_GUILD)]
#[only_in("guilds")]
#[aliases(server)]
#[sub_commands(prefix)]
async fn guild(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[min_args(1)]
async fn prefix(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(&ctx, "Invalid prefix was given").await?;
        return Ok(());
    }
    let prefix = args.message();

    // change prefix on db 
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let guild_id = msg.guild_id.unwrap().0 as i64;

    let data = sqlx::query!("SELECT prefix FROM prefixes WHERE guild_id = $1", guild_id)
        .fetch_optional(pool)
        .boxed()
        .await?;

    if let None = data {
        sqlx::query!("INSERT INTO prefixes (guild_id, prefix) VALUES ($1, $2)", guild_id, &prefix)
            .execute(pool)
            .await?;
    } else {
        sqlx::query!("UPDATE prefixes SET prefix = $2 WHERE guild_id = $1", guild_id, &prefix)
            .execute(pool)
            .await?;
    }

    let content_safe_options = ContentSafeOptions::default();
    let bad_success_message = format!("Successfully changed your prefix to `{}`", prefix);
    let success_message = content_safe(&ctx, bad_success_message, &content_safe_options).await;
    msg.reply(&ctx, success_message).await?;
    Ok(())
}
