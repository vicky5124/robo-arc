use crate::{
    utils::booru,
    ConnectionPool,
    AnnoyedChannels,
    notifications::Post,
};

use std::time::Duration;

use sqlx;
use futures::TryStreamExt;
use futures::stream::StreamExt;

use serde_json;
use reqwest::Url;

use serenity::{
    prelude::Context,
    model::channel::Message,
    model::user::User,
    framework::standard::{
        Args,
        Delimiter,
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

async fn yande_re_channel(ctx: &Context, msg: &mut Message, author: &User) -> Result<(), Box<dyn std::error::Error>> {
    msg.edit(ctx, |m| {
        m.content(format!("<@{}>", author.id));
        m.embed(|e| {
            e.title("Say the tag set you would like to get notified about");
            e.description("This supports the same flags as the `.yandere` command.\n\nExample: `feet stockings -x yuri`")
        })
    }).await?;

    if let Some(reply) = author.await_reply(&ctx).timeout(Duration::from_secs(120)).await {
        msg.edit(ctx, |m| {
            m.content(format!("<@{}>", author.id));
            m.embed(|e| {
                e.title("Say the tag set you would like to get notified about");
                e.description(format!("You selected the tags: `{}`", reply.content))
            })
        }).await?;
    }

    Ok(())
}

async fn yande_re_webhook(ctx: &Context, msg: &mut Message, author: &User) -> Result<(), Box<dyn std::error::Error>> {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let hooks = msg.channel_id.webhooks(&ctx).await?;
    let mut existing_hook = false;

    let bot_id = {
        let cache_read = ctx.cache.read().await;
        cache_read.user.id
    };

    let mut hook_index = 0;

    for (index, hook) in hooks.iter().enumerate() {
        if let Some(u) = &hook.user {
            if u.id == bot_id {
                existing_hook = true;
                hook_index = index;
            }
        }
    }

    msg.edit(ctx, |m| {
        m.content(format!("<@{}>", author.id));
        m.embed(|e| {
            e.title("Say the tag set you would like to get notified about");
            e.description("This supports the same flags as the `.yandere` command.\n\nExample: `feet stockings -x yuri`")
        })
    }).await?;
    
    if let Some(reply) = author.await_reply(ctx).timeout(Duration::from_secs(120)).await {
        let args = Args::new(&reply.content, &[Delimiter::Single(' ')]);

        let channel = ctx.http.get_channel(msg.channel_id.0).await?; // Gets the channel object to be used for the nsfw check.
        // Checks if the command was invoked on a DM
        let dm_channel = msg.guild_id == None;
    
        let mut tags = {
            if channel.is_nsfw().await || dm_channel {
                let mut raw_tags = booru::obtain_tags_unsafe(args).await;
                booru::illegal_check_unsafe(&mut raw_tags).await
            } else {
                let mut raw_tags = booru::obtain_tags_safe(args).await;
                booru::illegal_check_safe(&mut raw_tags).await
            }
        };
        tags.sort();

        let mut sorted_tags = tags.iter().map(|i| format!("{} ", i)).collect::<String>();
        sorted_tags.pop();

        msg.edit(ctx, |m| {
            m.content(format!("<@{}>", author.id));
            m.embed(|e| {
                e.title("Say the tag set you would like to get notified about");
                e.description(format!("You selected the tags: `{}`", sorted_tags))
            })
        }).await?;

        let webhook = if existing_hook {
            hooks[hook_index].clone()
        } else {
            let channel_id = msg.channel_id.0;
            let map = serde_json::json!({"name": "Robo Arc: yande.re"});
            
            ctx.http.create_webhook(channel_id, &map).await?
        };

        let hook_url = format!("https://discordapp.com/api/webhooks/{}/{}", webhook.id, webhook.token);

        let query = sqlx::query!("SELECT webhook FROM new_posts WHERE booru_url = 'yande.re' AND tags = $1", &sorted_tags)
            .fetch_optional(pool)
            .await?;

        if let Some(row) = query {
            let hooks_raw = row.webhook;


            let mut hooks = if let Some(mut hooks) = hooks_raw {
                if hooks.contains(&hook_url) {
                    msg.edit(ctx, |m| {
                        m.content(format!("<@{}>", author.id));
                        m.embed(|e| {
                            e.title("It looks like the bot is already posting this tags.");
                            e.description("Would you like stop getting notified?\n\n1: Yes\n2: No")
                        })
                    }).await?;

                    if let Some(reaction) = author.await_reaction(&ctx).timeout(Duration::from_secs(120)).await {
                        //reaction.as_inner_ref().delete(&ctx).await?;
                        let emoji = &reaction.as_inner_ref().emoji;

                        match emoji.as_data().as_str() {
                            "1\u{fe0f}\u{20e3}" => {
                                hooks.remove_item(&hook_url);
                                sqlx::query!("UPDATE new_posts SET webhook = $2 WHERE booru_url = 'yande.re' AND tags = $1",
                                    &sorted_tags, &hooks)
                                    .execute(pool)
                                    .await?;
                                return Ok(());
                            },
                            "2\u{fe0f}\u{20e3}" => {
                                return Ok(());
                            },
                            _ => (),
                        }
                    }
                    return Ok(());
                    
                } else {
                    hooks.push(hook_url);
                    hooks
                }
            } else {
                vec![hook_url]
            };
            hooks.dedup();

            sqlx::query!("UPDATE new_posts SET webhook = $2 WHERE booru_url = 'yande.re' AND tags = $1", &sorted_tags, &hooks)
                .execute(pool)
                .await?;
        } else {
            let hooks = vec![hook_url];

            let md5s = {
                let url = Url::parse_with_params("https://yande.re/post/index.json",
                    &[("tags", &sorted_tags), ("limit", &"100".to_string())])?;

                let resp = reqwest::get(url)
                    .await?
                    .json::<Vec<Post>>()
                    .await?;
                
                resp.iter().map(|post| post.md5.clone()).collect::<Vec<String>>()
            };

            sqlx::query!("INSERT INTO new_posts (booru_url, tags, webhook, sent_md5) VALUES ('yande.re', $1, $2, $3)", &sorted_tags, &hooks, &md5s)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}

/// Configure the notifications of the channel.
/// WIP
#[command]
async fn notifications(ctx: &mut Context, message: &Message, _args: Args) -> CommandResult {
    let mut msg = message.channel_id.send_message(&ctx, |m| {
        m.content(format!("<@{}>", message.author.id));
        m.embed(|e| {
            e.title("Select the number of option that you want");
            e.description("Choose what notification type you want:\n\n1: yande.re posts (WebHook)\n2: yande.re posts (Bot Message)  `WIP`")
        })
    }).await?;

    for i in 1..3_u8 {
        msg.react(&ctx, format!("{}\u{fe0f}\u{20e3}", i)).await?;
    }

    loop {
        if let Some(reaction) = message.author.await_reaction(&ctx).timeout(Duration::from_secs(120)).await {
            reaction.as_inner_ref().delete(&ctx).await?;
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "1\u{fe0f}\u{20e3}" => {yande_re_webhook(&ctx, &mut msg, &message.author).await?; break},
                "2\u{fe0f}\u{20e3}" => {yande_re_channel(&ctx, &mut msg, &message.author).await?; break},
                _ => (),
            }

        } else {
            msg.edit(&ctx, |m| {
                m.content(format!("<@{}>: Timeout", message.author.id))
            }).await?;
            ctx.http.edit_message(msg.channel_id.0, msg.id.0, &serde_json::json!({"flags" : 4})).await?;
            msg.delete_reactions(&ctx).await?;
            return Ok(());
        }
    }

    msg.edit(&ctx, |m| {
        m.content(format!("<@{}>: Success!", message.author.id))
    }).await?;

    //ctx.http.edit_message(msg.channel_id.0, msg.id.0, &serde_json::json!({"flags" : 4})).await?;
    msg.delete_reactions(&ctx).await?;

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
