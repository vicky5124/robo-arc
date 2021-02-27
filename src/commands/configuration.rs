use crate::{
    global_data::*, notifications::Post, utils::booru, utils::checks::*,
    utils::logging::LoggingEvents, MASTER_GROUP,
};

use std::time::Duration;

use futures::stream::StreamExt;
use futures::TryStreamExt;

use reqwest::Url;
use serde_json;

use regex::Regex;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult, Delimiter},
    model::channel::Channel,
    model::channel::{Message, ReactionType},
    model::id::RoleId,
    model::webhook::Webhook,
    prelude::Context,
    utils::{content_safe, ContentSafeOptions},
};

async fn set_best_tags(
    sex: &str,
    ctx: &Context,
    msg: &Message,
    mut tags: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let user_id = msg.author.id.0 as i64;

    let data = sqlx::query!(
        "SELECT best_boy, best_girl FROM best_bg WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await?;

    if let None = data {
        if sex == "boy" {
            // insert +1boy
            tags += " 1boy";

            sqlx::query!(
                "INSERT INTO best_bg (best_boy, user_id) VALUES ($1, $2)",
                &tags,
                user_id
            )
            .execute(&pool)
            .await?;

            msg.reply(
                ctx,
                format!("Successfully set your husbando to `{}`", &tags),
            )
            .await?;
        } else if sex == "girl" {
            // insert +1girl
            tags += " 1girl";

            sqlx::query!(
                "INSERT INTO best_bg (best_girl, user_id) VALUES ($1, $2)",
                &tags,
                user_id
            )
            .execute(&pool)
            .await?;

            msg.reply(ctx, format!("Successfully set your waifu to `{}`", &tags))
                .await?;
        }
    } else if sex == "boy" {
        // update +1boy
        tags += " 1boy";

        sqlx::query!(
            "UPDATE best_bg SET best_boy = $1 WHERE user_id = $2",
            &tags,
            user_id
        )
        .execute(&pool)
        .await?;

        msg.reply(
            ctx,
            format!(
                "You successfully broke up with your old husbando, now your husbando is `{}`",
                &tags
            ),
        )
        .await?;
    } else if sex == "girl" {
        // update +1girl
        tags += " 1girl";

        sqlx::query!(
            "UPDATE best_bg SET best_girl = $1 WHERE user_id = $2",
            &tags,
            user_id
        )
        .execute(&pool)
        .await?;

        msg.reply(
            ctx,
            format!(
                "You successfully broke up with your old waifu, now your waifu is `{}`",
                &tags
            ),
        )
        .await?;
    }

    Ok(())
}

/// Configures aspects of the bot tied to your account.
///
/// Configurable aspects:
/// `best_girl`: Sets your best girl to the given tags.
/// `best_boy`: Sets your best boy to the given tags.
/// `booru`: Sets the booru to be used for the best_X commands ~~and `picture`~~
/// `streamrole`: Gives you the configured streamrole of a streamer the guild gets notifications on.
#[command]
#[aliases("self", "me")]
#[sub_commands(best_boy, best_girl, booru, streamrole)]
async fn user(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

/// Gives you the stream notification role bound to a streamer being notified on the server.
///
/// Usage: `config user streamrole bobross`
#[command]
#[only_in("guilds")]
#[min_args(1)]
#[checks("bot_has_manage_roles")]
async fn streamrole(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let streamer = args.single_quoted::<String>()?;
    let mut channels = Vec::new();

    for channel in msg.guild_id.unwrap().channels(ctx).await?.keys() {
        channels.push(channel.0 as i64);
    }

    let role_ids = sqlx::query!("SELECT role_id FROM streamer_notification_channel WHERE streamer = $1 AND channel_id = ANY($2)", &streamer, &channels)
        .fetch_optional(&pool)
        .boxed()
        .await?;

    let role_id = if let Some(roles) = role_ids {
        if let Some(r) = roles.role_id {
            r
        } else {
            1
        }
    } else {
        0
    };

    if role_id == 1 {
        msg.channel_id
            .say(
                ctx,
                "The mentioned streamer does not have a role configured on this server.",
            )
            .await?;
    } else if role_id == 0 {
        msg.channel_id
            .say(
                ctx,
                "The mentioned streamer is not being notified on this server",
            )
            .await?;
    } else {
        let mut member = ctx
            .http
            .get_member(msg.guild_id.unwrap().0, msg.author.id.0)
            .await?;
        if !member.roles.contains(&RoleId(role_id as u64)) {
            if let Err(_) = member.add_role(ctx, role_id as u64).await {
                msg.channel_id.say(ctx, "The configured role does not exist, contact the server administrators about the issue.").await?;
            } else {
                msg.channel_id
                    .say(
                        ctx,
                        format!(
                            "Successfully obtained the role `{}`",
                            RoleId(role_id as u64)
                                .to_role_cached(ctx)
                                .await
                                .unwrap()
                                .name
                        ),
                    )
                    .await?;
            }
        } else {
            if let Err(why) = member.remove_role(ctx, role_id as u64).await {
                msg.channel_id
                    .say(ctx, format!("I was unable to remove your role: {}", why))
                    .await?;
            } else {
                msg.channel_id
                    .say(
                        ctx,
                        format!(
                            "Successfully removed the role `{}`",
                            RoleId(role_id as u64)
                                .to_role_cached(ctx)
                                .await
                                .unwrap()
                                .name
                        ),
                    )
                    .await?;
            }
        }
    }

    Ok(())
}

#[command]
#[aliases(husbando, husband, bb)]
async fn best_boy(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("boy", ctx, msg, args.message().to_string()).await?)
}

#[command]
#[aliases(waifu, wife, bg)]
async fn best_girl(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("girl", ctx, msg, args.message().to_string()).await?)
}

