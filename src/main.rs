/// This is a discord bot made with `serenity.rs` as a Rust learning project.
/// If you see a lot of different ways to do the same thing, specially with error handling,
/// this is indentional, as it helps me remember the concepts that rust provides, so they can be
/// used in the future for whatever they could be needed.
///
/// This is lisenced with the copyleft license Mozilla Public License Version 2.0

mod utils; // Load the utils module
mod commands; // Load the commands module

// Import this 2 commands in specific with a different name
// as they interfere with the configuration commands that are also being imported.
use commands::booru::{
    BEST_BOY_COMMAND as BG_COMMAND,
    BEST_GIRL_COMMAND as BB_COMMAND,
};
use commands::meta::PREFIX_COMMAND as PREFIXES_COMMAND;

use commands::booru::*; // Import everything from the booru module.
use commands::sankaku::*; // Import everything from the sankaku booru module.
use commands::osu::*; // Import everything from the osu module.
use commands::meta::*; // Import everything from the meta module.
use commands::image_manipulation::*; // Import everything from the image manipulation module.
use commands::fun::*; // Import everything from the fun module.
use commands::moderation::*; // Import everything from the moderation module.
use commands::configuration::*; // Import everything from the configuration module.

use utils::database::get_database; // Obtain the get_database function from the utilities.
use utils::basic_functions::capitalize_first; // Obtain the capitalize_first function from the utilities.

use std::{
    collections::HashSet, // Low cost indexable lists.
    // For saving / reading files
    fs::File,
    io::prelude::*,

    // For having refferences between threads
    sync::Arc,
};

use tokio::sync::Mutex;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tracing_log::LogTracer;
use log;

use tokio_postgres::Client as PgClient; // PostgreSQL Client struct.
use toml::Value; // To parse the data of .toml files
use serde_json; // To parse the data of .json files (where's serde_toml smh)
use serde::Deserialize; // To deserialize data into structures

// Serenity! what make's the bot function. Discord API wrapper.
use serenity::{
    async_trait,
    utils::Colour, // To change the embed help color
    client::{
        Client, // To create a client that runs eveyrthing.
        bridge::gateway::ShardManager, // To manage shards, or in the case of this small bot, just to get the latency of it for ping.
    },
    http::Http,
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
        user::OnlineStatus,
        id::UserId,
    },
    prelude::{
        EventHandler,
        Context,
        TypeMapKey,
        RwLock,
    },
    framework::standard::{
        Args,
        Delimiter,
        CommandResult,
        CommandGroup,
        DispatchError,
        HelpOptions,
        help_commands,
        StandardFramework,
        macros::{
            group,
            help,
            hook,
        },
    },
};

// Defining a structure to deserialize "boorus.json" into
// Debug is so it can be formatted with {:?}
// Default is so the values can be defaulted.
// Clone is so it can be cloned. (`Booru.clone()`)
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Booru {
    names: Vec<String>, // Default Vec<String>[String::new()]
    url: String, // String::new()
    typ: u8, // 0
}

// Because "boorus.json" boorus key is a list of values.
#[derive(Debug, Deserialize)]
struct BooruRaw {
    boorus: Vec<Booru>,
}

// Defining the structures to be used for "global" data
// this data is not really global, it's just shared with Context.data
struct ShardManagerContainer; // Shard manager to use for the latency.
struct DatabaseConnection; // The connection to the database, because having multiple connections is a bad idea.
struct Tokens; // For the configuration found on "config.toml"
struct AnnoyedChannels; // This is a HashSet of all the channels the bot is allowed to be annoyed on.
struct BooruList; // This is a HashSet of all the boorus found on "boorus.json"
struct BooruCommands; // This is a HashSet of all the commands/aliases found on "boorus.json"

