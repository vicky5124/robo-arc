use crate::{
    DatabaseConnection,
    AnnoyedChannels,
};
use std::{
    sync::Arc,
    collections::HashSet,
};
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

fn set_best_tags(sex: &str, ctx: &mut Context, msg: &Message, mut tags: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = {
        let rdata = ctx.data.read();
        Arc::clone(rdata.get::<DatabaseConnection>().expect("Could not find a database connection."))
    };
    let user_id = *&msg.author.id.0 as i64;

    let data = {
        let mut client = client.write();
        client.query("SELECT best_boy, best_girl FROM best_bg WHERE user_id = $1", &[&user_id])?
    };

    if data.is_empty() {
        if sex == "boy" {
            // insert +1boy
            tags += " 1boy";

            let mut client = client.write();
            client.execute(
                "INSERT INTO best_bg (best_boy, user_id) VALUES ($1, $2)",
                &[&tags, &user_id]
            )?;
            msg.reply(&ctx, format!("Successfully set your husbando to `{}`", &tags))?;
        } else if sex == "girl" {
            // insert +1girl
            tags += " 1girl";

            let mut client = client.write();
            client.execute(
                "INSERT INTO best_bg (best_girl, user_id) VALUES ($1, $2)",
                &[&tags, &user_id]
            )?;
            msg.reply(&ctx, format!("Successfully set your waifu to `{}`", &tags))?;
        }
    } else {
        if sex == "boy" {
            // update +1boy
            tags += " 1boy";

            let mut client = client.write();
            client.execute(
                "UPDATE best_bg SET best_boy = $1 WHERE user_id = $2",
                &[&tags, &user_id]
            )?;
            msg.reply(&ctx, format!("You successfully broke up with your old husbando, now your husbando is `{}`", &tags))?;
        } else if sex == "girl" {
            // update +1girl
            tags += " 1girl";

            let mut client = client.write();
            client.execute(
                "UPDATE best_bg SET best_girl = $1 WHERE user_id = $2",
                &[&tags, &user_id]
            )?;
            msg.reply(&ctx, format!("You successfully broke up with your old waifu, now your waifu is `{}`", &tags))?;
        }
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
fn user(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[aliases(husbando, husband)]
fn best_boy(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("boy", ctx, msg, args.message().to_string())?)
}

#[command]
#[aliases(waifu, wife)]
fn best_girl(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(set_best_tags("girl", ctx, msg, args.message().to_string())?)
}

#[command]
fn booru(ctx: &mut Context, msg: &Message, raw_args: Args) -> CommandResult {
    let booru = raw_args.message().to_lowercase();
    let client = {
        let rdata = ctx.data.read();
        Arc::clone(rdata.get::<DatabaseConnection>().expect("Could not find a database connection."))
    };

    let user_id = *&msg.author.id.0 as i64;

    let data = {
        let mut client = client.write();
        client.query("SELECT best_boy, best_girl FROM best_bg WHERE user_id = $1", &[&user_id])?
    };

    if data.is_empty() {
        if booru.as_str() == "" {
            msg.reply(&ctx, "Please, specify the booru to set as your default.")?;
            return Ok(());
        }
        let mut client = client.write();
        client.execute(
            "INSERT INTO booru (booru, user_id) VALUES ($1, $2)",
            &[&booru, &user_id]
        )?;

        msg.reply(&ctx, format!("Successfully set your main booru to `{}`", &booru))?;
    } else {
        if booru.as_str() == "" {return Ok(());}

        let mut client = client.write();
        client.execute(
            "UPDATE best_bg SET booru = $1 WHERE user_id = $2",
            &[&booru, &user_id]
        )?;

        msg.reply(&ctx, format!("Successfully edited your main booru to `{}`", &booru))?;
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
#[sub_commands(annoy)]
fn channel(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

/// Toggles the annoying features on or off.
#[command]
fn annoy(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let client = {
        let rdata = ctx.data.read();
        Arc::clone(rdata.get::<DatabaseConnection>().expect("Could not find a database connection."))
    };
    let channel_id = *&msg.channel_id.0 as i64;

    let data = {
        let mut client = client.write();
        client.query("SELECT channel_id FROM annoyed_channels WHERE channel_id = $1", &[&channel_id])?
    };

    if !data.is_empty() {
        for row in data {
            if row.get::<_, i64>(0) == channel_id {
                {
                    let mut client = client.write();
                    client.execute(
                        "DELETE FROM annoyed_channels WHERE channel_id IN ($1)",
                        &[&channel_id]
                    )?;
                }

                msg.channel_id.say(&ctx, format!("Successfully removed `{}` from the list of channels that allows the bot to do annoying features.", msg.channel_id.name(&ctx).unwrap()))?;
            }
        }

    } else {
        {
            let mut client = client.write();
            client.execute(
                "INSERT INTO annoyed_channels (channel_id) VALUES ($1)",
                &[&channel_id]
            )?;
        }

        msg.channel_id.say(&ctx, format!("Successfully added `{}` to the list of channels that allows the bot to do annoying features.", msg.channel_id.name(&ctx).unwrap()))?;
    }

    {
        let mut db_client = client.write();
        let raw_annoyed_channels = {
            db_client.query("SELECT channel_id from annoyed_channels", &[])?
        };
        let mut annoyed_channels = HashSet::new();

        for row in raw_annoyed_channels {
            annoyed_channels.insert(row.get::<_, i64>(0) as u64);
        } 
        {
            let mut data = ctx.data.write();
            data.insert::<AnnoyedChannels>(annoyed_channels);
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
fn guild(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[min_args(1)]
fn prefix(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() < 1 {
        &msg.reply(&ctx, "Invalid prefix was given")?;
        return Ok(());
    }
    let prefix = args.message();

    // change prefix on db 
    let client = {
        let rdata = ctx.data.read();
        Arc::clone(rdata.get::<DatabaseConnection>().expect("Could not find a database connection."))
    };
    let guild_id = msg.guild_id.unwrap().0 as i64;

    let data = {
        let mut client = client.write();
        client.query("SELECT prefix FROM prefixes WHERE guild_id = $1", &[&guild_id])?
    };

    if data.is_empty() {
        let mut client = client.write();
        client.execute(
            "INSERT INTO prefixes (guild_id, prefix) VALUES ($1, $2)",
            &[&guild_id, &prefix]
        )?;
    } else {
        let mut client = client.write();
        client.execute(
            "UPDATE prefixes SET prefix = $2 WHERE guild_id = $1",
            &[&guild_id, &prefix]
        )?;
    }

    let content_safe_options = ContentSafeOptions::default();
    let bad_success_message = format!("Successfully changed your prefix to `{}`", prefix);
    let success_message = content_safe(&ctx, bad_success_message, &content_safe_options);
    &msg.reply(&ctx, success_message)?;
    Ok(())
}