#[command]
async fn booru(ctx: &Context, msg: &Message, raw_args: Args) -> CommandResult {
    let booru = raw_args.message().to_lowercase();

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let user_id = msg.author.id.0 as i64;

    let data = sqlx::query!(
        "SELECT best_boy, best_girl FROM best_bg WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .boxed()
    .await?;

    if let None = data {
        if booru.as_str() == "" {
            msg.reply(ctx, "Please, specify the booru to set as your default.")
                .await?;
            return Ok(());
        }
        sqlx::query!(
            "INSERT INTO best_bg (booru, user_id) VALUES ($1, $2)",
            &booru,
            user_id
        )
        .execute(&pool)
        .await?;

        msg.reply(
            ctx,
            format!("Successfully set your main booru to `{}`", &booru),
        )
        .await?;
    } else {
        if booru.as_str() == "" {
            return Ok(());
        }

        sqlx::query!(
            "UPDATE best_bg SET booru = $1 WHERE user_id = $2",
            &booru,
            user_id
        )
        .execute(&pool)
        .await?;

        msg.reply(
            ctx,
            format!("Successfully edited your main booru to `{}`", &booru),
        )
        .await?;
    }
    Ok(())
}

/// Configures the bot for the channel it was invoked on.
///
/// Configurable aspects:
/// `toggle_annoy`: Toggles the annoying features on or off.
/// `notifications`: Configure the notifications for YandeRe posts or Twitch livestreams.
#[command]
#[required_permissions(MANAGE_CHANNELS)]
#[only_in("guilds")]
#[sub_commands(toggle_annoy, notifications, logging)]
#[aliases(chan)]
async fn channel(_ctx: &Context, _message: &Message, _args: Args) -> CommandResult {
    Ok(())
}

/// Configure the notifications of the channel.
/// WIP means that is basically doesn't work, so don't use those.
#[command]
async fn notifications(ctx: &Context, message: &Message, _args: Args) -> CommandResult {
    let mut msg = message.channel_id.send_message(ctx, |m| {
        m.content(format!("<@{}>", message.author.id));
        m.embed(|e| {
            e.title("Select the number of option that you want");
            e.description("Choose what message type you want:\n\n1: WebHook\n2: Bot Message\n\n(Bot message will allow for users to delete the notification with 🚫 reactions. WebHook is recommended for non-NSFW messages.)")
        })
    }).await?;

    for i in 1..=2_u8 {
        let num = ReactionType::Unicode(String::from(format!("{}\u{fe0f}\u{20e3}", i)));
        msg.react(ctx, num).await?;
    }

    let mut is_hook = true;
    let mut is_create = true;
    let mut site = "yandere";

    loop {
        if let Some(reaction) = message
            .author
            .await_reaction(ctx)
            .timeout(Duration::from_secs(120))
            .await
        {
            reaction.as_inner_ref().delete(ctx).await?;
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "1\u{fe0f}\u{20e3}" => is_hook = true,
                "2\u{fe0f}\u{20e3}" => is_hook = false,
                _ => (),
            }

            msg.edit(ctx, |m| {
                m.content(format!("<@{}>", message.author.id));
                m.embed(|e| {
                    e.title("Select the number of option that you want");
                    e.description("Select the operation you want to do:\n\n1: Create Notification\n2: Remove Existing Notification (WIP)")
                })
            }).await?;

            if let Some(reaction) = message
                .author
                .await_reaction(ctx)
                .timeout(Duration::from_secs(120))
                .await
            {
                reaction.as_inner_ref().delete(ctx).await?;
                let emoji = &reaction.as_inner_ref().emoji;

                match emoji.as_data().as_str() {
                    "1\u{fe0f}\u{20e3}" => is_create = true,
                    "2\u{fe0f}\u{20e3}" => is_create = false,
                    _ => (),
                }
                msg.edit(ctx, |m| {
                    m.content(format!("<@{}>", message.author.id));
                    m.embed(|e| {
                        e.title("Select the number of option that you want");
                        e.description("Select the site to configure notifications on:\n\n1: YandeRe\n2: Twitch")
                    })
                }).await?;

                if let Some(reaction) = message
                    .author
                    .await_reaction(ctx)
                    .timeout(Duration::from_secs(120))
                    .await
                {
                    reaction.as_inner_ref().delete(ctx).await?;
                    let emoji = &reaction.as_inner_ref().emoji;

                    match emoji.as_data().as_str() {
                        "1\u{fe0f}\u{20e3}" => site = "yandere",
                        "2\u{fe0f}\u{20e3}" => site = "twitch",
                        _ => (),
                    }
                    break;
                } else {
                    timeout(ctx, &mut msg, message).await?;
                    return Ok(());
                }
            } else {
                timeout(ctx, &mut msg, message).await?;
                return Ok(());
            }
        } else {
            timeout(ctx, &mut msg, message).await?;
            return Ok(());
        }
    }

    match site {
        "yandere" => {
            configure_yandere(ctx, &mut msg, message, is_create, is_hook).await?;
        }
        "twitch" => {
            configure_twitch(ctx, &mut msg, message, is_create, is_hook).await?;
        }
        _ => {
            timeout(ctx, &mut msg, message).await?;
        }
    }

    msg.edit(ctx, |m| {
        m.content(format!("<@{}>: Done.", message.author.id))
    })
    .await?;

    ctx.http
        .edit_message(
            msg.channel_id.0,
            msg.id.0,
            &serde_json::json!({"flags" : 4}),
        )
        .await?;
    msg.delete_reactions(ctx).await?;

    Ok(())
}