// Implementing a type for each structure
// This is made to make a Map<Struct, TypeValue>
impl TypeMapKey for ShardManagerContainer {
    // Mutex allows for the arc refference to be mutated
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for DatabaseConnection {
    // RwLock (aka Read Write Lock) makes the data only modifyable by 1 thread at a time
    // So you can only have the lock open with write a single use at a time.
    // You can have multiple reads, but you can't read as soon as the lock is opened for writing.
    type Value = Arc<RwLock<PgClient>>;
}

impl TypeMapKey for Tokens {
    // Value is not to be confused with the other values.
    // This Value is toml::Value
    type Value = Value;
}

impl TypeMapKey for AnnoyedChannels {
    type Value = HashSet<u64>;
}

impl TypeMapKey for BooruList {
    type Value = Vec<Booru>; 
}

impl TypeMapKey for BooruCommands {
    type Value = HashSet<String>; 
}



// The basic commands group is being defined here.
// this group includes the commands that basically every bot has, nothing really special.
#[group("Meta")]
#[description = "All the basic commands that basically every bot has."]
#[commands(ping, test, invite, source, todo, prefixes, about, changelog, reload_db)]
struct Meta;

// The SankakuComplex command group.
// This group contains commands for the variants Chan and Idol of the sankaku boorus.
#[group("Sankaku")]
#[help_available(false)] // So the group doesn't show up on the help command.
#[description = "All the NSFW/BSFW related commands."]
#[commands(idol, chan)]
struct Sankaku;

// The osu! command group.
// This group contains all the osu! related commands.
#[group("osu!")]
#[description = "All the osu! related commands"]
#[commands(configure_osu, recent)]
struct Osu;

// The Booru command group.
// This group will contain every single command from every booru that gets implemented.
// As you can see on the last line, the description also supports url markdown.
#[group("All Boorus")]
#[description = "All the booru related commands.\n\
Available parameters:
`-x` Explicit
`-q` Questionable
`-s` Safe
`-n` Non Safe (Random between E or Q)

Inspired by -GN's WaifuBot ([source](https://github.com/isakvik/waifubot/))"]
#[commands(booru_command, BB, BG)] // We imported BB_COMMAND and BG_COMMAND, but this macro automatically adds _COMMAND, so we don't put that.
struct AllBoorus;

// The Image Manipulation command group.
// This group contains all the commands that manipulate images.
#[group("Image Manipulation")]
#[description = "All the image manipulaiton based commands."]
#[commands(pride)]
struct ImageManipulation;

// The FUN command group.
// Where all the random commands go into lol
#[group("Fun")]
#[description = "All the random and fun commands."]
#[commands(qr, urban, translate, duck_duck_go, encrypt, decrypt)]
struct Fun;

// The moderation command group.
#[group("Moderation")]
#[description = "All the moderation related commands."]
#[commands(kick, ban, clear)]
struct Mod;

// The configuration command.
// Technically a group, but it only has a single command.
#[group("Configuration")]
#[description = "All the configuration related commands.
Basic usage:
`.config user VALUE DATA`
`.config guild VALUE DATA`
`.config channel VALUE DATA`"]
#[prefixes("config", "configure", "conf")]
#[commands(guild, channel, user)]
struct Configuration;

// This is a custom help command.
// Each line has the explaination that is required.
#[help]
// This is the basic help message
// We use \ at the end of the line to easily allow for newlines visually on the code.
#[individual_command_tip = "Hello!
If youd like to get more information about a specific command or group, you can just pass it as a command argument.
All the command examples through out the help will be shown using the default prefix `.`

You can react with 🚫 on *any* message sent by the bot to delete it.\n"]
// This is the text that gets displayed when a given parameter was not found for information.
#[command_not_found_text = "Could not find: `{}`."]
// This is the level of similarities between the given argument and possible other arguments.
// This is used to give suggestions in case of a typo.
#[max_levenshtein_distance(3)]
// This makes it so specific sections don't get showed to the user if they don't have the
// permission to use them.
#[lacking_permissions = "Hide"]
// In the case of just lacking a role to use whatever is necessary, nothing will happen when
// setting it to "Nothing", rn it just strikes the option.
#[lacking_role = "Strike"]
// In the case of being on the wrong channel type (either DM for Guild only commands or vicecersa)
// the command will be ~~striked~~
#[wrong_channel = "Strike"]
// This will change the text that appears on groups that have a custom prefix
#[group_prefix = "Prefix commands"]
async fn my_help(
    ctx: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let mut ho = help_options.clone();
    // Changeing the color of the embed sidebar, because the default one is ugly :P
    ho.embed_error_colour = Colour::from_rgb(255, 30, 30);
    ho.embed_success_colour= Colour::from_rgb(141, 91, 255);

    help_commands::with_embeds(ctx, msg, args, &ho, groups, owners).await
}



struct Handler; // Defines the handler to be used for events.

#[async_trait]
impl EventHandler for Handler {
    // on_ready event on d.py
    // This function triggers when the client is ready.
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Changes the presence of the bot to "Listening to ..."
        // https://docs.rs/serenity/0.8.0/serenity/model/gateway/struct.Activity.html#methods
        // for all the available activities.
        ctx.set_presence(
            Some(Activity::listening("the awaitening.")),
            OnlineStatus::Online
        ).await;

