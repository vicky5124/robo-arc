use crate::ConnectionPool;
use crate::utils::logging::LoggingEvents;

//use tracing::{
//    warn,
//    error,
//};

use serenity::{
    model::{
        event::*,
        id::{
            ChannelId,
            UserId,
        },
        channel::{
            PermissionOverwriteType,
            ReactionType,
            Channel,
        },
    },
    prelude::Context,
};

pub async fn send_message_update(ctx: &Context, data: &MessageUpdateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();
    let old_message = sqlx::query!("SELECT content_history FROM log_messages WHERE id = $1", data.id.0 as i64)
        .fetch_optional(pool)
        .await;
    if let Ok(old_message) = old_message {
        if let Some(old_message) = old_message {
            if let Some(old_message) = old_message.content_history {
                let old_message_content = old_message.get(old_message.len().checked_sub(1).unwrap_or(0));
                if old_message_content.clone().unwrap_or(&String::new()) != &data.content.clone().unwrap_or_default() {
                    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.unwrap().0 as i64)
                        .fetch_optional(pool)
                        .await;

                    if let Ok(query) = query {
                        if let Some(query) = query {
                            let log_events = LoggingEvents::from_bits_truncate(query.bitwise as u64);
                            if log_events.contains(LoggingEvents::MessageUpdate) {
                                let _ = ChannelId(query.channel_id as u64).send_message(&ctx, |m| m.embed(|e| {
                                    e.title("Message Updated");
                                    e.description(format!("[Jump](https://discord.com/channels/{}/{}/{})", data.guild_id.unwrap().0, data.channel_id.0, data.id.0));

                                    let content = old_message_content.unwrap_or(&String::new()).to_owned() + "\u{200b}";
                                    if content.len() > 1000 {
                                        e.field("Original Content (1)", &content[..content.len()/2], false);
                                        e.field("Original Content (2)", &content[content.len()/2..], false);
                                    } else {
                                        e.field("Original Content", &content, false);
                                    }

                                    let content = data.content.as_ref().unwrap_or(&"- Empty Message".to_string()).to_string();
                                    if content.len() > 1000 {
                                        e.field("New Content (1)", &content[..content.len()/2], false);
                                        e.field("New Content (2)", &content[content.len()/2..], false);
                                    } else {
                                        e.field("New Content", &content, false);
                                    }
                                    if let Some(author) = &data.author {
                                        e.author(|a| {
                                            a.icon_url(author.face());
                                            a.name(author.tag())
                                        });
                                        //e.footer(|f| {
                                        //    f.icon_url(author.face());
                                        //    f.text(author.tag())
                                        //});
                                    }

                                    e
                                })).await;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub async fn send_message_delete(ctx: &Context, data: &MessageDeleteEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();
    let result = sqlx::query!("SELECT content, author_id, attachments, pinned, edited_timestamp, tts, webhook_id FROM log_messages WHERE id = $1", data.message_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(message) = result {
        let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.unwrap().0 as i64)
            .fetch_optional(pool)
            .await;

        if let Ok(query) = query {
            if let Some(query) = query {
                if let Some(msg) = message {
                    let author = if let Ok(x) = UserId(msg.author_id as u64).to_user(ctx).await { x } else { return };

                    let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                        e.title("Message Deleted");
                        e.description(format!("Message from <#{}> with the id `{}`", data.channel_id.0, data.message_id.0));
                        e.author(|a| {
                            a.icon_url(author.face());
                            a.name(author.tag())
                        });

                        if let Some(content) = &msg.content {
                            e.field("Content", content, false);
                        }

                        if let Some(id) = &msg.webhook_id {
                            e.field("Webhook ID", id, false);
                        }

                        if let Some(attachments) = &msg.attachments {
                            for attachment in attachments {
                                e.field("Attachment", attachment, false);
                            }
                        }

                        if let Some(x) = &msg.edited_timestamp {
                            e.timestamp(x);
                        }

                        e.footer(|f| {
                            f.text({
                                if msg.pinned && msg.tts.unwrap() {
                                    "Message was pinned and it was sent with TTS".to_string()
                                } else if msg.pinned && !msg.tts.unwrap() {
                                    "Message was pinned".to_string()
                                } else if !msg.pinned && msg.tts.unwrap() {
                                    "Message was sent with TTS".to_string()
                                } else {
                                    "Normal Message".to_string()
                                }
                            })
                        });

                        e
                    })).await;
                } else {
                    let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                        e.title("Message Deleted");
                        e.description(format!("No information about the message was available.\nMessage from <#{}> with the id `{}`", data.channel_id.0, data.message_id.0))
                    })).await;
                }
            }
        }
    }
}

pub async fn send_guild_member_add(ctx: &Context, data: &GuildMemberAddEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Member Joined");
                e.author(|a| {
                    a.icon_url(data.member.user.face());
                    a.name(data.member.user.tag())
                });
                e.field("Created at", &data.member.user.created_at().to_rfc2822(), false);
                if let Some(x) = &data.member.joined_at {
                    e.field("Joined at", x.to_rfc2822(), false);
                }
                e.field("ID", &data.member.user.id.0, false);

                e
            })).await;
        }
    }
}