trait Hook {
    fn swap_hook(&mut self, data: Webhook);
}

#[derive(Debug, Default)]
struct YandeRe {
    tags: String,
    hook: Option<Webhook>,
}

impl Hook for YandeRe {
    fn swap_hook(&mut self, data: Webhook) {
        self.hook = Some(data);
    }
}

#[derive(Debug, Default)]
struct Twitch {
    streamer: String,
    allow_user: Option<bool>,
    role: Option<String>,
    role_id: Option<i64>,
    hook: Option<Webhook>,
}

impl Hook for Twitch {
    fn swap_hook(&mut self, data: Webhook) {
        self.hook = Some(data);
    }
}

async fn check_hook(ctx: &Context, msg: &Message, embed: &mut impl Hook) {
    let hooks = msg
        .channel_id
        .webhooks(ctx)
        .await
        .expect("Error obtaining webhooks on channel.");

    let bot_id = ctx.cache.current_user().await.id;

    for (index, hook) in hooks.iter().enumerate() {
        if let Some(u) = &hook.user {
            if u.id == bot_id {
                embed.swap_hook(hooks[index].clone());
            }
        }
    }
}

async fn configure_yandere(
    ctx: &Context,
    msg: &mut Message,
    og_message: &Message,
    is_create: bool,
    is_hook: bool,
) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let mut config = YandeRe::default();
    let author = &og_message.author;

    if is_create {
        msg.edit(ctx, |m| {
            m.content(format!("<@{}>", author.id));
            m.embed(|e| {
                e.title("Say the tags you would like to get notified about");
                e.description("This supports the same flags as the `yandere` command.\nEx: `uruha_rushia -x yuri`")
            })
        }).await?;

        if let Some(reply) = author
            .await_reply(ctx)
            .timeout(Duration::from_secs(120))
            .await
        {
            let args = Args::new(&reply.content, &[Delimiter::Single(' ')]);

            let channel = ctx.http.get_channel(msg.channel_id.0).await?;

            let dm_channel = if let Some(channel) = msg.channel_id.to_channel_cached(ctx).await {
                channel.guild().is_none()
            } else {
                true
            };

            let mut tags = {
                if channel.is_nsfw() || dm_channel {
                    let mut raw_tags = booru::obtain_tags_unsafe(args).await;
                    booru::illegal_check_unsafe(&mut raw_tags).await
                } else {
                    let mut raw_tags = booru::obtain_tags_safe(args).await;
                    booru::illegal_check_safe(&mut raw_tags).await
                }
            };

            tags.sort();
            config.tags = tags.join(" ");

            msg.edit(ctx, |m| {
                m.content(format!("<@{}>", author.id));
                m.embed(|e| {
                    e.title("You selected the following tags");
                    e.description(format!("`{}`", config.tags))
                })
            })
            .await?;

            if is_hook {
                check_hook(ctx, msg, &mut config).await;
                if let None = config.hook {
                    let channel_id = msg.channel_id.0;
                    let map = serde_json::json!({"name": "Robo Arc"});

                    config.hook = Some(ctx.http.create_webhook(channel_id, &map).await?);
                }

                if let Some(webhook) = config.hook {
                    let hook_url = format!(
                        "https://discord.com/api/webhooks/{}/{}",
                        webhook.id, webhook.token
                    );

                    let query = sqlx::query!(
                        "SELECT webhook FROM new_posts WHERE booru_url = 'yande.re' AND tags = $1",
                        &config.tags
                    )
                    .fetch_optional(&pool)
                    .await?;

                    if let Some(row) = query {
                        let mut hooks = row.webhook.unwrap_or(Vec::new());
                        hooks.push(hook_url);
                        hooks.dedup();

                        sqlx::query!("UPDATE new_posts SET webhook = $2 WHERE booru_url = 'yande.re' AND tags = $1", &config.tags, &hooks)
                            .execute(&pool)
                            .await?;
                    } else {
                        let hooks = vec![hook_url];

                        let md5s = {
                            let url = Url::parse_with_params(
                                "https://yande.re/post/index.json",
                                &[("tags", &config.tags), ("limit", &"100".to_string())],
                            )?;

                            let resp = reqwest::get(url).await?.json::<Vec<Post>>().await?;

                            resp.iter()
                                .map(|post| post.md5.clone())
                                .collect::<Vec<String>>()
                        };

                        sqlx::query!("INSERT INTO new_posts (booru_url, tags, webhook, sent_md5) VALUES ('yande.re', $1, $2, $3)", &config.tags, &hooks, &md5s)
                            .execute(&pool)
                            .await?;
                    }
                } else {
                    og_message.reply(ctx, "There was an error obtaining a webhook. Make sure i have the permission to manage webhooks.").await?;
                    timeout(ctx, msg, og_message).await?;
                    return Ok(());
                }
            } else {
                let query = sqlx::query!(
                    "SELECT channel_id FROM new_posts WHERE booru_url = 'yande.re' AND tags = $1",
                    &config.tags
                )
                .fetch_optional(&pool)
                .await?;

                if let Some(row) = query {
                    let mut channels = row.channel_id.unwrap_or(Vec::new());
                    channels.push(msg.channel_id.0 as i64);
                    channels.dedup();

                    sqlx::query!("UPDATE new_posts SET channel_id = $2 WHERE booru_url = 'yande.re' AND tags = $1", &config.tags, &channels)
                        .execute(&pool)
                        .await?;
                } else {
                    let channels = vec![msg.channel_id.0 as i64];

                    let md5s = {
                        let url = Url::parse_with_params(
                            "https://yande.re/post/index.json",
                            &[("tags", &config.tags), ("limit", &"100".to_string())],
                        )?;

                        let resp = reqwest::get(url).await?.json::<Vec<Post>>().await?;

                        resp.iter()
                            .map(|post| post.md5.clone())
                            .collect::<Vec<String>>()
                    };

                    sqlx::query!("INSERT INTO new_posts (booru_url, tags, channel_id, sent_md5) VALUES ('yande.re', $1, $2, $3)", &config.tags, &channels, &md5s)
                        .execute(&pool)
                        .await?;
                }
            }
        } else {
            timeout(ctx, msg, og_message).await?;
            return Ok(());
        }
    }

    Ok(())
}