        println!("{} is ready to rock!", ready.user.name);
    }

    // on_message event on d.py
    // This function triggers every time a message is sent.
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignores itself.
        //if &msg.author.id.0 == ctx.cache.read().user.id.as_u64() {
        //    return;
        //}
        // Ignores bot accounts.
        if msg.author.bot {
            return;
        }

        // This is where i basically make a small command framework
        // this is to run the booru commands on the "boorus.json" file.
        // I shoul reimplement this using:
        // StandardFramework::before()
        // StandardFramework::unrecognised_command()

        // Read the global data lock
        let data_read = ctx.data.read().await;
        // Read the cache lock
        let cache = ctx.cache.read().await;
        // obtain the id of the guild.
        let guild_id = &msg.guild_id;
        // get the ID of the bot.
        let bot_id = cache.user.id.as_u64();

        // obtain the mention strings of the bot.
        // one with a space after the mention, another one without it.
        let bot_mention = format!("<@!{}>", bot_id);
        let bot_mention_spaced = format!("<@!{}> ", bot_id);
        let bot_mention_pure = format!("<@{}>", bot_id);
        let bot_mention_spaced_pure = format!("<@{}> ", bot_id);

        // Here's where i obtain the prefix used for the command.
        let prefix;

        // if the message starts with any of the 2 bot mentions
        // set the prefix to the mentions
        if msg.content.starts_with(bot_mention_spaced.as_str()) {
            prefix = bot_mention_spaced;
        } else if msg.content.starts_with(bot_mention_spaced_pure.as_str()) {
            prefix = bot_mention_spaced_pure;
        } else if msg.content.starts_with(bot_mention.as_str()) {
            prefix = bot_mention;
        } else if msg.content.starts_with(bot_mention_pure.as_str()) {
            prefix = bot_mention_pure;
        // else, obtain the custom prefix of the guild
        } else if let Some(id) = guild_id {
            // obtain the id of the guild as an i64, because the id is stored as a u64, which is
            // not compatible with the postgre datbase types.
            let gid = id.0 as i64;

            // Obtain the database connection.
            let db_conn = data_read.get::<DatabaseConnection>().unwrap();
            // Read the configured prefix of the guild from the database.
            let db_prefix = {
                let db_conn = db_conn.write().await;
                db_conn.query("SELECT prefix FROM prefixes WHERE guild_id = $1",
                    &[&gid])
                    .await
                    .expect("Could not query the database.")
            };
            // If the guild doesn't have a configured prefix, return the default prefix.
            if db_prefix.is_empty() {
                prefix = ".".to_string();
            // Else, just read the value that was stored on the database and return it.
            } else {
                let row = db_prefix.first().unwrap();
                let p = row.get::<_, Option<&str>>(0);
                prefix = p.unwrap().to_string();
            }
        // If the message was sent on a dm, there's no guild, so just return the default prefix.
        } else {
            prefix = ".".to_string();
        }

        // Get the list of booru commands and the data of the json.
        let commands = data_read.get::<BooruCommands>();
        let boorus = data_read.get::<BooruList>().unwrap();

