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
        macros::command,
    },
};
use regex::Regex;
use futures::{
    //future::FutureExt,
    stream,
    StreamExt,
};

async fn parse_member(ctx: &mut Context, msg: &Message, args: Args) -> Result<Member, String> {
    let member_name = args.message();
    let mut members = Vec::new();

    if let Ok(id) = member_name.parse::<u64>() {
        let member = &msg.guild_id.unwrap().member(&ctx, id).await;
        match member {
            Ok(m) => Ok(m.to_owned()),
            Err(why) => Err(why.to_string()),
        }
    } else if member_name.starts_with("<@") && member_name.ends_with('>') {
        let re = Regex::new("[<@!>]").unwrap();
        let member_id = re.replace_all(member_name, "").into_owned();
        let member = &msg.guild_id.unwrap().member(&ctx, UserId(member_id.parse::<u64>().unwrap())).await;

        match member {
            Ok(m) => Ok(m.to_owned()),
            Err(why) => Err(why.to_string()),
        }
    } else {
        let guild = &msg.guild(&ctx).await.unwrap();
        let rguild = &guild.read().await;
        let member_name = member_name.split('#').next().unwrap();

        for m in rguild.members.values() {
            if m.display_name().await == std::borrow::Cow::Borrowed(member_name) ||
                m.user.read().await.name == member_name
            {
                members.push(m);
            }
        }

        if members.is_empty() {
            let similar_members = &rguild.members_containing(&member_name, false, false).await;

            let mut members_string =  stream::iter(similar_members.iter())
                .map(|m| async move {
                    let member = m.0.user.read().await;
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
                    let member = m.user.read().await;
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

/// Kicks the specified member
/// Usage: `.kick @user` or `.kick user name`
///
/// NOTE: This does not support usernames with discriminators, in those cases it's recommended to just
/// mention the member.
#[command]
#[required_permissions(KICK_MEMBERS)]
#[min_args(1)]
#[only_in("guilds")]
async fn kick(mut ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let member = parse_member(&mut ctx, &msg, args).await;
    match member {
        Ok(m) => {
            m.kick(&ctx).await?;
            msg.reply(&ctx, format!("Successfully kicked member `{}#{}`", m.user.read().await.name, m.user.read().await.discriminator)).await?;
        },
        Err(why) => {msg.reply(&ctx, why.to_string()).await?;},
    }

    Ok(())
}

/// Bans the specified member
/// Usage: `.ban @user` or `.ban user name`
///
/// NOTE: This does not support usernames with discriminators, in those cases it's recommended to just
/// mention the member.
#[command]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
#[only_in("guilds")]
async fn ban(mut ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let member = parse_member(&mut ctx, &msg, args).await;
    match member {
        Ok(m) => {
            m.ban(&ctx, &1).await?;
            msg.reply(&ctx, format!("Successfully banned member `{}#{}`", m.user.read().await.name, m.user.read().await.discriminator)).await?;
        },
        Err(why) => {msg.reply(&ctx, why.to_string()).await?;},
    }

    Ok(())
}

/// Deletes X number of messages from the current channel.
/// If the messages are older than 2 weeks, due to api limitations, they will not get deleted.
///
/// Usage: `.clear 20`
#[command]
#[required_permissions(MANAGE_MESSAGES)]
#[num_args(1)]
#[only_in("guilds")]
#[aliases(purge)]
async fn clear(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = args.single::<u64>();
    match num {
        Err(_) => {msg.channel_id.say(&ctx, "The value provided was not a valid number").await?;},
        Ok(n) => {
            let channel = &msg.channel(&ctx).await.unwrap().guild().unwrap();

            let messages = &channel.read().await.messages(&ctx, |r| r.before(&msg.id).limit(n)).await?;
            let messages_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            channel.read().await.delete_messages(&ctx, messages_ids).await?;

            msg.channel_id.say(&ctx, format!("Successfully deleted `{}` message", n)).await?;
        }
    }
    Ok(())
}