async fn configure_twitch(
    ctx: &Context,
    msg: &mut Message,
    og_message: &Message,
    is_create: bool,
    is_hook: bool,
) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let author = &og_message.author;
    let mut config = Twitch::default();

    if is_create {
        msg.edit(ctx, |m| {
            m.content(format!("<@{}>", author.id));
            m.embed(|e| {
                e.title("Say the name of the streamer, whether or not you let the discord user bound to the twitch account to change the notification message and an optional notification role.");
                e.description("Examples:
                    `bobross yes @jop_notifications`
                    `raysworks no @technical_minecraft`
                    `the8bitdrummer no`")
            })
        }).await?;

        if let Some(reply) = author
            .await_reply(ctx)
            .timeout(Duration::from_secs(120))
            .await
        {
            let content = reply.content.split(' ').collect::<Vec<&str>>();

            if content.len() < 2 {
                og_message.reply(ctx, "Not enough arguments.").await?;
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            }

            config.allow_user = match content[1].to_lowercase().as_str() {
                "yes" | "1" | "true" => Some(true),
                "no" | "0" | "false" => Some(false),
                _ => None,
            };

            if config.allow_user.is_none() {
                og_message
                    .reply(ctx, "Invalid argument passed on the second possition.")
                    .await?;
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            } else {
                config.streamer = content[0].to_string();
                let use_default = config.allow_user.unwrap();

                if let Some(role_id_raw) = content.get(2) {
                    let re = Regex::new("[<@&>]").unwrap();
                    let role_id = re.replace_all(&role_id_raw, "").into_owned();

                    if let Ok(r) = role_id.parse::<i64>() {
                        config.role_id = Some(r)
                    } else {
                        msg.reply(ctx, "You provided an invalid role, Defaulting to no role.")
                            .await?;
                    }
                }

                let streamer_data = sqlx::query!(
                    "SELECT streamer FROM streamers WHERE streamer = $1",
                    &config.streamer
                )
                .fetch_optional(&pool)
                .await?;

                if streamer_data.is_none() {
                    sqlx::query!(
                        "INSERT INTO streamers (streamer) VALUES ($1)",
                        &config.streamer
                    )
                    .execute(&pool)
                    .await?;
                }

                if is_hook {
                    check_hook(ctx, msg, &mut config).await;
                    if let None = config.hook {
                        let channel_id = msg.channel_id.0;
                        let map = serde_json::json!({"name": "Robo Arc"});

                        config.hook = Some(ctx.http.create_webhook(channel_id, &map).await?);
                    }
                    if let Some(webhook) = &config.hook {
                        let hook_url = format!(
                            "https://discord.com/api/webhooks/{}/{}",
                            webhook.id, webhook.token
                        );

                        sqlx::query!("INSERT INTO streamer_notification_webhook (streamer, role_id, use_default, webhook) VALUES ($1, $2, $3, $4)", &config.streamer, config.role_id, use_default, hook_url)
                            .execute(&pool)
                            .await?;
                    } else {
                        og_message.reply(ctx, "There was an error obtaining a webhook. Make sure i have the permission to manage webhooks.").await?;
                        timeout(ctx, msg, og_message).await?;
                        return Ok(());
                    }
                } else {
                    sqlx::query!("INSERT INTO streamer_notification_channel (streamer, role_id, use_default, channel_id) VALUES ($1, $2, $3, $4)", &config.streamer, config.role_id, use_default, msg.channel_id.0 as i64)
                        .execute(&pool)
                        .await?;
                }

                msg.channel_id
                    .send_message(ctx, |m| {
                        m.content(format!("<@{}>", author.id));
                        m.embed(|e| {
                            e.title("Success!");
                            e.description(format!(
                                "Streamer: `{}`
                            Use Default Text: `{}`
                            Role: <@&{}>",
                                &config.streamer,
                                use_default,
                                config.role_id.unwrap_or(0)
                            ))
                        })
                    })
                    .await?;
            }
        }
    } else {
        if is_hook {
            msg.edit(ctx, |m| {
                m.content(format!("<@{}>", author.id));
                m.embed(|e| {
                    e.title("Say the webhook url you would like to see the streamers to unnotify.");
                    e.description("Tip: you can locate the url inside the channel configuration, on the webhooks tab, it will be part of the hook created by me.\n You will likely need to replace `discordapp.com` to `discord.com` so it's recognized correctly.")
                })
            }).await?;

            let hook_url;
            if let Some(reply) = author
                .await_reply(ctx)
                .timeout(Duration::from_secs(120))
                .await
            {
                let _ = reply.delete(ctx).await;

                let mut m = reply.content.to_string();
                if m.starts_with("http") && m.contains("discord") && m.contains("webhook") {
                    m = m.replace("canary.", "");
                    m = m.replace("pbt.", "");
                    m = m.replace("discordapp", "discord");
                    hook_url = m.to_string();
                } else {
                    og_message
                        .reply(ctx, "An invalid url was provided.")
                        .await?;
                    timeout(ctx, msg, og_message).await?;
                    return Ok(());
                }
            } else {
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            }

            let mut query = sqlx::query!(
                "SELECT streamer FROM streamer_notification_webhook WHERE webhook = $1",
                &hook_url
            )
            .fetch(&pool);

            let mut streamers = Vec::new();
            while let Some(i) = query.try_next().await? {
                streamers.push(i.streamer);
            }

            let mut x = 0_usize;
            let streamers_choice = streamers
                .iter()
                .map(|i| {
                    x += 1;
                    format!("{}: '{}'\n", x, i)
                })
                .collect::<String>();

            if streamers_choice.is_empty() {
                og_message
                    .reply(ctx, "No streamers are being notified with that webhook.")
                    .await?;
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            }

            msg.edit(ctx, |m| {
                m.content(format!("<@{}>", author.id));
                m.embed(|e| {
                    e.title("Say the number of the streamer you would like to stop getting notified about.");
                    e.description(&streamers_choice)
                })
            }).await?;

            let index;
            if let Some(reply) = author
                .await_reply(ctx)
                .timeout(Duration::from_secs(120))
                .await
            {
                if let Ok(x) = reply.content.parse::<usize>() {
                    index = x;
                } else {
                    og_message
                        .reply(ctx, "An invalid number was provided.")
                        .await?;
                    timeout(ctx, msg, og_message).await?;
                    return Ok(());
                }
            } else {
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            }

            config.streamer = if let Some(x) = streamers.get(index - 1) {
                x.to_string()
            } else {
                og_message
                    .reply(ctx, "The number provided is too large.")
                    .await?;
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            };

            sqlx::query!(
                "DELETE FROM streamer_notification_webhook WHERE streamer = $1 AND webhook = $2",
                &config.streamer,
                &hook_url
            )
            .execute(&pool)
            .await?;
        } else {
            let mut query = sqlx::query!(
                "SELECT streamer FROM streamer_notification_channel WHERE channel_id = $1",
                msg.channel_id.0 as i64
            )
            .fetch(&pool);

            let mut streamers = Vec::new();
            while let Some(i) = query.try_next().await? {
                streamers.push(i.streamer);
            }

            let mut x = 0_usize;
            let streamers_choice = streamers
                .iter()
                .map(|i| {
                    x += 1;
                    format!("{}: '{}'\n", x, i)
                })
                .collect::<String>();

            if streamers_choice.is_empty() {
                og_message
                    .reply(ctx, "No streamers are being notified in this channel.")
                    .await?;
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            }

            msg.edit(ctx, |m| {
                m.content(format!("<@{}>", author.id));
                m.embed(|e| {
                    e.title("Say the number of the streamer you would like to stop getting notified about.");
                    e.description(&streamers_choice)
                })
            }).await?;

            let index;
            if let Some(reply) = author
                .await_reply(ctx)
                .timeout(Duration::from_secs(120))
                .await
            {
                if let Ok(x) = reply.content.parse::<usize>() {
                    index = x;
                } else {
                    og_message
                        .reply(ctx, "An invalid number was provided.")
                        .await?;
                    timeout(ctx, msg, og_message).await?;
                    return Ok(());
                }
            } else {
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            }

            config.streamer = if let Some(x) = streamers.get(index - 1) {
                x.to_string()
            } else {
                og_message
                    .reply(ctx, "The number provided is too large.")
                    .await?;
                timeout(ctx, msg, og_message).await?;
                return Ok(());
            };

            sqlx::query!(
                "DELETE FROM streamer_notification_channel WHERE streamer = $1 AND channel_id = $2",
                &config.streamer,
                msg.channel_id.0 as i64
            )
            .execute(&pool)
            .await?;
        }
    }

    Ok(())
}