        // if the message content starts with a prefix of the guild
        if msg.content.starts_with(&prefix) {
            // remove the prefix from the message content
            let command = msg.content.replacen(&prefix, "", 1);
            // split the message words into a Vector
            let words = command.split(" ").collect::<Vec<&str>>();
            // get the first item of the Vector, aka the command name.
            // and everything after it.
            let (command_name, parameters) = &words.split_first().unwrap();

            // if the command invoked is on the list of booru commands
            // (obtained from "global" data)
            if commands.as_ref().unwrap().contains(&command_name.to_string()){
                let booru: Booru = {
                    // Get the Booru default values
                    let mut x = Booru::default();
                    // Set X to the data on the json matching the command invoked
                    for b in boorus {
                        if b.names.contains(&command_name.to_string()) {
                            x = b.clone();
                        }
                    }
                    x
                };
                // transform the parameters into a string
                let mut parameters_str = parameters.iter().map(|word| format!(" {}", word)).collect::<String>();
                // if there are parameters
                if parameters_str != "".to_string() {
                    // remove the first space on the string, created by the previous function
                    parameters_str = parameters_str.chars().next().map(|c| &parameters_str[c.len_utf8()..]).unwrap().to_string();
                }
                // Create an Args delimiting the string with a space (' ')
                let params = Args::new(&parameters_str, &[Delimiter::Single(' ')]);

                // Call the booru function with the obtained data
                let booru = get_booru(&mut ctx.clone(), &msg.clone(), &booru, params).await;
                if let Err(why) = booru {
                    // Handle any error that may occur.
                    let why = why.to_string();
                    let reason = format!("There was an error executing the command: {}", capitalize_first(&why).await);
                    let _ = msg.channel_id.say(&ctx, reason).await;
                }
            }
        }

        // Get the list of channels where the bot is allowed to be annoying
        let annoyed_channels = data_read.get::<AnnoyedChannels>();
        // if the channel the message was sent on is on the list
        if annoyed_channels.as_ref().map(|set| set.contains(&msg.channel_id.0)).unwrap_or(false) {
            // NO U
            if msg.content == "no u" {
                let _ = msg.reply(&ctx, "no u").await; // reply pings the user
            // AYY LMAO
            } else if msg.content == "ayy" {
                let _ = msg.channel_id.say(&ctx, "lmao").await; // say just send the message

            }
        }
    }

    /// on_raw_reaction_add event on d.py
    /// This function triggers every time a reaction gets added to a message.
    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // Ignores all reactions that come from the bot itself.
        if &add_reaction.user_id.0 == ctx.cache.read().await.user.id.as_u64() {
            return;
        }

        // gets the message the reaction happened on
        let msg = ctx
            .http
            .as_ref()
            .get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
            .await
            .expect("Error while obtaining message");

        // Obtain the "global" data in read mode
        let data_read = ctx.data.read().await;

        // Check if the channel is on the list of channels that can be annoyed
        let annoyed_channels = data_read.get::<AnnoyedChannels>();
        let annoy = if annoyed_channels.as_ref().unwrap().contains(&msg.channel_id.0) {true} else {false};

        match add_reaction.emoji {
            // Matches custom emojis.
            ReactionType::Custom{id, ..} => {
                // If the emote is the GW version of slof, React back.
                // This also shows a couple ways to do error handling.
                if id.0 == 375_459_870_524_047_361 {
                    let reaction = msg.react(&ctx, add_reaction.emoji).await;
                    if let Err(why) = reaction {
                        eprintln!("There was an error adding a reaction: {}", why)
                    }
                    if annoy {
                        let _ = msg.channel_id.say(&ctx, format!("<@{}>: qt", add_reaction.user_id.0)).await;
                    }
                }
            },
            // Matches unicode emojis.
            ReactionType::Unicode(s) => {
                if annoy {
                    // This will not be kept here for long, as i see it being very annoying eventually.
                    if s == "🤔" {
                        let _ = msg.channel_id.say(&ctx, format!("<@{}>: What ya thinking so much about",
                                                                 add_reaction.user_id.0)).await;
                    }
                } else {
                    // This makes every message sent by the bot get deleted if 🚫 is on the reactions.
                    // aka If you react with 🚫 on any message sent by the bot, it will get deleted.
                    // This is helpful for antispam and anti illegal content measures.
                    if s == "🚫" {
                        let msg = ctx
                            .http
                            .as_ref()
                            .get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
                            .await
                            .expect("Error while obtaining message");
                        if msg.author.id == ctx.cache.read().await.user.id {
                            let _ = msg.delete(&ctx).await;
                        }
                    }
                }
            },
            // Ignore the rest of the cases.
            _ => (), // complete code
            //_ => {}, // incomplete code / may be longer in the future
        }
    }
}


