use crate::{
    ShardManagerContainer,
    DatabaseConnection,
    utils::database::get_database,
};
use std::{
    fs::File,
    io::prelude::*,
    sync::Arc,
    process::{
        Command,
        Stdio,
        id,
    },
};
use serenity::{
    prelude::{
        Context,
        RwLock,
    },
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
use num_format::{
    Locale,
    ToFormattedString,
};
use toml::Value;


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
        Some(ms) => latency = format!("{:.2}ms", ms.as_micros() as f32 / 1000.0),
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
    permissions.set(Permissions::MENTION_EVERYONE, true);

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
                     ("Manage Channels", "Be able to mute members on the channel without having to create a role for it.", true),
                     ("Manage Webhooks", "For all the commands that can be ran on a schedule, so it's more efficient.", true),
                     ("Manage Roles", "Be able to give a stream notification role.\nMute moderation command.", true),
                     ("Read Message History", "This is a required permission for every paginated command.", true),
                     ("Use External Emojis", "For all the commands that use emojis for better emphasis.", true),
                     ("View Audit Log", "To be able to have a more feature rich logging to a channel.", true),
                     ("Add Reactions", "To be able to add reactions for all the paginated commands.", true),
                     ("Mention Everyone", "To be able to mention the livestream notification role.", true),
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
#[help_available(false)] // makes it not show up on the help menu
#[owners_only] // to only allow the owner of the bot to use this command
#[min_args(3)] // Sets the minimum ammount of arguments the command requires to be ran. This is used to trigger the `NotEnoughArguments` error.
// Testing command, please ignore.
fn test(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    source(&mut ctx.clone(), &msg.clone(), args.clone())?;
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

/// Sends the source code url to the bot.
#[command]
fn source(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx, "<https://gitlab.com/nitsuga5124/robo-arc/>")?;
    Ok(())
}

/// Sends the current TO-DO list of the bot
#[command]
#[aliases(todo_list)]
fn todo(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx, "```prolog
TODO:

#Random/Fun
Dictionary (dictionary search)
Calculator (maths)
Encrypt/Decrypt (encrypts and decrypts a message)
Reminder (reminds you of a message after X time)

#Osu!
Score (posts the user score on the map id specified)
Profile (posts the user profile data)
Top (posts the top plays of the user)
MapPP (calculates pp of a map, like ezpp or tillerino)

#DDG
Search (searches term on duckduckgo)

#Twitch
\"Livestream notifications\"
Streamrole (gives the stream notification role set to the guild)
ChangeLiveMessage (changes the \"im live\" message)
ChangeNotLiveMessage (changes the \"im no longer live\" message)
ConfigureStream (configures stream notifications for the channel)

#Reddit
Subreddit (posts a random post from the subreddit specified)
User (posts a random post from the user specified)
Sub/User Bomb (posts 5 posts from the subreddit or user specified)

#Image Manipulation
Pride (prides the provided image, either bi or gay)

#Mod
Clear (add specific requieriments like \"only webhooks\")
PermaBan (permanently bans a user from the guild by not allowing the user to ever get back on (perma kick))
TempMute (mutes the user on the specific channel or all channels)
Logging (set a channel to log specific events)

#Tags
\"Basically the same as R. Danny, but with personal tags supported\"

# Boorus
\"Fix tag filter and the 2 boorus that don't work\"
Source (sends the source of an image, using iqdb and saucenao)
Exclude (excludes tags automatically from your search)
nHentai (nhentai reader and searcher)

# Music
\"Make a lavalink wrapper for serenity\"
```")?;
    Ok(())
}

/// Sends the current prefixes set to the server.
#[command]
#[aliases(prefix)]
fn prefixes(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx, "Current prefixes:\n`.` `arc!`")?;
    Ok(())
}

/// Sends information about the bot.
#[command]
#[aliases(info)]
fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    let pid = id().to_string();

    let mut full_mem = String::new();
    let mut reasonable_mem = String::new();

    let mut stdout = Command::new("pmap")
        .arg(&pid)
        //.arg(" | tail -n 1 | awk '/[0-9]K/{print $2}'")
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(pmap_out) = stdout.stdout.take() {
        let mut tail = Command::new("tail")
            .arg("-n 1")
            .stdin(pmap_out)
            .stdout(Stdio::piped())
            .spawn()?;
        stdout.wait()?;

        if let Some(tail_out) = tail.stdout.take() {
            let awk = Command::new("awk")
                .arg("/[0-9]K/{print $2}")
                .stdin(tail_out)
                .stdout(Stdio::piped())
                .spawn()?;
            let awk_stdout = awk.wait_with_output()?;
            tail.wait()?;
            full_mem = String::from_utf8(awk_stdout.stdout).unwrap();
            full_mem.pop();
            full_mem.pop();
        }
    }

    let mut stdout = Command::new("pmap")
        .arg(&pid)
        //.arg(" | tail -n 1 | awk '/[0-9]K/{print $2}'")
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(pmap_out) = stdout.stdout.take() {
        let mut head = Command::new("head")
            .arg("-n 2")
            .stdin(pmap_out)
            .stdout(Stdio::piped())
            .spawn()?;

        stdout.wait()?;

        if let Some(head_out) = head.stdout.take() {
            let mut tail = Command::new("tail")
                .arg("-n 1")
                .stdin(head_out)
                .stdout(Stdio::piped())
                .spawn()?;
            head.wait()?;

            if let Some(tail_out) = tail.stdout.take() {
                let awk = Command::new("awk")
                    .arg("/[0-9]K/{print $2}")
                    .stdin(tail_out)
                    .stdout(Stdio::piped())
                    .spawn()?;
                let awk_stdout = awk.wait_with_output()?;
                tail.wait()?;
                reasonable_mem = String::from_utf8(awk_stdout.stdout).unwrap();
                reasonable_mem.pop();
                reasonable_mem.pop();
            }
        }
    }
    let cache = &ctx.cache.read();
    let current_user = &ctx.http.get_current_user()?;
    let app_info = &ctx.http.get_current_application_info()?;

    let hoster_tag = &app_info.owner.tag();
    let hoster_id = &app_info.owner.id;

    let version = {
        let mut file = File::open("Cargo.toml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data = contents.parse::<Value>().unwrap();
        let version = data["package"]["version"].as_str().unwrap();
        version.to_string()
    };

    let bot_name = &current_user.name;
    let bot_icon = &current_user.avatar_url();

    let num_guilds = &cache.guilds.len();
    let num_shards = &cache.shard_count;
    let num_members = &cache.users.len();


    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title(format!("**{}** - Version: {}", bot_name, version));
            e.description("General Purpose Discord Bot made in [Rust](https://www.rust-lang.org/) using [serenity.rs](https://github.com/serenity-rs/serenity)\n\nHaving any issues? join the [Support Server](https://discord.gg/kH7z85n)\n\n[Top.GG](https://top.gg/bot/673680961535475712)");

            //e.field("Creator", "Tag: nitsuga5124#2207\nID: 182891574139682816", true);
            e.field("Hoster", format!("Tag: {}\nID: {}", hoster_tag, hoster_id), true);
            e.field("Memory usage", format!("Complete:\n`{} KB`\nBase:\n`{} KB`",
                                    &full_mem.parse::<u32>().expect("NaN").to_formatted_string(&Locale::en), &reasonable_mem.parse::<u32>().expect("NaN").to_formatted_string(&Locale::en)), true);
            e.field("Guild Data", format!("Guilds: {}\nUsers: {}\nShards: {}", num_guilds, num_members, num_shards), true);

            if let Some(x) = bot_icon {
                e.thumbnail(x);
            }
            e
        });
        m
    })?;

    Ok(())
}

/// Sends the bot changelog.
#[command]
fn changelog(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx, "<https://gitlab.com/nitsuga5124/robo-arc/-/blob/master/CHANGELOG.md>")?;
    Ok(())
}

#[command]
#[owners_only]
fn reload_db(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write();
    data.insert::<DatabaseConnection>(Arc::clone(&Arc::new(RwLock::new(get_database()?))));
    msg.channel_id.say(&ctx, "Ok.")?;
    Ok(())
}
