use crate::ConnectionPool;
use crate::utils::basic_functions::string_to_seconds;
use crate::utils::checks::BOT_HAS_MANAGE_ROLES_CHECK;

use serenity::{
    prelude::Context,
    model::{
        channel::Message,
        guild::Member,
        id::{
            UserId,
            MessageId,
        },
    },
    framework::standard::{
        Args,
        CommandResult,
        Delimiter,
        macros::command,
    },
};
use regex::Regex;
use futures::{
    //future::FutureExt,
    stream,
    StreamExt,
};

pub async fn parse_member(ctx: &Context, msg: &Message, member_name: String) -> Result<Member, String> {
    let mut members = Vec::new();

    if let Ok(id) = member_name.parse::<u64>() {
        let member = &msg.guild_id.unwrap().member(ctx, id).await;
        match member {
            Ok(m) => Ok(m.to_owned()),
            Err(why) => Err(why.to_string()),
        }
    } else if member_name.starts_with("<@") && member_name.ends_with('>') {
        let re = Regex::new("[<@!>]").unwrap();
        let member_id = re.replace_all(&member_name, "").into_owned();
        let member = &msg.guild_id.unwrap().member(ctx, UserId(member_id.parse::<u64>().unwrap())).await;

        match member {
            Ok(m) => Ok(m.to_owned()),
            Err(why) => Err(why.to_string()),
        }
    } else {
        let guild = &msg.guild(ctx).await.unwrap();
        let member_name = member_name.split('#').next().unwrap();

        for m in guild.members.values() {
            if m.display_name() == std::borrow::Cow::Borrowed(member_name) ||
                m.user.name == member_name
            {
                members.push(m);
            }
        }

        if members.is_empty() {
            let similar_members = &guild.members_containing(&member_name, false, false).await;

            let mut members_string =  stream::iter(similar_members.iter())
                .map(|m| async move {
                    let member = &m.0.user;
                    format!("`{}`|", member.name)
                })
                .fold(String::new(), |mut acc, c| async move {
                    acc.push_str(&c.await);
                    acc
                }).await;

            let message = {
                if members_string == "" {
                    format!("No member named '{}' was found.", member_name)
                } else {
                    members_string.pop();
                    format!("No member named '{}' was found.\nDid you mean: {}", member_name, members_string)
                }
            };
            Err(message)
        } else if members.len() == 1 {
            Ok(members[0].to_owned())
        } else {
            let mut members_string =  stream::iter(members.iter())
                .map(|m| async move {
                    let member = &m.user;
                    format!("`{}#{}`|", member.name, member.discriminator)
                })
                .fold(String::new(), |mut acc, c| async move {
                    acc.push_str(&c.await);
                    acc
                }).await;

            members_string.pop();

            let message = format!("Multiple members with the same name where found: '{}'", &members_string);
            Err(message)
        }
    }
}

/// Kicks the specified member with an optional reason.
///
/// Usage:
/// `kick @user`
/// `kick "user name"`
/// `kick "user name#3124"`
/// `kick 135423120268984330 he is a very bad person.`
#[command]
#[required_permissions(KICK_MEMBERS)]
#[min_args(1)]
#[only_in("guilds")]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member_arg = args.single_quoted::<String>()?;
    let member = parse_member(ctx, &msg, member_arg).await;

    let reason = args.remains();

    match member {
        Ok(m) => {
            if let Some(r) = reason {
                m.kick_with_reason(ctx, r).await?;
            } else {
                m.kick(ctx).await?;
            }
            msg.reply(ctx, format!("Successfully kicked member `{}#{}` with id `{}`", m.user.name, m.user.discriminator, m.user.id.0)).await?;
        },
        Err(why) => {msg.reply(ctx, why.to_string()).await?;},
    }

    Ok(())
}

/// Bans the specified member with an optional reason.
///
/// Usage:
/// `ban @user`
/// `ban "user name"`
/// `ban "user name#3124"`
/// `ban 135423120268984330 he is a very bad person.`
#[command]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
#[only_in("guilds")]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member_arg = args.single_quoted::<String>()?;
    let member = parse_member(ctx, &msg, member_arg).await;

    let reason = args.remains();

    match member {
        Ok(m) => {
            if let Some(r) = reason {
                m.ban_with_reason(ctx, 1, &r).await?;
            } else {
                m.ban(ctx, 1).await?;
            }
            msg.reply(ctx, format!("Successfully banned member `{}#{}` with id `{}`", m.user.name, m.user.discriminator, m.user.id.0)).await?;
        },
        Err(why) => {msg.reply(ctx, why.to_string()).await?;},
    }

    Ok(())
}

/// Deletes X number of messages from the current channel.
/// If the messages are older than 2 weeks, due to api limitations, they will not get deleted.
///
/// Usage: `clear 20`
#[command]
#[required_permissions(MANAGE_MESSAGES)]
#[num_args(1)]
#[only_in("guilds")]
#[aliases(purge)]
async fn clear(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = args.single::<u64>();
    match num {
        Err(_) => {msg.channel_id.say(ctx, "The value provided was not a valid number").await?;},
        Ok(n) => {
            let channel = &msg.channel(ctx).await.unwrap().guild().unwrap();

            let messages = &channel.messages(ctx, |r| r.before(&msg.id).limit(n)).await?;
            let messages_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            channel.delete_messages(ctx, messages_ids).await?;

            msg.channel_id.say(ctx, format!("Successfully deleted `{}` message", n)).await?;
        }
    }
    Ok(())
}