pub async fn send_guild_member_remove(ctx: &Context, data: &GuildMemberRemoveEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Member Left");
                e.author(|a| {
                    a.icon_url(data.user.face());
                    a.name(data.user.tag())
                });
                e.field("Created at", &data.user.created_at().to_rfc2822(), false);
                e.field("ID", &data.user.id.0, false);

                e
            })).await;
        }
    }
}

pub async fn send_message_delete_bulk(ctx: &Context, data: &MessageDeleteBulkEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.unwrap().0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Bulk of messages Deleted");
                e.description(format!("Deleted on <#{}>", &data.channel_id.0));
                e.field("Number of messages:", &data.ids.len(), false);

                e
            })).await;
        }
    }
}

pub async fn send_guild_role_create(ctx: &Context, data: &GuildRoleCreateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Role Created");
                e.field("ID", &data.role.id.0, false);
                e.field("Mention", format!("<@&{}>", &data.role.id.0), false);
                e.field("Mentionable", &data.role.mentionable, false);
                e.field("Permissions", format!("{:?}", &data.role.permissions), false);
                e.colour(data.role.colour);
                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Created")
                });

                e
            })).await;
        }
    }
}

pub async fn send_guild_role_delete(ctx: &Context, data: &GuildRoleDeleteEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Role Deleted");
                e.field("ID", &data.role_id.0, false);
                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Deleted")
                });

                e
            })).await;
        }
    }
}

// discord sends update of every single role, even if a role has not changed...
pub async fn send_guild_role_update(_ctx: &Context, _data: &GuildRoleUpdateEvent) {
    //let rdata = ctx.data.read().await;
    //let pool = rdata.get::<ConnectionPool>().unwrap();

    //let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
    //    .fetch_optional(pool)
    //    .await;

    //if let Ok(query) = query {
    //    if let Some(query) = query {
    //        let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
    //            e.title("Role Updated");
    //            e.field("ID", &data.role.id.0, false);
    //            e.field("Mention", format!("<@&{}>", &data.role.id.0), false);
    //            e.field("Mentionable", &data.role.mentionable, false);
    //            e.field("Permissions", format!("{:?}", &data.role.permissions), false);
    //            e.colour(data.role.colour);
    //            e.timestamp(&chrono::offset::Utc::now());
    //            e.footer(|f| {
    //                f.text("Created")
    //            });

    //            e
    //        })).await;
    //    }
    //}
}

pub async fn send_guild_member_update(ctx: &Context, data: &GuildMemberUpdateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Member Updated");
                e.author(|a| {
                    a.icon_url(data.user.face());
                    a.name(data.user.tag())
                });
                e.description(format!("The user <@{}> has been updated", &data.user.id.0));
                e.field("User ID", &data.user.id.0, false);
                if let Some(nick) = &data.nick {
                    e.field("Nickname", nick, false);
                }
                e.field("Roles", &data.roles.iter().map(|i| format!("<@&{}>", i.0)).collect::<Vec<_>>().join(" | "), false);
                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Updated")
                });

                e
            })).await;
        }
    }
}

