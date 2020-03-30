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

fn parse_member(ctx: &mut Context, msg: &Message, args: Args) -> Result<Member, String> {
    let member_name = args.message();
    let mut members = Vec::new();

    if member_name.starts_with("<@") && member_name.ends_with(">") {
        let re = Regex::new("[<@!>]").unwrap();
        let member_id = re.replace_all(member_name, "").into_owned();
        let member = &msg.guild_id.unwrap().member(&ctx, UserId(member_id.parse::<u64>().unwrap()));

        match member {
            Ok(m) => Ok(m.to_owned()),
            Err(why) => Err(why.to_string()),
        }
    } else {
        let guild = &msg.guild(&ctx).unwrap();
        let rguild = &guild.read();

        for (_,m) in &rguild.members {
            if m.display_name() == std::borrow::Cow::Borrowed(member_name) ||
                m.user.read().name == member_name.to_string()
            {
                members.push(m);
            }
        }

        if &members.len() == &0 {
            let similar_members = &rguild.members_containing(&member_name, false, false);
            let mut members_string = similar_members.iter().map(|i| format!("`{}`|", i.display_name())).collect::<String>();

            let message = {
                if members_string == "".to_string() {
                    format!("No member named '{}' was found.", member_name)
                } else {
                    members_string.pop();
                    format!("No member named '{}' was found.\nDid you mean: {}", member_name, members_string)
                }
            };
            Err(message)
        } else if &members.len() == &1 {
            Ok(members[0].to_owned())
        } else {
            let members_string = &mut members.iter().map(|i| format!("`{}#{}`|", i.user.read().name, i.user.read().discriminator)).collect::<String>();
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
fn kick(mut ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let member = parse_member(&mut ctx, &msg, args);
    match member {
        Ok(m) => {
            m.kick(&ctx)?;
            &msg.reply(&ctx, format!("Successfully kicked member `{}#{}`", m.user.read().name, m.user.read().discriminator))?;
        },
        Err(why) => {&msg.reply(&ctx, why.to_string())?;},
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
fn ban(mut ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let member = parse_member(&mut ctx, &msg, args);
    match member {
        Ok(m) => {
            m.ban(&ctx, &1)?;
            &msg.reply(&ctx, format!("Successfully banned member `{}#{}`", m.user.read().name, m.user.read().discriminator))?;
        },
        Err(why) => {&msg.reply(&ctx, why.to_string())?;},
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
fn clear(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = args.single::<u64>();
    match num {
        Err(_) => {&msg.channel_id.say(&ctx, "The value provided was not a valid number")?;},
        Ok(n) => {
            let channel = &msg.channel(&ctx).unwrap().guild().unwrap();

            let messages = &channel.read().messages(&ctx, |r| r.before(&msg.id).limit(n))?;
            let messages_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            channel.read().delete_messages(&ctx, messages_ids)?;

            &msg.channel_id.say(&ctx, format!("Successfully deleted `{}` message", n))?;
        }
    }
    Ok(())
}
