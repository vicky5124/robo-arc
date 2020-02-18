use crate::ShardManagerContainer;
use serenity::{
    prelude::Context,
    model::{
        channel::Message,
        Permissions,
    },
    client::bridge::gateway::ShardId,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};


#[command] // Sets up a command
#[aliases("pong", "latency")] // Sets up aliases to that command.
#[description = "Sends the latency of the bot to the shards."] // Sets a description to be used for the help command. You can also use docstrings.

// All command functions must take a Context and Message type parameters.
// Optionally they may also take an Args type parameter for command arguments.
// They must also return CommandResult.
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read();
    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

            return Ok(());
        },
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            let _ = msg.reply(&ctx,  "No shard found");

            return Ok(());
        },
    };
   
    let latency;
    match runner.latency {
        Some(ms) => latency = format!("{:?}", ms),
        _ => latency = String::new(),
    }
    msg.reply(&ctx, format!("Ping? {}", latency))?;

    Ok(())
}

/// This command just sends an invite of the bot with the required permissions.
#[command]
fn invite(ctx: &mut Context, msg: &Message) -> CommandResult {
    // Sets up the permissions
    let mut permissions = Permissions::empty();
    permissions.set(Permissions::KICK_MEMBERS, true);
    permissions.set(Permissions::BAN_MEMBERS, true);
    permissions.set(Permissions::MANAGE_CHANNELS, true);
    permissions.set(Permissions::ADD_REACTIONS, true);
    permissions.set(Permissions::VIEW_AUDIT_LOG, true);
    permissions.set(Permissions::READ_MESSAGES, true);
    permissions.set(Permissions::SEND_MESSAGES, true);
    permissions.set(Permissions::MANAGE_MESSAGES, true);
    permissions.set(Permissions::EMBED_LINKS, true);
    permissions.set(Permissions::ATTACH_FILES, true);
    permissions.set(Permissions::READ_MESSAGE_HISTORY, true);
    permissions.set(Permissions::USE_EXTERNAL_EMOJIS, true);
    permissions.set(Permissions::CONNECT, true);
    permissions.set(Permissions::SPEAK, true);
    permissions.set(Permissions::MOVE_MEMBERS, true);
    permissions.set(Permissions::MANAGE_ROLES, true);
    permissions.set(Permissions::MANAGE_WEBHOOKS, true);

    // Creates the invite link for the bot with the permissions specified earlier.
    // Error handling in rust is so nice.
    let url = match ctx.cache.read().user.invite_url(&ctx, permissions) {
        Ok(v) => v,
        Err(why) => {
            println!("Error creating invite url: {:?}", why);

            return Ok(()); // Prematurely finish the command function.
        }
    };
    
    // Sends a DM to the author of the message with the invite link.
    //let _ = msg.author.direct_message(&ctx, |m| {
    //    m.content(format!("My invite link: <{}>\nCurrently private only, while the bot is in developement.", url))
    //});
    let _ = msg.channel_id.send_message(&ctx, |m| {
        m.embed( |e| {
            e.title("Invite Link");
            e.url(url);
            e.description("Keep in mind, this bot is still in pure developement, so not all of this mentioned features are implemented.\n\n__**Reason for each permission**__");
            e.fields(vec![
                     ("Move Members", "To automatically move members to the current music room (as long as there's people already listening there).", true),
                     ("Attach Files", "For some of the booru commands.\nFor an automatic text file to be sent when a message is too long.", true),
                     ("Read Messages", "So the bot can read the messages to know when a command was invoked and such.", true),
                     ("Manage Messages", "Be able to clear reactions of timed out paginations.\nClear moderation command.", true),
                     ("Manage Webhooks", "For all the commands that can be ran on a schedule, so it's more efficient.", true),
                     ("Manage Channels", "To have native access to see and speak on every channel on the server.\nTo avoid slowmode.", true),
                     ("Manage Roles", "Be able to give a stream notification role.\nMute moderation command", true),
                     ("Read Message History", "This is a required permission for every paginated command.", true),
                     ("Use External Emojis", "For all the commands that use emojis for better emphasis.", true),
                     ("View Audit Log", "To be able to have a more feature rich logging to a channel.", true),
                     ("Add Reactions", "To be able to add reactions for all the paginated commands.", true),
                     ("Send Messages", "So the bot can send the messages it needs to send.", true),
                     ("Speak", "To be able to play music on that voice channel.", true),
                     ("Embed Links", "For the tags to be able to embed images.", true),
                     ("Connect", "To be able to connect to a voice channel.", true),
                     ("Kick Members", "Kick/GhostBan moderation command.", true),
                     ("Ban Members", "Ban moderation command.", true),
            ]);
            e
        });

        m
    });
    Ok(())
}

#[command]
#[owners_only] // to only allow the owner of the bot to use this command
#[min_args(3)] // Sets the minimum ammount of arguments the command requires to be ran. This is used to trigger the `NotEnoughArguments` error.
// Testing command, please ignore.
fn test(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    std::thread::sleep(std::time::Duration::from_secs(50));
    let x = args.single::<String>()?;
    let y = args.single::<i32>()?;
    let z = args.single::<i32>()?;
    
    let multiplied = y * z;
    msg.channel_id.say(&ctx, format!("{} nice: {}", x, multiplied))?;
    let f = vec![123; 1000];
    msg.channel_id.say(&ctx, format!("{:?}", &f))?;

    Ok(())
}

#[command]
fn source(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx, "<https://gitlab.com/nitsuga5124/robo-arc/>")?;
    Ok(())
}