async fn timeout(ctx: &Context, msg: &mut Message, og_message: &Message) -> CommandResult {
    msg.edit(ctx, |m| {
        m.content(format!(
            "<@{}>: Timeout (Command Terminated)",
            og_message.author.id
        ))
    })
    .await?;
    ctx.http
        .edit_message(
            msg.channel_id.0,
            msg.id.0,
            &serde_json::json!({"flags" : 4}),
        )
        .await?;
    msg.delete_reactions(ctx).await?;
    return Ok(());
}

/// Toggles the annoying features on or off.
#[command]
#[aliases(annoy)]
async fn toggle_annoy(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let channel_id = msg.channel_id.0 as i64;

    let data = sqlx::query!(
        "SELECT channel_id FROM annoyed_channels WHERE channel_id = $1",
        channel_id
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(_) = data {
        sqlx::query!(
            "DELETE FROM annoyed_channels WHERE channel_id IN ($1)",
            channel_id
        )
        .execute(&pool)
        .await?;

        msg.channel_id.say(ctx, format!("Successfully removed `{}` from the list of channels that allows the bot to do annoying features.", msg.channel_id.name(ctx).await.unwrap())).await?;
    } else {
        sqlx::query!(
            "INSERT INTO annoyed_channels (channel_id) VALUES ($1)",
            channel_id
        )
        .execute(&pool)
        .await?;

        msg.channel_id.say(ctx, format!("Successfully added `{}` to the list of channels that allows the bot to do annoying features.", msg.channel_id.name(ctx).await.unwrap())).await?;
    }

    {
        let mut raw_annoyed_channels = sqlx::query!("SELECT channel_id from annoyed_channels")
            .fetch(&pool)
            .boxed();

        let channels_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<AnnoyedChannels>().unwrap().clone()
        };

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
/// `prefix`: Changes the bot prefix.
/// `mute_role`: Sets the mute role of the server.
/// `disable_command`: Disables a command.
/// `enable_command`: Enables a disabled command.
/// `toggle_anti_spam`: Enables or Disables antispam.
#[command]
#[required_permissions(MANAGE_GUILD)]
#[only_in("guilds")]
#[aliases(server)]
#[sub_commands(prefix, mute_role, disable_command, enable_command, toggle_anti_spam)]
async fn guild(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[min_args(1)]
#[aliases(muterole, mute, mrole, mutrole, mutrol, muted_role, muted)]
#[checks("bot_has_manage_roles")]
#[required_permissions(MANAGE_ROLES)]
async fn mute_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let role = if let Ok(x) = args.single::<RoleId>() {
        x
    } else {
        msg.reply(
            ctx,
            "An invalid role was provided, please mention the role or post it's id.",
        )
        .await?;
        return Ok(());
    };

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    dbg!(&role);

    sqlx::query!("INSERT INTO muted_roles (guild_id, role_id) VALUES ($1, $2) ON CONFLICT (guild_id) DO UPDATE SET role_id = $2",
                  msg.guild_id.unwrap().0 as i64,
                  role.0 as i64)
        .execute(&pool)
        .await?;

    msg.react(ctx, '👍').await?;

    Ok(())
}

#[command]
#[min_args(1)]
async fn prefix(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(ctx, "Invalid prefix was given").await?;
        return Ok(());
    }
    let prefix = args.message();

    // change prefix on db
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let guild_id = msg.guild_id.unwrap().0 as i64;

    let data = sqlx::query!("SELECT prefix FROM prefixes WHERE guild_id = $1", guild_id)
        .fetch_optional(&pool)
        .boxed()
        .await?;

    if let None = data {
        sqlx::query!(
            "INSERT INTO prefixes (guild_id, prefix) VALUES ($1, $2)",
            guild_id,
            &prefix
        )
        .execute(&pool)
        .await?;
    } else {
        sqlx::query!(
            "UPDATE prefixes SET prefix = $2 WHERE guild_id = $1",
            guild_id,
            &prefix
        )
        .execute(&pool)
        .await?;
    }

    let content_safe_options = ContentSafeOptions::default();
    let bad_success_message = format!("Successfully changed your prefix to `{}`", prefix);
    let success_message = content_safe(ctx, bad_success_message, &content_safe_options).await;
    msg.reply(ctx, success_message).await?;
    Ok(())
}

/// Disables a command on this guild.
/// Note: Disablig any booru command will disable all the booru commands but Sankaku Chan and Idol.
///
/// Usage: `config guild disable_command urban`
#[command]
#[min_args(1)]
async fn disable_command(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let booru_commands = {
        let data_read = ctx.data.read().await;
        data_read.get::<BooruCommands>().unwrap().clone()
    };

    let mut command_name = args.single_quoted::<String>()?;
    if booru_commands.contains(&command_name) {
        command_name = "picture".to_string();
    }

    for group in MASTER_GROUP.options.sub_groups {
        for command in group.options.commands {
            if command.options.names.contains(&command_name.as_str()) {
                let pool = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<DatabasePool>().unwrap().clone()
                };

                let command_name = command.options.names[0];

                let disallowed_commands = sqlx::query!(
                    "SELECT disallowed_commands FROM prefixes WHERE guild_id = $1",
                    msg.guild_id.unwrap().0 as i64,
                )
                .fetch_optional(&pool)
                .await?;

                if let Some(x) = disallowed_commands {
                    if let Some(mut disallowed_commands) = x.disallowed_commands {
                        disallowed_commands.push(command_name.to_string());
                        sqlx::query!(
                            "UPDATE prefixes SET disallowed_commands = $1 WHERE guild_id = $2",
                            &disallowed_commands,
                            msg.guild_id.unwrap().0 as i64,
                        )
                        .execute(&pool)
                        .await?;
                    } else {
                        let disallowed_commands = vec![command_name.to_string()];
                        sqlx::query!(
                            "UPDATE prefixes SET disallowed_commands = $1 WHERE guild_id = $2",
                            &disallowed_commands,
                            msg.guild_id.unwrap().0 as i64,
                        )
                        .execute(&pool)
                        .await?;
                    }
                } else {
                    let disallowed_commands = vec![command_name.to_string()];
                    sqlx::query!("INSERT INTO prefixes (disallowed_commands, guild_id, prefix) VALUES ($1, $2, $3)",
                        &disallowed_commands,
                        msg.guild_id.unwrap().0 as i64,
                        ".".to_string(),
                    ).execute(&pool).await?;
                }

                msg.reply(
                    ctx,
                    format!("Command `{}` successfully disabled.", command_name),
                )
                .await?;
                return Ok(());
            }
        }
    }
    msg.reply(ctx, "Command not found.").await?;

    Ok(())
}

