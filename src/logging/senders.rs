use crate::global_data::DatabasePool;
use crate::utils::logging::{guild_has_logging, LoggingEvents};

use serenity::{
    model::{
        channel::{Channel, Embed, PermissionOverwriteType, ReactionType},
        event::*,
        id::UserId,
    },
    prelude::Context,
    prelude::Mentionable,
};

#[instrument(skip(ctx))]
pub async fn send_message_update(ctx: &Context, data: &MessageUpdateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::MessageUpdate, data.guild_id.unwrap()).await
    {
        let old_message = sqlx::query!(
            "SELECT content_history FROM log_messages WHERE id = $1",
            data.id.0 as i64
        )
        .fetch_optional(&pool)
        .await;

        if let Ok(Some(old_message)) = old_message {
            if let Some(old_message) = old_message.content_history {
                let old_message_content = old_message.get(old_message.len().saturating_sub(1));
                if old_message_content.unwrap_or(&String::new())
                    == &data.content.clone().unwrap_or_default()
                {
                    return;
                }

                let embed = Embed::fake(|e| {
                    e.title("Message Updated");

                    e.field("Message ID", &data.id.0, false);

                    let content = old_message_content
                        .unwrap_or(&String::from("- Unknown Content."))
                        .to_owned();
                    if content.len() > 1000 {
                        e.field("Original Content (1)", &content[..content.len() / 2], false);
                        e.field("Original Content (2)", &content[content.len() / 2..], false);
                    } else if !content.is_empty() {
                        e.field("Original Content", &content, false);
                    }

                    let content = data
                        .content
                        .as_ref()
                        .unwrap_or(&String::from("- Unknown Content."))
                        .to_owned();

                    if content.len() > 1000 {
                        e.field("New Content (1)", &content[..content.len() / 2], false);
                        e.field("New Content (2)", &content[content.len() / 2..], false);
                    } else if !content.is_empty() {
                        e.field("New Content", &content, false);
                    }

                    if let Some(author) = &data.author {
                        e.description(format!(
                            "[This](https://discord.com/channels/{}/{}/{}) message was sent by {}",
                            data.guild_id.unwrap().0,
                            data.channel_id.0,
                            data.id.0,
                            author.mention()
                        ));
                        e.author(|a| {
                            a.icon_url(author.face());
                            a.name(author.tag())
                        });
                    }

                    e.timestamp(&chrono::offset::Utc::now());
                    e.footer(|f| f.text("Updated"));

                    e
                });

                let mut split = channel_data.webhook_url.split('/');
                let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
                let token = split.next().unwrap();

                match &ctx.http.get_webhook_with_token(id, token).await {
                    Ok(hook) => {
                        if let Err(why) = hook
                            .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                            .await
                        {
                            error!("Error Sending Hook: {}", why)
                        }
                    }
                    Err(why) => {
                        error!("Error Obtaining Hook: {}", why);
                        return;
                    }
                }
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_message_delete(ctx: &Context, data: &MessageDeleteEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::MessageDelete, data.guild_id.unwrap()).await
    {
        let raw_message = sqlx::query!(
                "SELECT content, author_id, attachments, pinned, edited_timestamp, tts, webhook_id FROM log_messages WHERE id = $1",
                data.message_id.0 as i64
            )
            .fetch_optional(&pool)
            .await;

        if let Ok(Some(msg)) = raw_message {
            let author = if let Ok(x) = UserId(msg.author_id as u64).to_user(ctx).await {
                x
            } else {
                return;
            };

            let embed = Embed::fake(|e| {
                e.title("Message Deleted");
                e.description(format!(
                    "Message from <#{}> with the id `{}`",
                    data.channel_id.0, data.message_id.0
                ));
                e.author(|a| {
                    a.icon_url(author.face());
                    a.name(author.tag())
                });

                let content = msg
                    .content
                    .as_ref()
                    .unwrap_or(&String::from("- Unknown Content."))
                    .to_owned()
                    + "\u{200b}";

                if content.len() > 1000 {
                    e.field("Content (1)", &content[..content.len() / 2], false);
                    e.field("Content (2)", &content[content.len() / 2..], false);
                } else {
                    e.field("Content", &content, false);
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
            });

            let mut split = channel_data.webhook_url.split('/');
            let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
            let token = split.next().unwrap();

            match &ctx.http.get_webhook_with_token(id, token).await {
                Ok(hook) => {
                    if let Err(why) = hook
                        .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                        .await
                    {
                        error!("Error Sending Hook: {}", why)
                    }
                }
                Err(why) => {
                    error!("Error Obtaining Hook: {}", why);
                    return;
                }
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_member_add(ctx: &Context, data: &GuildMemberAddEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildMemberAdd, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
            e.title("Member Joined");
            e.author(|a| {
                a.icon_url(data.member.user.face());
                a.name(data.member.user.tag())
            });
            e.field(
                "Created at",
                &data.member.user.created_at().to_rfc2822(),
                false,
            );
            if let Some(x) = &data.member.joined_at {
                e.field("Joined at", x.to_rfc2822(), false);
            }
            e.field("ID", &data.member.user.id.0, false);

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_member_remove(ctx: &Context, data: &GuildMemberRemoveEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildMemberRemove, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
            e.title("Member Left");
            e.author(|a| {
                a.icon_url(data.user.face());
                a.name(data.user.tag())
            });
            e.field("Created at", &data.user.created_at().to_rfc2822(), false);
            e.field("ID", &data.user.id.0, false);

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_message_delete_bulk(ctx: &Context, data: &MessageDeleteBulkEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) = guild_has_logging(
        &pool,
        LoggingEvents::MessageDeleteBulk,
        data.guild_id.unwrap(),
    )
    .await
    {
        let embed = Embed::fake(|e| {
            e.title("Bulk of messages Deleted");
            e.description(format!("Deleted on <#{}>", &data.channel_id.0));
            e.field("Number of messages:", &data.ids.len(), false);

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_role_create(ctx: &Context, data: &GuildRoleCreateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildRoleCreate, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
            e.title("Role Created");
            e.field("ID", &data.role.id.0, false);
            e.field("Mention", format!("<@&{}>", &data.role.id.0), false);
            e.field("Mentionable", &data.role.mentionable, false);
            e.field(
                "Permissions",
                format!("{:?}", &data.role.permissions),
                false,
            );
            e.colour(data.role.colour);
            e.timestamp(&chrono::offset::Utc::now());
            e.footer(|f| f.text("Created"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_role_delete(ctx: &Context, data: &GuildRoleDeleteEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildRoleDelete, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
            e.title("Role Deleted");
            e.field("ID", &data.role_id.0, false);
            e.timestamp(&chrono::offset::Utc::now());
            e.footer(|f| f.text("Deleted"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

// discord sends update of every single role, even if a role has not changed...
// possition changes smh
#[instrument(skip(_ctx))]
pub async fn send_guild_role_update(_ctx: &Context, _data: &GuildRoleUpdateEvent) {
    /*
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) = guild_has_logging(&pool, LoggingEvents::GuildRoleUpdate, data.guild_id.unwrap()).await {
        let embed = Embed::fake(|e| {
            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook.execute(&ctx.http, false, |m| {
                    m.embeds(vec![embed])
                }).await {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return
            }
        }
    }
    */
}

// Why does this event trigger for no reason reeeee
#[instrument(skip(ctx))]
pub async fn send_guild_member_update(ctx: &Context, data: &GuildMemberUpdateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildMemberUpdate, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
            e.title("Member Updated");
            e.author(|a| {
                a.icon_url(data.user.face());
                a.name(data.user.tag())
            });
            e.description(format!("The user <@!{}> has been updated", &data.user.id.0));
            e.field("User ID", &data.user.id.0, false);
            if let Some(nick) = &data.nick {
                e.field("Nickname", nick, false);
            }
            if data.roles.len() > 1 {
                e.field(
                    "Roles",
                    &data
                        .roles
                        .iter()
                        .map(|i| format!("<@&{}>", i.0))
                        .collect::<Vec<_>>()
                        .join(" | "),
                    false,
                );
            }
            e.timestamp(&chrono::offset::Utc::now());
            e.footer(|f| f.text("Updated"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_reaction_add(ctx: &Context, data: &ReactionAddEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) = guild_has_logging(
        &pool,
        LoggingEvents::ReactionAdd,
        data.reaction.guild_id.unwrap(),
    )
    .await
    {
        let reaction = &data.reaction;

        if let ReactionType::Custom { animated, id, name } = &reaction.emoji {
            let embed = Embed::fake(|e| {
                e.title("Reaction Added");
                e.description(format!("Reaction on <#{0}> in [this message](https://discord.com/channels/{1}/{0}/{2}) by <@!{3}>", reaction.channel_id.0, reaction.guild_id.unwrap().0, reaction.message_id.0, reaction.user_id.unwrap().0));
                e.field("User ID", reaction.user_id.unwrap().0, false);

                e.field(
                    "Emoji",
                    {
                        if *animated {
                            format!("<a:{}:{}>", name.as_ref().unwrap(), id.0)
                        } else {
                            format!("<:{}:{}>", name.as_ref().unwrap(), id.0)
                        }
                    },
                    false,
                );

                if *animated {
                    e.image(format!("https://cdn.discordapp.com/emojis/{}.gif", id));
                } else {
                    e.image(format!("https://cdn.discordapp.com/emojis/{}.png", id));
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Added"));

                e
            });

            let mut split = channel_data.webhook_url.split('/');
            let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
            let token = split.next().unwrap();

            match &ctx.http.get_webhook_with_token(id, token).await {
                Ok(hook) => {
                    if let Err(why) = hook
                        .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                        .await
                    {
                        error!("Error Sending Hook: {}", why)
                    }
                }
                Err(why) => {
                    error!("Error Obtaining Hook: {}", why);
                    return;
                }
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_reaction_remove(ctx: &Context, data: &ReactionRemoveEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) = guild_has_logging(
        &pool,
        LoggingEvents::ReactionRemove,
        data.reaction.guild_id.unwrap(),
    )
    .await
    {
        let reaction = &data.reaction;

        if let ReactionType::Custom { animated, id, name } = &reaction.emoji {
            let embed = Embed::fake(|e| {
                e.title("Reaction Added");
                e.description(format!("Reaction on <#{0}> in [this message](https://discord.com/channels/{1}/{0}/{2}) by <@!{3}>", reaction.channel_id.0, reaction.guild_id.unwrap().0, reaction.message_id.0, reaction.user_id.unwrap().0));
                e.field("User ID", reaction.user_id.unwrap().0, false);

                e.field(
                    "Emoji",
                    {
                        if *animated {
                            format!("<a:{}:{}>", name.as_ref().unwrap(), id.0)
                        } else {
                            format!("<:{}:{}>", name.as_ref().unwrap(), id.0)
                        }
                    },
                    false,
                );

                if *animated {
                    e.image(format!("https://cdn.discordapp.com/emojis/{}.gif", id));
                } else {
                    e.image(format!("https://cdn.discordapp.com/emojis/{}.png", id));
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Removed"));

                e
            });

            let mut split = channel_data.webhook_url.split('/');
            let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
            let token = split.next().unwrap();

            match &ctx.http.get_webhook_with_token(id, token).await {
                Ok(hook) => {
                    if let Err(why) = hook
                        .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                        .await
                    {
                        error!("Error Sending Hook: {}", why)
                    }
                }
                Err(why) => {
                    error!("Error Obtaining Hook: {}", why);
                    return;
                }
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_reaction_remove_all(ctx: &Context, data: &ReactionRemoveAllEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) = guild_has_logging(
        &pool,
        LoggingEvents::ReactionRemoveAll,
        data.guild_id.unwrap(),
    )
    .await
    {
        let embed = Embed::fake(|e| {
            e.title("Reactions Cleared");
            e.description(format!(
                "Reactions on <#{0}> in [this message](https://discord.com/channels/{1}/{0}/{2})",
                data.channel_id.0,
                data.guild_id.unwrap().0,
                data.message_id.0
            ));

            e.timestamp(&chrono::offset::Utc::now());
            e.footer(|f| f.text("Cleared"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_channel_create(ctx: &Context, data: &ChannelCreateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let channel_data = match &data.channel {
        Channel::Guild(channel) => {
            if let Some(x) =
                guild_has_logging(&pool, LoggingEvents::ChannelCreate, channel.guild_id).await
            {
                x
            } else {
                return;
            }
        }
        Channel::Category(channel) => {
            if let Some(x) =
                guild_has_logging(&pool, LoggingEvents::ChannelCreate, channel.guild_id).await
            {
                x
            } else {
                return;
            }
        }
        _ => return,
    };

    let embed = match &data.channel {
        Channel::Guild(channel) => {
            let category_name = if let Some(category) = channel.parent_id {
                category.name(ctx).await
            } else {
                None
            };

            let mut fields = Vec::new();

            for perm in &channel.permission_overwrites {
                match perm.kind {
                    PermissionOverwriteType::Member(x) => {
                        let user = x.to_user(ctx).await.unwrap_or_default();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    PermissionOverwriteType::Role(x) => {
                        let role = x.to_role_cached(ctx).unwrap();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    _ => (),
                }
            }

            Embed::fake(|e| {
                e.title("Channel Created");
                if let Some(category) = category_name {
                    e.description(format!(
                        "Channel <#{}> (`{}`) has been created on the category {}.",
                        channel.id.0, channel.name, category
                    ));
                } else {
                    e.description(format!(
                        "Channel <#{}> (`{}`) has been created outside a category.",
                        channel.id.0, channel.name
                    ));
                }
                e.field("ID", channel.id.0, false);
                e.field("Type", format!("{:?}", channel.kind), false);
                e.fields(fields);

                if channel.nsfw {
                    e.field("NSFW?", "Yes", false);
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Created"));

                e
            })
        }
        Channel::Category(category) => {
            let mut fields = Vec::new();

            for perm in &category.permission_overwrites {
                match perm.kind {
                    PermissionOverwriteType::Member(x) => {
                        let user = x.to_user(ctx).await.unwrap_or_default();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    PermissionOverwriteType::Role(x) => {
                        let role = x.to_role_cached(ctx).unwrap();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    _ => (),
                }
            }

            Embed::fake(|e| {
                e.title("Category Created");
                e.description(format!("Category `{}` has been created.", category.name));
                e.field("ID", category.id.0, false);
                if category.nsfw {
                    e.field("NSFW?", "Yes", false);
                }
                e.fields(fields);

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Created"));

                e
            })
        }
        _ => return,
    };

    let mut split = channel_data.webhook_url.split('/');
    let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
    let token = split.next().unwrap();

    match &ctx.http.get_webhook_with_token(id, token).await {
        Ok(hook) => {
            if let Err(why) = hook
                .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                .await
            {
                error!("Error Sending Hook: {}", why)
            }
        }
        Err(why) => {
            error!("Error Obtaining Hook: {}", why);
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_channel_delete(ctx: &Context, data: &ChannelDeleteEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let channel_data = match &data.channel {
        Channel::Guild(channel) => {
            if let Some(x) =
                guild_has_logging(&pool, LoggingEvents::ChannelDelete, channel.guild_id).await
            {
                x
            } else {
                return;
            }
        }
        Channel::Category(channel) => {
            if let Some(x) =
                guild_has_logging(&pool, LoggingEvents::ChannelDelete, channel.guild_id).await
            {
                x
            } else {
                return;
            }
        }
        _ => return,
    };

    let embed = match &data.channel {
        Channel::Guild(channel) => {
            let category_name = if let Some(category) = channel.parent_id {
                category.name(ctx).await
            } else {
                None
            };

            let mut fields = Vec::new();

            for perm in &channel.permission_overwrites {
                match perm.kind {
                    PermissionOverwriteType::Member(x) => {
                        let user = x.to_user(ctx).await.unwrap_or_default();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    PermissionOverwriteType::Role(x) => {
                        let role = x.to_role_cached(ctx).unwrap();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    _ => (),
                }
            }

            Embed::fake(|e| {
                e.title("Channel Deleted");
                if let Some(category) = category_name {
                    e.description(format!(
                        "Channel <#{}> (`{}`) has been deleted from the category {}.",
                        channel.id.0, channel.name, category
                    ));
                } else {
                    e.description(format!(
                        "Channel <#{}> (`{}`) has been deleted; it was outside a category.",
                        channel.id.0, channel.name
                    ));
                }
                e.field("ID", channel.id.0, false);
                e.field("Type", format!("{:?}", channel.kind), false);
                e.fields(fields);

                if channel.nsfw {
                    e.field("NSFW?", "Yes", false);
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Deleted"));

                e
            })
        }
        Channel::Category(category) => {
            let mut fields = Vec::new();

            for perm in &category.permission_overwrites {
                match perm.kind {
                    PermissionOverwriteType::Member(x) => {
                        let user = x.to_user(ctx).await.unwrap_or_default();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    PermissionOverwriteType::Role(x) => {
                        let role = x.to_role_cached(ctx).unwrap();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    _ => (),
                }
            }

            Embed::fake(|e| {
                e.title("Category Deleted");
                e.description(format!("Category `{}` has been deleted.", category.name));
                e.field("ID", category.id.0, false);
                if category.nsfw {
                    e.field("NSFW?", "Yes", false);
                }
                e.fields(fields);

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Deleted"));

                e
            })
        }
        _ => return,
    };

    let mut split = channel_data.webhook_url.split('/');
    let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
    let token = split.next().unwrap();

    match &ctx.http.get_webhook_with_token(id, token).await {
        Ok(hook) => {
            if let Err(why) = hook
                .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                .await
            {
                error!("Error Sending Hook: {}", why)
            }
        }
        Err(why) => {
            error!("Error Obtaining Hook: {}", why);
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_channel_update(ctx: &Context, data: &ChannelUpdateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let channel_data = match &data.channel {
        Channel::Guild(channel) => {
            if let Some(x) =
                guild_has_logging(&pool, LoggingEvents::ChannelUpdate, channel.guild_id).await
            {
                x
            } else {
                return;
            }
        }
        Channel::Category(channel) => {
            if let Some(x) =
                guild_has_logging(&pool, LoggingEvents::ChannelUpdate, channel.guild_id).await
            {
                x
            } else {
                return;
            }
        }
        _ => return,
    };

    let embed = match &data.channel {
        Channel::Guild(channel) => {
            let category_name = if let Some(category) = channel.parent_id {
                category.name(ctx).await
            } else {
                None
            };

            let mut fields = Vec::new();

            for perm in &channel.permission_overwrites {
                match perm.kind {
                    PermissionOverwriteType::Member(x) => {
                        let user = x.to_user(ctx).await.unwrap_or_default();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    PermissionOverwriteType::Role(x) => {
                        let role = x.to_role_cached(ctx).unwrap();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    _ => (),
                }
            }

            Embed::fake(|e| {
                e.title("Channel Updated");
                if let Some(category) = category_name {
                    e.description(format!(
                        "Channel <#{}> (`{}`) has been updated on the category {}.",
                        channel.id.0, channel.name, category
                    ));
                } else {
                    e.description(format!(
                        "Channel <#{}> (`{}`) has been updated outside a category.",
                        channel.id.0, channel.name
                    ));
                }
                e.field("ID", channel.id.0, false);
                e.field("Type", format!("{:?}", channel.kind), false);
                e.fields(fields);

                if channel.nsfw {
                    e.field("NSFW?", "Yes", false);
                }

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Updated"));

                e
            })
        }
        Channel::Category(category) => {
            let mut fields = Vec::new();

            for perm in &category.permission_overwrites {
                match perm.kind {
                    PermissionOverwriteType::Member(x) => {
                        let user = x.to_user(ctx).await.unwrap_or_default();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for \"{}\"", user.tag()),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    PermissionOverwriteType::Role(x) => {
                        let role = x.to_role_cached(ctx).unwrap();

                        if !perm.allow.is_empty() {
                            fields.push((
                                format!("Allowed Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.allow),
                                false,
                            ));
                        }

                        if !perm.deny.is_empty() {
                            fields.push((
                                format!("Denied Permissions for role \"{}\"", role.name),
                                format!("{:?}", perm.deny),
                                false,
                            ));
                        }
                    }
                    _ => (),
                }
            }

            Embed::fake(|e| {
                e.title("Category Updated");
                e.description(format!("Category `{}` has been updated.", category.name));
                e.field("ID", category.id.0, false);
                if category.nsfw {
                    e.field("NSFW?", "Yes", false);
                }
                e.fields(fields);

                e.timestamp(&chrono::offset::Utc::now());
                e.footer(|f| f.text("Updated"));

                e
            })
        }
        _ => return,
    };

    let mut split = channel_data.webhook_url.split('/');
    let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
    let token = split.next().unwrap();

    match &ctx.http.get_webhook_with_token(id, token).await {
        Ok(hook) => {
            if let Err(why) = hook
                .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                .await
            {
                error!("Error Sending Hook: {}", why)
            }
        }
        Err(why) => {
            error!("Error Obtaining Hook: {}", why);
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_channel_pins_update(ctx: &Context, data: &ChannelPinsUpdateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) = guild_has_logging(
        &pool,
        LoggingEvents::ChannelPinsUpdate,
        data.guild_id.unwrap(),
    )
    .await
    {
        let embed = Embed::fake(|e| {
            e.title("Channel Pins Updated");
            e.description(format!(
                "Pins have been updated on <#{}>",
                data.channel_id.0
            ));

            if let Some(timestamp) = data.last_pin_timestamp {
                e.footer(|f| f.text("Last pin"));
                e.timestamp(&timestamp);
            }

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_ban_add(ctx: &Context, data: &GuildBanAddEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildBanAdd, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
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
            e.footer(|f| f.text("Banned"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_ban_remove(ctx: &Context, data: &GuildBanRemoveEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildBanRemove, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
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
            e.footer(|f| f.text("Unbanned"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_emojis_update(ctx: &Context, data: &GuildEmojisUpdateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildEmojisUpdate, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
            e.title("Emojis Updated");
            for emoji in data.emojis.values() {
                if emoji.animated {
                    e.field(
                        format!("<a:{}:{}>", emoji.name, emoji.id.0),
                        format!("https://cdn.discordapp.com/emojis/{}.gif", emoji.id.0),
                        false,
                    );
                } else {
                    e.field(
                        format!("<:{}:{}>", emoji.name, emoji.id.0),
                        format!("https://cdn.discordapp.com/emojis/{}.gif", emoji.id.0),
                        false,
                    );
                }
            }

            e.timestamp(&chrono::offset::Utc::now());
            e.footer(|f| f.text("Updated"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}

#[instrument(skip(ctx))]
pub async fn send_guild_integrations_update(ctx: &Context, data: &GuildIntegrationsUpdateEvent) {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Some(channel_data) =
        guild_has_logging(&pool, LoggingEvents::GuildIntegrationsUpdate, data.guild_id).await
    {
        let embed = Embed::fake(|e| {
            e.title("Integrations have been modified");

            e.timestamp(&chrono::offset::Utc::now());
            e.footer(|f| f.text("Modified"));

            e
        });

        let mut split = channel_data.webhook_url.split('/');
        let id = split.nth(5).unwrap().parse::<u64>().unwrap_or_default();
        let token = split.next().unwrap();

        match &ctx.http.get_webhook_with_token(id, token).await {
            Ok(hook) => {
                if let Err(why) = hook
                    .execute(&ctx.http, false, |m| m.embeds(vec![embed]))
                    .await
                {
                    error!("Error Sending Hook: {}", why)
                }
            }
            Err(why) => {
                error!("Error Obtaining Hook: {}", why);
                return;
            }
        }
    }
}