pub async fn send_reaction_add(ctx: &Context, data: &ReactionAddEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.reaction.guild_id.unwrap().0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let reaction = &data.reaction;
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Reaction Added");
                e.description(format!("Reaction on <#{0}> in [this message](https://discord.com/channels/{1}/{0}/{2})", reaction.channel_id.0, reaction.guild_id.unwrap().0, reaction.message_id.0));
                e.field("User ID", reaction.user_id.unwrap().0, false);

                let mut id = 0;

                e.field("Emoji", {
                    match &reaction.emoji {
                        ReactionType::Custom{
                            animated: x,
                            id: y,
                            name: z
                        } => {
                            id = y.0;

                            if *x {
                                format!("<a:{}:{}>", z.as_ref().unwrap(), y.0)
                            } else {
                                format!("<:{}:{}>", z.as_ref().unwrap(), y.0)
                            }
                        },
                        ReactionType::Unicode(x) => x.to_string(),
                        _ => String::new(),
                    }
                }, false);

                if id != 0 {
                    e.image(format!("https://cdn.discordapp.com/emojis/{}.png", id));
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Added")
                });

                e
            })).await;
        }
    }
}

pub async fn send_reaction_remove(ctx: &Context, data: &ReactionRemoveEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.reaction.guild_id.unwrap().0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let reaction = &data.reaction;
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Reaction Removed");
                e.description(format!("Reaction on <#{0}> in [this message](https://discord.com/channels/{1}/{0}/{2})", reaction.channel_id.0, reaction.guild_id.unwrap().0, reaction.message_id.0));
                e.field("User ID", reaction.user_id.unwrap().0, false);

                let mut id = 0;

                e.field("Emoji", {
                    match &reaction.emoji {
                        ReactionType::Custom{
                            animated: x,
                            id: y,
                            name: z
                        } => {
                            id = y.0;

                            if *x {
                                format!("<a:{}:{}>", z.as_ref().unwrap(), y.0)
                            } else {
                                format!("<:{}:{}>", z.as_ref().unwrap(), y.0)
                            }
                        },
                        ReactionType::Unicode(x) => x.to_string(),
                        _ => String::new(),
                    }
                }, false);

                if id != 0 {
                    e.image(format!("https://cdn.discordapp.com/emojis/{}.png", id));
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Removed")
                });

                e
            })).await;
        }
    }
}

pub async fn send_reaction_remove_all(ctx: &Context, data: &ReactionRemoveAllEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.unwrap().0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Reactions Cleared");
                e.description(format!("Reactions on <#{0}> in [this message](https://discord.com/channels/{1}/{0}/{2})", data.channel_id.0, data.guild_id.unwrap().0, data.message_id.0));

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Cleared")
                });

                e
            })).await;
        }
    }
}

