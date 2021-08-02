use crate::commands::booru::get_booru;
use crate::commands::music::_join;
use crate::utils::basic_functions::capitalize_first;
use crate::{BooruCommands, BooruList, DatabasePool};

use serenity::{
    framework::standard::{macros::hook, Args, CommandResult, Delimiter, DispatchError, Reason},
    model::prelude::*,
    prelude::*,
};

// Defining a structure to deserialize "boorus.json" into
// Debug is so it can be formatted with {:?}
// Default is so the values can be defaulted.
// Clone is so it can be cloned. (`Booru.clone()`)
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Booru {
    pub names: Vec<String>, // Default Vec<String>[String::new()]
    pub url: String,        // String::new()
    pub typ: u8,            // 0
    pub post_url: String,
}

// This is for errors that happen before command execution.
#[hook]
pub async fn on_dispatch_error(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    _command_name: &str,
) {
    match error {
        // Notify the user if the reason of the command failing to execute was because of
        // inssufficient arguments.
        DispatchError::NotEnoughArguments { min, given } => {
            let s = {
                if given == 0 && min == 1 {
                    "I need an argument to run this command".to_string()
                } else if given == 0 {
                    format!("I need atleast {} arguments to run this command", min)
                } else {
                    format!(
                        "I need {} arguments to run this command, but i was only given {}.",
                        min, given
                    )
                }
            };
            // Send the message, but supress any errors that may occur.
            let _ = msg.channel_id.say(ctx, s).await;
        }
        //DispatchError::IgnoredBot {} => {
        //    return;
        //},
        DispatchError::CheckFailed(_, reason) => {
            if let Reason::User(r) = reason {
                let _ = msg.channel_id.say(ctx, r).await;
            }
        }
        DispatchError::Ratelimited(x) => {
            let _ = msg
                .reply(
                    ctx,
                    format!(
                        "You can't run this command for {} more seconds.",
                        x.as_secs()
                    ),
                )
                .await;
        }
        // eprint prints to stderr rather than stdout.
        _ => {
            error!("Unhandled dispatch error: {:?}", error);
            eprintln!("An unhandled dispatch error has occurred:");
            eprintln!("{:?}", error);
        }
    }
}

// This function executes before a command is called.
#[hook]
pub async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    if let Some(guild_id) = msg.guild_id {
        let pool = {
            let data_read = ctx.data.read().await;
            data_read.get::<DatabasePool>().unwrap().clone()
        };

        let disallowed_commands = sqlx::query!(
            "SELECT disallowed_commands FROM prefixes WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        if let Some(x) = disallowed_commands {
            if let Some(disallowed_commands) = x.disallowed_commands {
                if disallowed_commands.contains(&cmd_name.to_string()) {
                    let _ = msg
                        .reply(
                            ctx,
                            "This command has been disabled by an administrtor of this guild.",
                        )
                        .await;
                    return false;
                }
            }
        }

        if cmd_name == "play" || cmd_name == "play_playlist" {
            let manager = songbird::get(ctx).await.unwrap().clone();

            if manager.get(guild_id).is_none() {
                if let Err(why) = _join(ctx, msg).await {
                    error!("While running command: {}", cmd_name);
                    error!("{:?}", why);
                    return false;
                }
            }
        }
    }

    info!("Running command: {}", &cmd_name);
    debug!("Command Message Struct: {:?}", &msg);

    true
}

// This function executes every time a command finishes executing.
// It's used here to handle errors that happen in the middle of the command.
#[hook]
pub async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: CommandResult) {
    // error is the command result.
    // inform the user about an error when it happens.
    if let Err(why) = &error {
        error!("Error while running command {}", &cmd_name);
        error!("{:?}", &error);

        //let err = why.0.to_string();
        if msg.channel_id.say(ctx, &why).await.is_err() {
            error!(
                "Unable to send messages on channel id {}",
                &msg.channel_id.0
            );
        };
    }
}

// Small error event that triggers when a command doesn't exist.
#[hook]
pub async fn unrecognised_command(ctx: &Context, msg: &Message, command_name: &str) {
    let (pool, commands, boorus) = {
        let data_read = ctx.data.read().await;

        let pool = data_read.get::<DatabasePool>().unwrap();
        let commands = data_read.get::<BooruCommands>().unwrap();
        let boorus = data_read.get::<BooruList>().unwrap();

        (pool.clone(), commands.clone(), boorus.clone())
    };

    if let Some(guild_id) = msg.guild_id {
        let disallowed_commands = sqlx::query!(
            "SELECT disallowed_commands FROM prefixes WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        if let Some(x) = disallowed_commands {
            if let Some(disallowed_commands) = x.disallowed_commands {
                if disallowed_commands.contains(&"booru_command".to_string()) {
                    let _ = msg
                        .reply(
                            ctx,
                            "This command has been disabled by an administrtor of this guild.",
                        )
                        .await;
                    return;
                }
            }
        }
    }

    if commands.contains(command_name) {
        let booru: Booru = {
            let mut x = Booru::default();
            for b in boorus.iter() {
                if b.names.contains(&command_name.to_string()) {
                    x = b.clone();
                }
            }
            x
        };

        info!("Running command: {}", &booru.names[0]);
        debug!("Message: {}", &msg.content);

        let lower_content = msg.content.to_lowercase();
        let parameters = lower_content.split(&command_name).nth(1).unwrap();
        let params = Args::new(parameters, &[Delimiter::Single(' ')]);

        let booru_result = get_booru(ctx, msg, &booru, params).await;
        if let Err(why) = booru_result {
            // Handle any error that may occur.
            let why = why.to_string();
            let reason = format!(
                "There was an error executing the command {}: {}",
                &booru.names[0],
                capitalize_first(&why)
            );
            error!("{}", reason);
            let _ = msg
                .channel_id
                .say(ctx, format!("There was an error running {}", command_name))
                .await;
        }
    }
}

#[hook]
pub async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    // Custom per guild prefixes.
    //info!("Dynamic prefix call.");
    // obtain the guild id of the command message.
    let guild_id = &msg.guild_id;

    let p;

    // If the command was invoked on a guild
    if let Some(id) = guild_id {
        // Get the real guild id, and the i64 type becaues that's what postgre uses.
        let gid = id.0 as i64;
        let pool = {
            // Open the context data lock in read mode.
            let data = ctx.data.read().await;
            // it's safe to clone PgPool
            data.get::<DatabasePool>().unwrap().clone()
        };

        // Obtain the database connection for the data.
        // Obtain the configured prefix from the database
        match sqlx::query!("SELECT prefix FROM prefixes WHERE guild_id = $1", gid)
            .fetch_optional(&pool)
            .await
        {
            Err(why) => {
                error!("Could not query database: {}", why);
                p = ".".to_string();
            }
            Ok(db_prefix) => {
                p = if let Some(result) = db_prefix {
                    result.prefix.unwrap_or_else(|| ".".to_string())
                } else {
                    ".".to_string()
                };
            }
        }

    // If the command was invoked on a dm
    } else {
        p = ".".to_string();
    };

    // dynamic_prefix() needs an Option<String>
    Some(p)
}
