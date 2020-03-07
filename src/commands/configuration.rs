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
};

/// Configures the bot for the channel it was invoked on.
///
/// Current configurable aspects:
/// `annoy` Toggles the annoying features on or off.
#[command]
#[required_permissions(MANAGE_CHANNELS)]
fn channel(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.message().starts_with("annoy") {
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
    }

    Ok(())
}