pub async fn send_channel_create(ctx: &Context, data: &ChannelCreateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let mut channel_id = 0;

    match &data.channel {
        Channel::Guild(channel) => {
            let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", channel.guild_id.0 as i64)
                .fetch_optional(pool)
                .await;
            if let Ok(query) = query {
                if let Some(query) = query {
                    channel_id = query.channel_id;
                }
            }
        }
        Channel::Category(category) => {
            let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", category.guild_id.0 as i64)
                .fetch_optional(pool)
                .await;
            if let Ok(query) = query {
                if let Some(query) = query {
                    channel_id = query.channel_id;
                }
            }
        }
         _ => (),
    }

    if channel_id != 0 {
        match &data.channel {
            Channel::Guild(channel) => {
                let category_name = if let Some(category) = channel.category_id {
                    category.name(ctx).await
                } else {
                    None
                };

                let mut fields = Vec::new();

                for perm in &channel.permission_overwrites {
                    match perm.kind {
                        PermissionOverwriteType::Member(x) => {
                            let user = x.to_user(ctx).await.unwrap_or_default();

                            fields.push((format!("Allowed Permissions for \"{}\"", user.tag()), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for \"{}\"", user.tag()), format!("{:?}", perm.deny), false));
                        }
                        PermissionOverwriteType::Role(x) => {
                            let role = x.to_role_cached(ctx).await.unwrap();

                            fields.push((format!("Allowed Permissions for role \"{}\"", role.name), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for role \"{}\"", role.name), format!("{:?}", perm.deny), false));
                        }
                        _ => (),
                    }
                }

                let _ = ChannelId(channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                    e.title("Channel Created");
                    if let Some(category) = category_name {
                        e.description(format!("Channel <#{}> (`{}`) has been created on the category {}.", channel.id.0, channel.name, category));
                    } else {
                        e.description(format!("Channel <#{}> (`{}`) has been created outside a category.", channel.id.0, channel.name));
                    }
                    e.field("ID", channel.id.0, false);
                    e.field("Type", format!("{:?}", channel.kind), false);
                    e.fields(fields);

                    if channel.nsfw {
                        e.field("NSFW?", "Yes", false);
                    }

                    e.timestamp(&chrono::offset::Utc::now());
                    e.footer(|f| {
                        f.text("Created")
                    });

                    e
                })).await;
            }
            Channel::Category(category) => {
                let mut fields = Vec::new();

                for perm in &category.permission_overwrites {
                    match perm.kind {
                        PermissionOverwriteType::Member(x) => {
                            let user = x.to_user(ctx).await.unwrap_or_default();

                            fields.push((format!("Allowed Permissions for \"{}\"", user.tag()), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for \"{}\"", user.tag()), format!("{:?}", perm.deny), false));
                        }
                        PermissionOverwriteType::Role(x) => {
                            let role = x.to_role_cached(ctx).await.unwrap();

                            fields.push((format!("Allowed Permissions for role \"{}\"", role.name), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for role \"{}\"", role.name), format!("{:?}", perm.deny), false));
                        }
                        _ => (),
                    }
                }

                let _ = ChannelId(channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                    e.title("Category Created");
                    e.description(format!("Category `{}` has been created.", category.name));
                    e.field("ID", category.id.0, false);
                    if category.nsfw {
                        e.field("NSFW?", "Yes", false);
                    }
                    e.fields(fields);

                    e.timestamp(&chrono::offset::Utc::now());
                    e.footer(|f| {
                        f.text("Created")
                    });

                    e
                })).await;
            }
            _ => (),
        };
    }
}

