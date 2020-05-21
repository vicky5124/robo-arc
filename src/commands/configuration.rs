use crate::{
    utils::booru,
    ConnectionPool,
    AnnoyedChannels,
    BooruCommands,
    notifications::Post,
    MASTER_GROUP,
};

use std::time::Duration;

use sqlx;
use futures::TryStreamExt;
use futures::stream::StreamExt;

use serde_json;
use reqwest::Url;

use regex::Regex;

use serenity::{
    prelude::Context,
    model::channel::{
        Message,
        ReactionType,
    },
    model::user::User,
    model::id::RoleId,
    framework::standard::{
        Args,
        Delimiter,
        CommandResult,
        CheckResult,
        macros::{
            command,
            check,
        },
    },
    utils::{
        content_safe,
        ContentSafeOptions,
    },
};

#[check]
#[name = "bot_has_manage_roles"]
async fn bot_has_manage_roles_check(ctx: &Context, msg: &Message) -> CheckResult {
    let bot_id = ctx.cache.current_user().await.id.0;
    if !ctx.http.get_member(msg.guild_id.unwrap().0, bot_id)
        .await
        .expect("What even")
        .permissions(ctx)
        .await
        .expect("What even 2")
        .manage_roles()
    {
        CheckResult::new_user("I'm unable to run this command due to missing the `Manage Roles` permission.")
    } else {
        CheckResult::Success
    }
}

async fn set_best_tags(sex: &str, ctx: &Context, msg: &Message, mut tags: String) -> Result<(), Box<dyn std::error::Error>> {
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

            msg.reply(ctx, format!("Successfully set your husbando to `{}`", &tags)).await?;
        } else if sex == "girl" {
            // insert +1girl
            tags += " 1girl";

            sqlx::query!("INSERT INTO best_bg (best_girl, user_id) VALUES ($1, $2)", &tags, user_id)
                .execute(pool)
                .await?;

            msg.reply(ctx, format!("Successfully set your waifu to `{}`", &tags)).await?;
        }
    } else if sex == "boy" {
        // update +1boy
        tags += " 1boy";

        sqlx::query!("UPDATE best_bg SET best_boy = $1 WHERE user_id = $2", &tags, user_id)
            .execute(pool)
            .await?;

        msg.reply(ctx, format!("You successfully broke up with your old husbando, now your husbando is `{}`", &tags)).await?;
    } else if sex == "girl" {
        // update +1girl
        tags += " 1girl";

        sqlx::query!("UPDATE best_bg SET best_girl = $1 WHERE user_id = $2", &tags, user_id)
            .execute(pool)
            .await?;

        msg.reply(ctx, format!("You successfully broke up with your old waifu, now your waifu is `{}`", &tags)).await?;
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
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let streamer = args.single_quoted::<String>()?;
    let mut channels = Vec::new();

    for channel in msg.guild_id.unwrap().channels(ctx).await?.keys() {
        channels.push(channel.0 as i64);
    }

    let role_ids = sqlx::query!("SELECT role_id FROM streamer_notification_channel WHERE streamer = $1 AND channel_id = ANY($2)", &streamer, &channels)
        .fetch_optional(pool)
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
        msg.channel_id.say(ctx, "The mentioned streamer does not have a role configured on this server.").await?;
    } else if role_id == 0 {
        msg.channel_id.say(ctx, "The mentioned streamer is not being notified on this server").await?;
    } else {
        let mut member = ctx.http.get_member(msg.guild_id.unwrap().0, msg.author.id.0).await?;
        if !member.roles.contains(&RoleId(role_id as u64)) {
            if let Err(_) = member.add_role(ctx, role_id as u64).await {
                msg.channel_id.say(ctx, "The configured role does not exist, contact the server administrators about the issue.").await?;
            } else {
                msg.channel_id.say(ctx, format!("Successfully obtained the role `{}`",
                    RoleId(role_id as u64).to_role_cached(ctx).await.unwrap().name))
                    .await?;
            }
        } else {
            if let Err(why) = member.remove_role(ctx, role_id as u64).await {
                msg.channel_id.say(ctx, format!("I was unable to remove your role: {}", why)).await?;
            } else {
                msg.channel_id.say(ctx, format!("Successfully removed the role `{}`",
                    RoleId(role_id as u64).to_role_cached(ctx).await.unwrap().name))
                    .await?;
            }
        }
    }

    Ok(())
}

#[command]
#[aliases(husbando, husband)]
async fn best_boy(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("boy", ctx, msg, args.message().to_string()).await?)
}

#[command]
#[aliases(waifu, wife)]
async fn best_girl(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("girl", ctx, msg, args.message().to_string()).await?)
}