/// Enables a disabled command on this guild.
///
/// Usage: `config guild enable_command urban`
#[command]
async fn enable_command(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let booru_commands = {
        let data_read = ctx.data.read().await;
        data_read.get::<BooruCommands>().unwrap().clone()
    };

    let mut command_name = args.single_quoted::<String>()?;
    if booru_commands.contains(&command_name) {
        command_name = "picture".to_string();
    }

    for group in MASTER_GROUP.options.sub_groups {
        for command in group.options.commands {
            if command.options.names.contains(&command_name.as_str()) {
                let command_name = command.options.names[0];

                let pool = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<DatabasePool>().unwrap().clone()
                };

                let disallowed_commands = sqlx::query!(
                    "SELECT disallowed_commands FROM prefixes WHERE guild_id = $1",
                    msg.guild_id.unwrap().0 as i64,
                )
                .fetch_optional(&pool)
                .await?;

                if let Some(x) = disallowed_commands {
                    if let Some(mut disallowed_commands) = x.disallowed_commands {
                        if disallowed_commands.contains(&command_name.to_string()) {
                            if let Some(pos) =
                                disallowed_commands.iter().position(|x| &x == &command_name)
                            {
                                disallowed_commands.remove(pos);
                            };

                            sqlx::query!(
                                "UPDATE prefixes SET disallowed_commands = $1 WHERE guild_id = $2",
                                &disallowed_commands,
                                msg.guild_id.unwrap().0 as i64,
                            )
                            .execute(&pool)
                            .await?;

                            msg.reply(
                                ctx,
                                format!("Command `{}` successfully enabled.", command_name),
                            )
                            .await?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
    msg.reply(ctx, "Command not disabled.").await?;

    Ok(())
}

/// Toggles the Anti-Spam system on or off.
///
/// Currently it's a very simple "if more than 5 messages where sent in less than 5 second
/// intervals between them, they get deleted"
///
/// In the future, this will be able to be configured in multiple ways.
#[command]
#[aliases(toggleantispam, antispam, anti_spam, "toggle-anti-spam", "anti-spam")]
async fn toggle_anti_spam(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let data = sqlx::query!(
        "SELECT enabled FROM anti_spam WHERE guild_id = $1",
        msg.guild_id.unwrap().0 as i64
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(row) = data {
        sqlx::query!(
            "UPDATE anti_spam SET enabled = $2 WHERE guild_id = $1",
            msg.guild_id.unwrap().0 as i64,
            !row.enabled
        )
        .execute(&pool)
        .await?;
    } else {
        sqlx::query!(
            "INSERT INTO anti_spam (guild_id, enabled) VALUES ($1, true)",
            msg.guild_id.unwrap().0 as i64
        )
        .execute(&pool)
        .await?;
    }

    msg.react(ctx, '✅').await?;

    Ok(())
}

/// WIP: Configures logging for the channel.
///
/// Usage: `configure channel logging 134217727`
#[command]
#[aliases("logs")]
#[min_args(1)]
async fn logging(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let digits = args.single::<u64>()?;

    let channel = msg.channel(ctx).await.unwrap();

    if let Channel::Guild(channel) = channel {
        let hook = match channel
            .create_webhook_with_avatar(
                ctx,
                "Robo Arc - Logging",
                ctx.cache.current_user().await.face().as_str(),
            )
            .await
        {
            Err(why) => {
                msg.reply(ctx, format!("Could not create a webhook, please provide the bot access to manage webhooks in this channel.\n{}", why)).await?;
                return Ok(());
            }
            Ok(x) => x,
        };

        let pool = {
            let data_read = ctx.data.read().await;
            data_read.get::<DatabasePool>().unwrap().clone()
        };

        sqlx::query!(
            "INSERT INTO logging_channels (guild_id, webhook_url, bitwise) VALUES ($1, $2, $3)",
            msg.guild_id.unwrap().0 as i64,
            hook.url(),
            digits as i64
        )
        .execute(&pool)
        .await?;

        let events = LoggingEvents::from_bits_truncate(digits);
        msg.reply(
            ctx,
            format!("Successfully added logging for this events:\n{:?}", events),
        )
        .await?;
    } else {
        msg.reply(ctx, "Invalid Channel Type").await?;
    }

    Ok(())
}