pub async fn send_channel_delete(ctx: &Context, data: &ChannelDeleteEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let mut channel_id = 0;

    match &data.channel {
        Channel::Guild(channel) => {
            let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", channel.guild_id.0 as i64)
                .fetch_optional(pool)
                .await;
            if let Ok(query) = query {
                if let Some(query) = query {
                    channel_id = query.channel_id;
                }
            }
        }
        Channel::Category(category) => {
            let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", category.guild_id.0 as i64)
                .fetch_optional(pool)
                .await;
            if let Ok(query) = query {
                if let Some(query) = query {
                    channel_id = query.channel_id;
                }
            }
        }
         _ => (),
    }

    if channel_id != 0 {
        match &data.channel {
            Channel::Guild(channel) => {
                let category_name = if let Some(category) = channel.category_id {
                    category.name(ctx).await
                } else {
                    None
                };

                let mut fields = Vec::new();

                for perm in &channel.permission_overwrites {
                    match perm.kind {
                        PermissionOverwriteType::Member(x) => {
                            let user = x.to_user(ctx).await.unwrap_or_default();

                            fields.push((format!("Allowed Permissions for \"{}\"", user.tag()), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for \"{}\"", user.tag()), format!("{:?}", perm.deny), false));
                        }
                        PermissionOverwriteType::Role(x) => {
                            let role = x.to_role_cached(ctx).await.unwrap();

                            fields.push((format!("Allowed Permissions for role \"{}\"", role.name), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for role \"{}\"", role.name), format!("{:?}", perm.deny), false));
                        }
                        _ => (),
                    }
                }

                let _ = ChannelId(channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                    e.title("Channel Deleted");
                    if let Some(category) = category_name {
                        e.description(format!("Channel <#{}> (`{}`) has been deleted on the category {}.", channel.id.0, channel.name, category));
                    } else {
                        e.description(format!("Channel <#{}> (`{}`) has been delated outside a category.", channel.id.0, channel.name));
                    }
                    e.field("ID", channel.id.0, false);
                    e.field("Type", format!("{:?}", channel.kind), false);
                    e.fields(fields);

                    if channel.nsfw {
                        e.field("NSFW?", "Yes", false);
                    }

                    e.timestamp(&chrono::offset::Utc::now());
                    e.footer(|f| {
                        f.text("Deleted")
                    });

                    e
                })).await;
            }
            Channel::Category(category) => {
                let mut fields = Vec::new();

                for perm in &category.permission_overwrites {
                    match perm.kind {
                        PermissionOverwriteType::Member(x) => {
                            let user = x.to_user(ctx).await.unwrap_or_default();

                            fields.push((format!("Allowed Permissions for \"{}\"", user.tag()), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for \"{}\"", user.tag()), format!("{:?}", perm.deny), false));
                        }
                        PermissionOverwriteType::Role(x) => {
                            let role = x.to_role_cached(ctx).await.unwrap();

                            fields.push((format!("Allowed Permissions for role \"{}\"", role.name), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for role \"{}\"", role.name), format!("{:?}", perm.deny), false));
                        }
                        _ => (),
                    }
                }

                let _ = ChannelId(channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                    e.title("Category Deleted");
                    e.description(format!("Category `{}` has been deleted.", category.name));
                    e.field("ID", category.id.0, false);
                    if category.nsfw {
                        e.field("NSFW?", "Yes", false);
                    }
                    e.fields(fields);

                    e.timestamp(&chrono::offset::Utc::now());
                    e.footer(|f| {
                        f.text("Deleted")
                    });

                    e
                })).await;
            }
            _ => (),
        };
    }
}

