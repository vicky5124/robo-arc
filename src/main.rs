/// This is a discord bot made with `serenity.rs` as a Rust learning project.
/// If you see a lot of different ways to do the same thing, specially with error handling,
/// this is indentional, as it helps me remember the concepts that rust provides, so they can be
/// used in the future for whatever they could be needed.
///
/// This is lisenced with the WTFPL, aka you can do whatever the freak you want to with it.

mod cogs; // Load the cogs module
use cogs::booru::*; // Import everything from the cogs module.

use std::{
    env,
    collections::HashSet,
    sync::Arc,
};

use serenity::{
    client::{
        Client,
        bridge::gateway::{
            ShardId,
            ShardManager,
        },
    },
    model::{
        channel::{
            Message,
            Reaction,
            ReactionType,
        },
        gateway::{
            Ready,
            Activity,
        },
        Permissions,
        user::OnlineStatus,
        id::UserId,
    },
    prelude::{
        EventHandler,
        Context,
        Mutex,
        TypeMapKey,
    },
    framework::standard::{
        Args,
        CommandResult,
        CommandGroup,
        DispatchError,
        HelpOptions,
        help_commands,
        StandardFramework,
        macros::{
            command,
            group,
            help,
        },
    },
};



struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}


// The basic commands group is being defined here.
// this group includes the commands ping and test, nothing really special.
#[group("The Basics")]
#[description = "All the basic commands that every bot should have."]
#[commands(ping, test)]
struct TheBasics;

// The NSFW command group.
// the list of commands will get added later, as soon as the commands get made.
// this commands will eventually only work on DM or NSFW Channels with a custom check.
#[group("NSFW")]
#[description = "All the NSFW/BSFW related commands."]
#[commands(test)]
struct NSFW;

// The Booru command group.
// This group will contain every single command from every booru that gets implemented.
// As you can see on the last line, the description also supports urk markdown.
#[group("Boorus")]
#[description = "All the booru related commands.\n\
Available parameters:\n\
`-x` Explicit\n\
`-q` Questionable\n\
`-n` Non Safe (Random between E or Q)\n\
`-r` Any Rating\n\n\
Inspired by -GN's WaifuBot ([source](https://github.com/isakvik/waifubot/))"]
#[commands(safebooru)]
struct Boorus;

// This is a custom help command.
// Each line has the explaination that is required.
#[help]
// This is the basic help message
// We use \ at the end of the line to easily allow for newlines visually on the code.
#[individual_command_tip = "Hello!\n\
If youd like to get extra information about a specific command, just pass it as an argument.\n\
You can also react with 🚫 on any message sent by the bot to delete it.\n"]
// This is the text that gets displayed when a given parameter was not found for information.
#[command_not_found_text = "Could not find: `{}`."]
// This is the level of similarities between the given argument and possible other arguments.
// This is used to give suggestions in case of a typo.
#[max_levenshtein_distance(3)]
// This makes it so specific sections don't get showed to the user if they don't have the
// permission to use them.
#[lacking_permissions = "Hide"]
// In the case of just lacking a role to use whatever is necessary, nothing will happen.
#[lacking_role = "Nothing"]
// In the case of being on the wrong channel type (either DM for Guild only commands or vicecersa)
// the command will be ~~striked~~
#[wrong_channel = "Strike"]
fn my_help(
    ctx: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, help_options, groups, owners)
}



struct Handler; // Defines the handler to be used for events.

impl EventHandler for Handler {
    /// on_ready event on d.py
    /// This function triggers when the bot is ready.
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is ready to rock!", ready.user.name);
        