// This is for errors that happen before command execution.
#[hook]
async fn on_dispatch_error(ctx: &mut Context, msg: &Message, error: DispatchError) {
    match error {
        // Notify the user if the reason of the command failing to execute was because of
        // inssufficient arguments.
        DispatchError::NotEnoughArguments { min, given } => {
            let s = format!("I need {} arguments to run this command, but i was only given {}.", min, given);
            // Send the message, but supress any errors that may occur.
            let _ = msg.channel_id.say(&ctx, s).await;
        },
        DispatchError::IgnoredBot {} => {
            return;
        },
        // eprint prints to stderr rather than stdout.
        _ => {
            eprintln!("An unhandled dispatch error has occurred:");
            eprintln!("{:?}", error);
        }
    }
}

// This lambda/closure function executes every time a command finishes executing.
// It's used here to handle errors that happen in the middle of the command.
#[hook]
async fn after(ctx: &mut Context, msg: &Message, cmd_name: &str, error: CommandResult) {
    // error is the command result.
    // inform the user about an error when it happens.
    if let Err(why) = &error {
        eprintln!("Error while ruiing {}:\n{:?}", &cmd_name, &error);
        let err = why.0.to_string();
        let _ = msg.channel_id.say(&ctx, &err).await;
    }
}

// Small error event that triggers when a command doesn't exist.
//.unrecognised_command(|_, _, unknown_command_name| {
//    eprintln!("Could not find command named '{}'", unknown_command_name);
//}

#[hook]
async fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> { // Custom per guild prefixes.
    // obtain the guild id of the command message.
    let guild_id = &msg.guild_id;

    let p;
    // If the command was invoked on a guild
    if let Some(id) = guild_id {
        // Get the real guild id, and the i64 type becaues that's what postgre uses.
        let gid = id.0 as i64;
        // Open the context data lock in read mode.
        let data = ctx.data.read().await;
        // Obtain the database connection for the data.
        let db_conn = data.get::<DatabaseConnection>().unwrap();
        // Obtain the configured prefix from the database
        let db_prefix = {
            let db_conn = db_conn.write().await;
            db_conn.query("SELECT prefix FROM prefixes WHERE guild_id = $1",
                         &[&gid]).await.expect("Could not query the database.")
        };
        // If the guild did nto configure a default prefix, return the default prefix.
        if db_prefix.is_empty() {
            p = ".".to_string();
        // Else return the configured prefix.
        } else {
            let row = db_prefix.first().unwrap();
            let prefix = row.get::<_, Option<&str>>(0);
            p = prefix.unwrap().to_string();
        }
    // If the command was invoked on a dm
    } else {
        p = ".".to_string();
    };
    // dynamic_prefix() needs an Option<String>
    Some(p)
}