#[command]
async fn booru(ctx: &Context, msg: &Message, raw_args: Args) -> CommandResult {
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
            msg.reply(ctx, "Please, specify the booru to set as your default.").await?;
            return Ok(());
        }
        sqlx::query!("INSERT INTO best_bg (booru, user_id) VALUES ($1, $2)", &booru, user_id)
            .execute(pool)
            .await?;

        msg.reply(ctx, format!("Successfully set your main booru to `{}`", &booru)).await?;
    } else {
        if booru.as_str() == "" {return Ok(());}

        sqlx::query!("UPDATE best_bg SET booru = $1 WHERE user_id = $2", &booru, user_id)
            .execute(pool)
            .await?;

        msg.reply(ctx, format!("Successfully edited your main booru to `{}`", &booru)).await?;
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
async fn channel(_ctx: &Context, _message: &Message, _args: Args) -> CommandResult {
    Ok(())
}

async fn twitch_channel(ctx: &Context, msg: &mut Message, author: &User) -> Result<(), Box<dyn std::error::Error>> {
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

    loop {
        if let Some(reply) = author.await_reply(ctx).timeout(Duration::from_secs(120)).await {
            let content = reply.content.split(' ').collect::<Vec<&str>>();

            if content.len() < 2 {
                msg.edit(ctx, |m| {
                    m.content(format!("<@{}>", author.id));
                    m.embed(|e| {
                        e.title("Not enough arguments provided.");
                        e.description("Examples:
                            `bobross yes @jop_notifications`
                            `raysworks no @technical_minecraft`
                            `the8bitdrummer no`")
                    })
                }).await?;
            } else {
                let allow_user = match content[1].to_lowercase().as_str() {
                    "yes" | "1" | "true" => Some(true),
                    "no" | "0" | "false" => Some(false),
                    _ => None,
                };

                if allow_user.is_none() {
                    msg.edit(ctx, |m| {
                        m.content(format!("<@{}>", author.id));
                        m.embed(|e| {
                            e.title("Invalid argumnet passed on the second possition.");
                            e.description("Examples:
                                `bobross yes @jop_notifications`
                                `raysworks no @technical_minecraft`
                                `the8bitdrummer no`")
                        })
                    }).await?;
                } else {
                    let streamer = content[0];
                    let use_default = allow_user.unwrap();
                    let mut role = None;

                    if let Some(role_id_raw) = content.get(2) {
                        let re = Regex::new("[<@&>]").unwrap();
                        let role_id = re.replace_all(&role_id_raw, "").into_owned();

                        if let Ok(x) = role_id.parse::<i64>() {
                            role = Some(x)
                        } else {
                            msg.reply(ctx, "You provided an invalid role, Defaulting to no role.").await?;
                        }
                    }

                    let data_read = ctx.data.read().await;
                    let pool = data_read.get::<ConnectionPool>().unwrap();

                    let streamer_data = sqlx::query!("SELECT streamer FROM streamers WHERE streamer = $1", streamer)
                        .fetch_optional(pool)
                        .await?;

                    if streamer_data.is_none() {
                        sqlx::query!("INSERT INTO streamers (streamer) VALUES ($1)", streamer)
                            .execute(pool)
                            .await?;
                    }

                    sqlx::query!("INSERT INTO streamer_notification_channel (streamer, role_id, use_default, channel_id) VALUES ($1, $2, $3, $4)", streamer, role, use_default, msg.channel_id.0 as i64)
                        .execute(pool)
                        .await?;

                    msg.edit(ctx, |m| {
                        m.content(format!("<@{}>", author.id));
                        m.embed(|e| {
                            e.title("Success!");
                            e.description(format!("Streamer: `{}`
                                Use Default Text: `{}`
                                Role: {:?}", streamer, use_default, role))
                        })
                    }).await?;
                    break;
                }
            }
        }
    }

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

    if let Some(reply) = author.await_reply(ctx).timeout(Duration::from_secs(120)).await {
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

    let hooks = msg.channel_id.webhooks(ctx).await?;
    let mut existing_hook = false;

    let bot_id = ctx.cache.current_user().await.id;

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

                    if let Some(reaction) = author.await_reaction(ctx).timeout(Duration::from_secs(120)).await {
                        //reaction.as_inner_ref().delete(ctx).await?;
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
/// WIP means that is basically doesn't work, so don't use those.
#[command]
async fn notifications(ctx: &Context, message: &Message, _args: Args) -> CommandResult {
    let mut msg = message.channel_id.send_message(ctx, |m| {
        m.content(format!("<@{}>", message.author.id));
        m.embed(|e| {
            e.title("Select the number of option that you want");
            e.description("Choose what notification type you want:\n\n1: yande.re posts (WebHook)\n2: yande.re posts (Bot Message)  `WIP`\n3: Twitch Notification (Bot Message)")
        })
    }).await?;

    for i in 1..4_u8 {
        let num = ReactionType::Unicode(String::from(format!("{}\u{fe0f}\u{20e3}", i)));
        msg.react(ctx, num).await?;
    }

    loop {
        if let Some(reaction) = message.author.await_reaction(ctx).timeout(Duration::from_secs(120)).await {
            reaction.as_inner_ref().delete(ctx).await?;
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "1\u{fe0f}\u{20e3}" => {yande_re_webhook(ctx, &mut msg, &message.author).await?; break},
                "2\u{fe0f}\u{20e3}" => {yande_re_channel(ctx, &mut msg, &message.author).await?; break},
                "3\u{fe0f}\u{20e3}" => {twitch_channel(ctx, &mut msg, &message.author).await?; break},
                _ => (),
            }

        } else {
            msg.edit(ctx, |m| {
                m.content(format!("<@{}>: Timeout", message.author.id))
            }).await?;
            ctx.http.edit_message(msg.channel_id.0, msg.id.0, &serde_json::json!({"flags" : 4})).await?;
            msg.delete_reactions(ctx).await?;
            return Ok(());
        }
    }

    msg.edit(ctx, |m| {
        m.content(format!("<@{}>: Success!", message.author.id))
    }).await?;

    //ctx.http.edit_message(msg.channel_id.0, msg.id.0, &serde_json::json!({"flags" : 4})).await?;
    msg.delete_reactions(ctx).await?;

    Ok(())
}

/// Toggles the annoying features on or off.
#[command]
async fn annoy(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
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

        msg.channel_id.say(ctx, format!("Successfully removed `{}` from the list of channels that allows the bot to do annoying features.", msg.channel_id.name(ctx).await.unwrap())).await?;

    } else {
        sqlx::query!("INSERT INTO annoyed_channels (channel_id) VALUES ($1)", channel_id)
            .execute(pool)
            .await?;

        msg.channel_id.say(ctx, format!("Successfully added `{}` to the list of channels that allows the bot to do annoying features.", msg.channel_id.name(ctx).await.unwrap())).await?;
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
/// `prefix`: Changes the bot prefix.
/// `disable_command`: Disables a command.
/// `enable_command`: Enables a disabled command.
#[command]
#[required_permissions(MANAGE_GUILD)]
#[only_in("guilds")]
#[aliases(server)]
#[sub_commands(prefix, disable_command, enable_command)]
async fn guild(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
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
    let data_read = ctx.data.read().await;
    let booru_commands = data_read.get::<BooruCommands>().unwrap();

    let mut command_name = args.single_quoted::<String>()?;
    if booru_commands.contains(&command_name) {
        command_name = "picture".to_string();
    }

    for group in MASTER_GROUP.options.sub_groups {
        for command in group.options.commands {
            if command.options.names.contains(&command_name.as_str()) {
                let pool = data_read.get::<ConnectionPool>().unwrap();

                let command_name = command.options.names[0];

                let disallowed_commands = sqlx::query!(
                    "SELECT disallowed_commands FROM prefixes WHERE guild_id = $1",
                    msg.guild_id.unwrap().0 as i64,
                ).fetch_optional(pool).await?;

                if let Some(x) = disallowed_commands {
                    if let Some(mut disallowed_commands) = x.disallowed_commands {
                        disallowed_commands.push(command_name.to_string());
                        sqlx::query!("UPDATE prefixes SET disallowed_commands = $1 WHERE guild_id = $2",
                            &disallowed_commands,
                            msg.guild_id.unwrap().0 as i64,
                        ).execute(pool).await?;
                    } else {
                        let disallowed_commands = vec![command_name.to_string()];
                        sqlx::query!("UPDATE prefixes SET disallowed_commands = $1 WHERE guild_id = $2",
                            &disallowed_commands,
                            msg.guild_id.unwrap().0 as i64,
                        ).execute(pool).await?;
                    }
                } else {
                    let disallowed_commands = vec![command_name.to_string()];
                    sqlx::query!("INSERT INTO prefixes (disallowed_commands, guild_id, prefix) VALUES ($1, $2, $3)",
                        &disallowed_commands,
                        msg.guild_id.unwrap().0 as i64,
                        ".".to_string(),
                    ).execute(pool).await?;
                }

                msg.reply(ctx, format!("Command `{}` successfully disabled.", command_name)).await?;
                return Ok(())
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
    let data_read = ctx.data.read().await;
    let booru_commands = data_read.get::<BooruCommands>().unwrap();

    let mut command_name = args.single_quoted::<String>()?;
    if booru_commands.contains(&command_name) {
        command_name = "picture".to_string();
    }

    for group in MASTER_GROUP.options.sub_groups {
        for command in group.options.commands {
            if command.options.names.contains(&command_name.as_str()) {
                let command_name = command.options.names[0];

                let pool = data_read.get::<ConnectionPool>().unwrap();

                let disallowed_commands = sqlx::query!(
                    "SELECT disallowed_commands FROM prefixes WHERE guild_id = $1",
                    msg.guild_id.unwrap().0 as i64,
                ).fetch_optional(pool).await?;

                if let Some(x) = disallowed_commands {
                    if let Some(mut disallowed_commands) = x.disallowed_commands {
                        if disallowed_commands.contains(&command_name.to_string()) {
                            disallowed_commands.remove_item(&command_name);
                            sqlx::query!("UPDATE prefixes SET disallowed_commands = $1 WHERE guild_id = $2",
                                &disallowed_commands,
                                msg.guild_id.unwrap().0 as i64,
                            ).execute(pool).await?;

                            msg.reply(ctx, format!("Command `{}` successfully enabled.", command_name)).await?;
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