/// Mutes a member with the configured role.
/// To configure a role, someone who has the "manage guild" permissions needs to run the next command:
/// 
/// `configure guild mute_role @role_mention`
/// or
/// `configure guild mute_role role_id`
///
/// Usage:
/// `mute @member`
/// `mute 135423120268984330`
#[command]
#[required_permissions(MANAGE_ROLES)]
#[min_args(1)]
#[only_in("guilds")]
#[aliases(mute, pmute, permamute, perma_mute, permanentmute)]
#[checks(bot_has_manage_roles)]
async fn permanent_mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member_arg = args.single_quoted::<String>()?;
    let mut member = parse_member(ctx, msg, member_arg).await?;

    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let row = sqlx::query!("SELECT role_id FROM muted_roles WHERE guild_id = $1", msg.guild_id.unwrap().0 as i64)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        member.add_role(ctx, row.role_id as u64).await?;
        msg.reply(ctx, format!("Successfully muted member `{}#{}` with id `{}`",
            member.user.name, member.user.discriminator, member.user.id.0)).await?;
    } else {
        msg.reply(ctx, "The server doesn't have a muted role configured, please tell someone with the \"manage guild\" permission to run the following command to configure one:\n`configure guild mute_role @role_mention`").await?;
        return Ok(());
    }

    Ok(())
}

/// Mute yourself with the configured role.
/// To configure a role, someone who has the "manage guild" permissions needs to run the next command:
/// 
/// `configure guild mute_role @role_mention`
/// or
/// `configure guild mute_role role_id`
///
/// Usage: `selfmute`
#[command]
#[only_in("guilds")]
#[aliases(self_mute_permanent, mute_self_permanent, permanentselfmute, selfmutepermanent, selfmutep, selfpmute, self_permanent_mute, self_perma_mute, perma_self_mute, mute_self_perma, pselfmute)]
#[checks(bot_has_manage_roles)]
async fn permanent_self_mute(ctx: &Context, msg: &Message) -> CommandResult {
    permanent_mute(ctx, msg, Args::new(&msg.author.id.0.to_string(), &[Delimiter::Single(' ')])).await
}

/// Mute a Member for a temporal amount of time.
///
/// Default is 1 Hour.
/// Supports the same time stamps as `reminder`.
///
/// To configure a role, someone who has the "manage guild" permissions needs to run the next command:
/// 
/// `configure guild mute_role @role_mention`
/// or
/// `configure guild mute_role role_id`
///
/// Usage:
/// `tempmute @member`
/// `tempmute @member "2D 12h"`
/// `tempmute @member "1W" posted porn on #general`
#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_ROLES)]
#[min_args(1)]
#[aliases(tempmute, tmute, temporalmute, temp_mute)]
#[checks(bot_has_manage_roles)]
async fn temporal_mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let raw_member = args.single_quoted::<String>()?;
    let mut member = parse_member(ctx, msg, raw_member).await?;

    let unformatted_time = args.single_quoted::<String>().unwrap_or("1h".to_string());
    let seconds = string_to_seconds(unformatted_time);

    if seconds < 30 {
        msg.reply(ctx, "Duration is too short").await?;
        return Ok(());
    }

    let text = args.rest();
    let message = if text.is_empty() {
        None
    } else {
        Some(text)
    };

    let row = sqlx::query!("SELECT role_id FROM muted_roles WHERE guild_id = $1", msg.guild_id.unwrap().0 as i64)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        member.add_role(ctx, row.role_id as u64).await?;

        sqlx::query!("INSERT INTO muted_members (date, message_id, channel_id, guild_id, user_id, message) VALUES ($1, $2, $3, $4, $5, $6)",
            chrono::offset::Utc::now() + chrono::Duration::seconds(seconds as i64),
            msg.id.0 as i64,
            msg.channel_id.0 as i64,
            msg.guild_id.unwrap().0 as i64,
            member.user.id.0 as i64,
            message,
        )
        .execute(pool)
        .await?;

        msg.reply(ctx, format!("Successfully muted member `{}#{}` with id `{}`\n until `{}`",
            member.user.name, member.user.discriminator, member.user.id.0,
            chrono::offset::Utc::now() + chrono::Duration::seconds(seconds as i64))).await?;
    } else {
        msg.reply(ctx, "The server doesn't have a muted role configured, please tell someone with the \"manage guild\" permission to run the following command to configure one:\n`configure guild mute_role @role_mention`").await?;
        return Ok(());
    }

    Ok(())
}

/// Mute yourself for a temporal amount of time.
///
/// Default is 1 Hour.
/// Supports the same time stamps as `reminder`.
///
/// To configure a role, someone who has the "manage guild" permissions needs to run the next command:
/// 
/// `configure guild mute_role @role_mention`
/// or
/// `configure guild mute_role role_id`
///
/// Usage:
/// `selftempmute`
/// `selftempmute "2D 12h"`
/// `selftempmute "1W" im an idiot :D`
#[command]
#[only_in("guilds")]
#[aliases(self_mute_temporal, mute_self_temporal, temporalselfmute, selfmutetemporal, selfmutet, selftmute, self_temporal_mute, self_temp_mute, temp_self_mute, mute_self_temp, tselfmute, selftempmute)]
#[checks(bot_has_manage_roles)]
async fn temporal_self_mute(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    temporal_mute(ctx, msg, Args::new(&format!("{} {}", msg.author.id.0.to_string(), args.message()), &[Delimiter::Single(' ')])).await
}