pub async fn send_channel_update(ctx: &Context, data: &ChannelUpdateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let mut channel_id = 0;

    match &data.channel {
        Channel::Guild(channel) => {
            let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", channel.guild_id.0 as i64)
                .fetch_optional(pool)
                .await;
            if let Ok(query) = query {
                if let Some(query) = query {
                    channel_id = query.channel_id;
                }
            }
        }
        Channel::Category(category) => {
            let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", category.guild_id.0 as i64)
                .fetch_optional(pool)
                .await;
            if let Ok(query) = query {
                if let Some(query) = query {
                    channel_id = query.channel_id;
                }
            }
        }
         _ => (),
    }

    if channel_id != 0 {
        match &data.channel {
            Channel::Guild(channel) => {
                let category_name = if let Some(category) = channel.category_id {
                    category.name(ctx).await
                } else {
                    None
                };

                let mut fields = Vec::new();

                for perm in &channel.permission_overwrites {
                    match perm.kind {
                        PermissionOverwriteType::Member(x) => {
                            let user = x.to_user(ctx).await.unwrap_or_default();

                            fields.push((format!("Allowed Permissions for \"{}\"", user.tag()), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for \"{}\"", user.tag()), format!("{:?}", perm.deny), false));
                        }
                        PermissionOverwriteType::Role(x) => {
                            let role = x.to_role_cached(ctx).await.unwrap();

                            fields.push((format!("Allowed Permissions for role \"{}\"", role.name), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for role \"{}\"", role.name), format!("{:?}", perm.deny), false));
                        }
                        _ => (),
                    }
                }

                let _ = ChannelId(channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                    e.title("Channel Updated");
                    if let Some(category) = category_name {
                        e.description(format!("Channel <#{}> (`{}`) has been updated on the category {}.", channel.id.0, channel.name, category));
                    } else {
                        e.description(format!("Channel <#{}> (`{}`) has been updated outside a category.", channel.id.0, channel.name));
                    }
                    e.field("ID", channel.id.0, false);
                    e.field("Type", format!("{:?}", channel.kind), false);
                    e.fields(fields);

                    if channel.nsfw {
                        e.field("NSFW?", "Yes", false);
                    }

                    e.timestamp(&chrono::offset::Utc::now());
                    e.footer(|f| {
                        f.text("Updated")
                    });

                    e
                })).await;
            }
            Channel::Category(category) => {
                let mut fields = Vec::new();

                for perm in &category.permission_overwrites {
                    match perm.kind {
                        PermissionOverwriteType::Member(x) => {
                            let user = x.to_user(ctx).await.unwrap_or_default();

                            fields.push((format!("Allowed Permissions for \"{}\"", user.tag()), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for \"{}\"", user.tag()), format!("{:?}", perm.deny), false));
                        }
                        PermissionOverwriteType::Role(x) => {
                            let role = x.to_role_cached(ctx).await.unwrap();

                            fields.push((format!("Allowed Permissions for role \"{}\"", role.name), format!("{:?}", perm.allow), false));
                            fields.push((format!("Denied Permissions for role \"{}\"", role.name), format!("{:?}", perm.deny), false));
                        }
                        _ => (),
                    }
                }

                let _ = ChannelId(channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                    e.title("Category Updated");
                    e.description(format!("Category `{}` has been updated.", category.name));
                    e.field("ID", category.id.0, false);
                    if category.nsfw {
                        e.field("NSFW?", "Yes", false);
                    }
                    e.fields(fields);

                    e.timestamp(&chrono::offset::Utc::now());
                    e.footer(|f| {
                        f.text("Updated")
                    });

                    e
                })).await;
            }
            _ => (),
        };
    }
}

pub async fn send_channel_pins_update(ctx: &Context, data: &ChannelPinsUpdateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.unwrap().0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Channel Pins Updated");
                e.description(format!("Pins have been updated on <#{}>", data.channel_id.0));

                if let Some(timestamp) = data.last_pin_timestamp {
                    e.footer(|f| {
                        f.text("Last pin")
                    });
                    e.timestamp(&timestamp);
                }

                e
            })).await;
        }
    }
}

pub async fn send_guild_ban_add(ctx: &Context, data: &GuildBanAddEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("User Banned");
                e.field("ID", data.user.id.0, false);

                e.author(|a| {
                    a.icon_url(data.user.face());
                    a.name(data.user.tag())
                });

                if data.user.bot {
                    e.description("User is a BOT account.");
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Banned")
                });

                e
            })).await;
        }
    }
}

pub async fn send_guild_ban_remove(ctx: &Context, data: &GuildBanRemoveEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("User Unbanned");
                e.field("ID", data.user.id.0, false);

                e.author(|a| {
                    a.icon_url(data.user.face());
                    a.name(data.user.tag())
                });

                if data.user.bot {
                    e.description("User is a BOT account.");
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Unbanned")
                });

                e
            })).await;
        }
    }
}

pub async fn send_guild_emojis_update(ctx: &Context, data: &GuildEmojisUpdateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Emojis Updated");
                for (_, emoji) in &data.emojis {
                    if emoji.animated {
                        e.field(format!("<a:{}:{}>", emoji.name, emoji.id.0), format!("https://cdn.discordapp.com/emojis/{}.gif", emoji.id.0), false);
                    } else {
                        e.field(format!("<:{}:{}>", emoji.name, emoji.id.0), format!("https://cdn.discordapp.com/emojis/{}.gif", emoji.id.0), false);
                    }
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Updated")
                });

                e
            })).await;
        }
    }
}

pub async fn send_guild_integrations_update(ctx: &Context, data: &GuildIntegrationsUpdateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(query) = query {
        if let Some(query) = query {
            let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                e.title("Integrations have been modified");

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| {
                    f.text("Modified")
                });

                e
            })).await;
        }
    }
}