        // Changes the presence of the bot to "Listening to C++ cry a Rusted death."
        ctx.set_presence(
            Some(Activity::listening("C++ cry a Rusted death.")),
            OnlineStatus::Online
        );
    }

    /// on_message even on d.py
    /// This function triggers every time a message is sent.
    fn message(&self, ctx: Context, msg: Message) {
        // Ignores itself.
        if &msg.author.id.0 == ctx.cache.read().user.id.as_u64() {
            return;
        }
        // This is gonna be annoying lol
        if msg.content == "no u" {
            let _ = msg.reply(&ctx, "no u"); // reply pings the user
            //let _ = msg.channel_id.say(&ctx, "no u"); // say just send the message
        }

        // This is an alternative way to make commands that doesn't involve the Command Framework.
        // this is not recommended as it would block the event thread, which Framework Commands
        // don't do.
        // This command just sends an invite of the bot witout permissions.
        if msg.content == ".invite" {
            // Creates the invite link for the bot without permissions.
            // Error handling in rust is so nice.
            let url = match ctx.cache.read().user.invite_url(&ctx, Permissions::empty()) {
                Ok(v) => v,
                Err(why) => {
                    println!("Error creating invite url: {:?}", why);

                    return; // Prematurely finish the function.
                },
            };
            
            // Sends a DM to the author of the message with the invite link.
            let _ = msg.author.direct_message(&ctx, |m| {
                m.content(format!("My invite link: <{}>\nCurrently private only, while the bot is in developement.", url))
            });
            
            /*
            // `dm` is a Message Type object
            match dm {
                Ok(_) => {
                    let _ = msg.react(&ctx, '👌');
                },
                Err(why) => {
                    println!("Err sending help: {:?}", why);

                    let _ = msg.reply(&ctx, "There was an error DMing you help.");
                },
            };
            */
        }
    }

    /// on_raw_reaction_add on d.py
    /// This function triggers every time a reaction gets added to a message.
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // Ignores all reactions that come from the bot itself.
        if &add_reaction.user_id.0 == ctx.cache.read().user.id.as_u64() {
            return;
        }

        match add_reaction.emoji {
            // Matches custom emojis.
            ReactionType::Custom{id, ..} => {
                // If the emote is the GW version of slof, React back.
                // This also shows a couple ways to do error handlig.
                if id.0 == 375459870524047361 {
                    let msg = ctx.http.as_ref().get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
                        .expect("Error while obtaining message");

                    let reaction = msg.react(&ctx, add_reaction.emoji);
                    if let Err(why) = reaction {
                        println!("There was an error adding a reaction: {}", why)
                    }

                    let _ = msg.channel_id.say(&ctx, format!("<@{}>: qt", add_reaction.user_id.0));
                }
            },
            // Matches unicode emojis.
            ReactionType::Unicode(s) => {
                // This will not be kept here for long, as i see it being very annoying eventually.
                if s == "🤔" {
                    let msg = ctx.http.as_ref().get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
                        .expect("Error while obtaining message");
                    let _ = msg.channel_id.say(&ctx, format!("<@{}>: What ya thinking so much about",
                                                             add_reaction.user_id.0));
                // This makes every message sent by the bot get deleted if 🚫 is on the reactions.
                // aka If you react with 🚫 on any message sent by the bot, it will get deleted.
                // This is helpful for antispam and anti illegal content measures.
                } else if s == "🚫" {
                    let msg = ctx.http.as_ref().get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
                        .expect("Error while obtaining message");
                    if msg.author.id == ctx.cache.read().user.id {
                        let _ = msg.delete(&ctx);
                    }
                }
            },
            // Ignore the rest of the cases.
            _ => (), // complete code
            //_ => {}, // incomplete code / may be longer in the future
        }
    }
}



/// The main function!
/// Here's where everything starts.
/// This main function is a little special, as it returns Result, which allows ? to be used for
/// error handling.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Defines a client with the token obtained from the DEV_DISCORD_TOKEN environmental variable.
    // This also starts up the Event Handler structure defined earlier.
    let mut client = Client::new(
        &env::var("DEV_DISCORD_TOKEN")?,
        Handler)?;

    // Closure to define shard data.
    // Don't ask me how this works, as i don't know either, yet.
    {
        let mut data = client.data.write();
        //data.insert::<CommandCounter>(HashMap::Default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }
    
    // Obtains and defines the owner/owners of the Bot Application
    // and the bot id. 
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Time to configure the Command Framework!
    // This is what allows for easier and faster commaands.
    client.with_framework(StandardFramework::new() // Create a new framework
        .configure(|c| c
            .prefixes(vec![".", "n!", "]"]) // Add a list of prefixes to be used to invoke commands.
            .on_mention(Some(bot_id)) // Add a bot mention as a prefix.
            .with_whitespace(true) // Allow a whitespace between the prefix and the command name.
            .owners(owners) // Defines the owners, this can be later used to make owner specific commands.
        )

        // This is for errors that happen before command execution.
        .on_dispatch_error(|ctx, msg, error| {
            println!("{:?}", error);
            match error {
                // Notify the user if the reason of the command failing to execute was because of
                // inssufficient arguments.
                DispatchError::NotEnoughArguments { min, given } => {
                    let s = format!("I need {} arguments to run this command, but i was only given {}.", min, given);

                    let _ = msg.channel_id.say(&ctx, s);
                },
                _ => println!("Unhandled dispatch error."),
            }
        })
        
        // This lambda/closure function executes every time a command finishes executing.
        // It's used here to handle errors that happen in the middle of the command.
        .after(|ctx, msg, _cmd_name, error| {
            if let Err(why) = error {
                let err = format!("{}", why.0);
                let _ = msg.channel_id.say(&ctx, &err);
            }
        })

        // Small error event that triggers when a command doesn't exist.
        .unrecognised_command(|_, _, unknown_command_name| {
            println!("Could not find command named '{}'", unknown_command_name);
        })
        .group(&THEBASICS_GROUP) // Load `The Basics` command group
        .group(&NSFW_GROUP) // Load `NSFW` command group
        .group(&BOORUS_GROUP) // Load `Boorus` command group
        .help(&MY_HELP) // Load the custom help.
    );


    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}



#[command] // Sets up a command
#[aliases("pong", "latency")] // Sets up aliases to that command.
#[description = "Sends the latency of the bot to the shards."] // Sets a description to be used for the help command.

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
    msg.reply(&ctx, format!("Pong! {}", latency))?;

    Ok(())
}



#[command]
#[min_args(3)] // Sets the minimum ammount of arguments the command requires to be ran. This is used to trigger the `NotEnoughArguments` error.
// Testing command, please ignore.
fn test(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let x = args.single::<String>()?;
    let y = args.single::<i32>()?;
    let z = args.single::<i32>()?;
    
    let multiplied = y * z;
    msg.channel_id.say(&ctx, format!("{} nice: {}", x, multiplied))?;
    let f = vec![123; 1000];
    msg.channel_id.say(&ctx, format!("{:?}", &f))?;

    Ok(())
}