// The main function!
// Here's where everything starts.
// This main function is a little special, as it returns Result
// which allows ? to be used for error handling.
#[tokio::main(core_threads=8)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Opens the config.toml file and reads it's content
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // gets the data from the config.toml file
    let configuration = contents.parse::<Value>().unwrap();
    
    if configuration["enable_tracing"].as_bool().unwrap() {
        LogTracer::init()?;
        log::trace!("test log");
        // obtains the tracing level from the config
        let base_level = configuration["trace_level"].as_str().unwrap();
        let level = match base_level {
            "error" => Level::ERROR,
            "warn" => Level::WARN,
            "info" => Level::INFO,
            "debug" => Level::DEBUG,
            "trace" => Level::TRACE,
            _ => Level::TRACE,
        };

        let subscriber = FmtSubscriber::builder()
            .with_max_level(level)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;

        let number_of_yaks = 3;
        info!(number_of_yaks, "Preparing to shave taks...");
    }

    // obtains the discord token from the config
    let bot_token = configuration["discord"].as_str().unwrap();
    // Defines a client with the token obtained from the config.toml file.
    // This also starts up the Event Handler structure defined earlier.
    
    let http = Http::new_with_token(&bot_token);

    // Obtains and defines the owner/owners of the Bot Application
    // and the bot id. 
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Time to configure the Command Framework!
    // This is what allows for easier and faster commaands.
    let framework = StandardFramework::new() // Create a new framework
        .configure(|c| c
            //.prefixes(vec![".", "arc!"]) // Add a list of prefixes to be used to invoke commands.
            .on_mention(Some(bot_id)) // Add a bot mention as a prefix.
            .dynamic_prefix(dynamic_prefix)
            .with_whitespace(true) // Allow a whitespace between the prefix and the command name.
            .owners(owners) // Defines the owners, this can be later used to make owner specific commands.
            .case_insensitivity(true) // Makes the prefix and command be case insensitive.
        )
        .on_dispatch_error(on_dispatch_error)
        .after(after)

        .group(&META_GROUP) // Load `Meta` command group
        .group(&FUN_GROUP) // Load `Fun` command group
        .group(&OSU_GROUP) // Load `osu!` command group
        .group(&MOD_GROUP) // Load `moderation` command group
        .group(&SANKAKU_GROUP) // Load `SankakuComplex` command group
        .group(&ALLBOORUS_GROUP) // Load `Boorus` command group
        .group(&CONFIGURATION_GROUP) // Load `Configuration` command group
        .group(&IMAGEMANIPULATION_GROUP) // Load `image manipulaiton` command group
        .help(&MY_HELP); // Load the custom help command.
    let mut client = Client::new_with_framework(&bot_token, Handler, framework).await?;

    // Block to define global data.
    // and so the data lock is not kept open in write mode.
    {
        // Open the data lock in write mode.
        let mut data = client.data.write().await;

        // Add the database connection to the data.
        let db = get_database().await?;
        data.insert::<DatabaseConnection>(Arc::clone(&Arc::new(RwLock::new(db))));
        // Add the shard manager to the data.
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        // Add the tokens to the data.
        data.insert::<Tokens>(configuration);

        {
            // Open the boorus.json file
            let mut file = File::open("boorus.json").unwrap();
            // read the content of the file to a string.
            let mut raw_data = String::new();
            file.read_to_string(&mut raw_data).unwrap();

            // Serialize the json string into a BooruRaw struct.
            let boorus: BooruRaw = serde_json::from_str(&raw_data.as_str())
                .expect("JSON was not well-formatted");

            // Add every command on the data to a HashSet
            let mut all_names = HashSet::new();
            for boorus_list in boorus.boorus.iter() {
                for booru in boorus_list.names.iter() {
                    all_names.insert(booru.to_owned());
                }
            }

            // Add the json file struct and the HashSet of all the commands to the data.
            data.insert::<BooruList>(boorus.boorus);
            data.insert::<BooruCommands>(all_names);
        }

        {
            // Obtain the database connection from the data.
            let db_client = Arc::clone(data.get::<DatabaseConnection>().expect("no database connection found"));
            // obtain all the channels where the bot is allowed to be annoyed on from the db.
            let raw_annoyed_channels = {
                let db_client = db_client.write().await;
                db_client.query("SELECT channel_id from annoyed_channels", &[]).await?
            };

            // add every channel id from the db to a HashSet.
            let mut annoyed_channels = HashSet::new();
            for row in raw_annoyed_channels {
                annoyed_channels.insert(row.get::<_, i64>(0) as u64);
            }

            // Insert the HashSet of annoyed channels to the data.
            data.insert::<AnnoyedChannels>(annoyed_channels);
        }
    }


    // start listening for events by starting a single shard
    if let Err(why) = client.start_autosharded().await {
        eprintln!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
