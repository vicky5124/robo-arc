/// This is a discord bot made with `serenity.rs` as a Rust learning project.
/// If you see a lot of different ways to do the same thing, specially with error handling,
/// this is indentional, as it helps me remember the concepts that rust provides, so they can be
/// used in the future for whatever they could be needed.
///
/// This is lisenced with the copyleft license Mozilla Public License Version 2.0

mod utils; // Load the utils module
mod commands; // Load the commands module

use commands::booru::*; // Import everything from the booru module.
use commands::sankaku::*; // Import everything from the sankaku booru module.
use commands::osu::*; // Import everything from the osu module.
use commands::meta::*; // Import everything from the meta module.
use commands::image_manipulation::*; // Import everything from the image manipulation module.
use commands::fun::*; // Import everything from the fun module.
use commands::moderation::*; // Import everything from the moderation module.
use commands::configuration::*; // Import everything from the moderation module.
use utils::database::get_database; // Obtain the get_database function from the utilities.
use utils::basic_functions::capitalize_first; // Obtain the capitalize_first function from the utilities.

use std::{
    collections::{
        HashSet, // Low cost indexable lists
        HashMap, // Basically python dicts
    },
    // For saving / reading files
    fs::File,
    io::prelude::*,
    // For having refferences between threads
    sync::Arc,
};

use postgres::Client as PgClient; // PostgreSQL client
use toml::Value; // To parse the data of .toml files
use serde_json; // To parse the data of .json files
use serde::Deserialize; // To deserialize data into structures

// A synchronous, parallel event dispatcher, used for reactions on specific messages.
use hey_listen::sync::ParallelDispatcher as Dispatcher;

// Serenity! what make's the bot function. Discord API wrapper.
use serenity::{
    utils::Colour,
    client::{
        Client,
        bridge::gateway::ShardManager,
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
        user::OnlineStatus,
        id::{
            UserId,
            //GuildId,
        },
    },
    prelude::{
        EventHandler,
        Context,
        Mutex,
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

// Because "boorus.json" is a list of values.
#[derive(Debug, Deserialize)]
struct BooruRaw {
    boorus: Vec<Booru>,
}

// Defining the structures to be used for global data
struct ShardManagerContainer;
struct DatabaseConnection;
struct Tokens;
struct AnnoyedChannels;
struct RecentIndex;
struct BooruList;
struct BooruCommands;

// Implementing a type for each structure
impl TypeMapKey for ShardManagerContainer {
    // Mutex allows for the refference to be mutated
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for DatabaseConnection {
    // RwLock (aka Read Write Lock) makes the data only modifyable by 1 thread at a time
    type Value = Arc<RwLock<PgClient>>;
}

impl TypeMapKey for Tokens {
    type Value = Value;
}

impl TypeMapKey for AnnoyedChannels {
    type Value = HashSet<u64>;
}

impl TypeMapKey for RecentIndex {
    type Value = HashMap<u64, usize>;
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
#[description = "All the basic commands that every bot should have."]
#[commands(ping, test, invite, source, todo, prefixes, about, changelog)]
struct Meta;

// The SankakuComplex command group.
// This group contains commands for the variants Chan and Idol of the sankaku boorus.
#[group("Sankaku")]
#[description = "All the NSFW/BSFW related commands."]
#[commands(idol)]
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
#[commands(booru_command)]
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
#[commands(qr, urban, translate)]
struct Fun;

#[group("Moderation")]
#[description = "All the moderation related commands."]
#[commands(kick, ban, clear)]
struct Mod;

#[group("Configuration")]
#[description = "All the configuration related commands.
Basic usage:
~~`.config user VALUE DATA`~~
~~`.config guild VALUE DATA`~~
`.config channel VALUE DATA`"]
#[prefixes("config", "configure", "conf")]
#[commands(channel)]
struct Configuration;

// This is a custom help command.
// Each line has the explaination that is required.
#[help]
// This is the basic help message
// We use \ at the end of the line to easily allow for newlines visually on the code.
#[individual_command_tip = "Hello!
If youd like to get extra information about a specific command, just pass it as an argument.
You can also react with 🚫 on any message sent by the bot to delete it.\n"]
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
fn my_help(
    ctx: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let mut ho = help_options.clone();
    ho.embed_error_colour = Colour::from_rgb(255, 30, 30);
    ho.embed_success_colour= Colour::from_rgb(141, 91, 255);
    help_commands::with_embeds(ctx, msg, args, &ho, groups, owners)
}



struct Handler; // Defines the handler to be used for events.

impl EventHandler for Handler {
    /// on_ready event on d.py
    /// This function triggers when the bot is ready.
    fn ready(&self, ctx: Context, ready: Ready) {
        // Changes the presence of the bot to "Listening to C++ cry a Rusted death."
        ctx.set_presence(
            Some(Activity::listening("C++ cry a Rusted death.")),
            OnlineStatus::Online
        );

        println!("{} is ready to rock!", ready.user.name);
    }

    /// on_message event on d.py
    /// This function triggers every time a message is sent.
    fn message(&self, ctx: Context, msg: Message) {
        // Ignores itself.
        if &msg.author.id.0 == ctx.cache.read().user.id.as_u64() {
            return;
        }

        // Read the global data
        let data_read = ctx.data.read();

        let guild_prefix = [".", "arc!"]; // Change this with the specific guild prefixes when dynamic prefixes gets implemented.
        // Get the list of booru commands and the data of the json.
        let commands = data_read.get::<BooruCommands>();
        let boorus = data_read.get::<BooruList>().unwrap();

        // iterate over the guild prefixes
        for prefix in &guild_prefix {
            // if the message content starts with a prefix of the guild
            if msg.content.starts_with(prefix) {
                // remove the prefix from the message content
                let command = msg.content.replacen(prefix, "", 1);
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
                    if let Err(why) = get_booru(&mut ctx.clone(), &msg.clone(), &booru, params) {
                        // Handle any error that may occur.
                        let _ = msg.channel_id.say(&ctx, format!("There was an error executing the command: {}", capitalize_first(&why.to_string())));
                    };
                }
            }
        }

        // Get the list of channels where the bot is allowed to be annoying
        let annoyed_channels = data_read.get::<AnnoyedChannels>();
        // if the channel the message was sent on is on the list
        if annoyed_channels.as_ref().map(|set| set.contains(&msg.channel_id.0)).unwrap_or(false) {
            // NO U
            if msg.content == "no u" {
                let _ = msg.reply(&ctx, "no u"); // reply pings the user
            // AYY LMAO
            } else if msg.content == "ayy" {
                let _ = msg.channel_id.say(&ctx, "lmao"); // say just send the message

            }
        }

        // This is an alternative way to make commands that doesn't involve the Command Framework.
        // this is not recommended as it would block the event thread, which Framework Commands
        // don't do.
        // This command just an example command made this way.
        //
        //if msg.content == ".hello" {
        //  msg.channel_id.say("Hello!")
        //}
    }

    /// on_raw_reaction_remove event on d.py
    /// This function triggers every time a reaction gets removed on a message.
    fn reaction_remove(&self, ctx: Context, add_reaction: Reaction) {
        // Ignores all reactions that come from the bot itself.
        if &add_reaction.user_id.0 == ctx.cache.read().user.id.as_u64() {
            return;
        }
        
        // Triggers reaction events from the commands.
        let dispatcher = {
            let mut context = ctx.data.write();
            context.get_mut::<DispatcherKey>().expect("Expected Dispatcher.").clone()
        };

        dispatcher.write().dispatch_event(
            &DispatchEvent::ReactEvent(add_reaction.message_id, add_reaction.emoji.clone(), false));

    }

    /// on_raw_reaction_add event on d.py
    /// This function triggers every time a reaction gets added to a message.
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // Ignores all reactions that come from the bot itself.
        if &add_reaction.user_id.0 == ctx.cache.read().user.id.as_u64() {
            return;
        }

        // Triggers reaction events from the commands.
        let dispatcher = {
            let mut context = ctx.data.write();
            context.get_mut::<DispatcherKey>().expect("Expected Dispatcher.").clone()
        };

        dispatcher.write().dispatch_event(
            &DispatchEvent::ReactEvent(add_reaction.message_id, add_reaction.emoji.clone(), false));

        // gets the message the reaction happened on
        let msg = ctx.http.as_ref().get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
            .expect("Error while obtaining message");

        // Obtain the "global" data in read mode
        let data_read = ctx.data.read();

        // Check if the channel is on the list of channels that can be annoyed
        let annoyed_channels = data_read.get::<AnnoyedChannels>();
        let annoy = if annoyed_channels.as_ref().unwrap().contains(&msg.channel_id.0) {true} else {false};

        match add_reaction.emoji {
            // Matches custom emojis.
            ReactionType::Custom{id, ..} => {
                // If the emote is the GW version of slof, React back.
                // This also shows a couple ways to do error handling.
                if id.0 == 375459870524047361 {
                    let reaction = msg.react(&ctx, add_reaction.emoji);
                    if let Err(why) = reaction {
                        eprintln!("There was an error adding a reaction: {}", why)
                    }
                    if annoy {
                        let _ = msg.channel_id.say(&ctx, format!("<@{}>: qt", add_reaction.user_id.0));
                    }
                }
            },
            // Matches unicode emojis.
            ReactionType::Unicode(s) => {
                if annoy {
                    // This will not be kept here for long, as i see it being very annoying eventually.
                    if s == "🤔" {
                        let _ = msg.channel_id.say(&ctx, format!("<@{}>: What ya thinking so much about",
                                                                 add_reaction.user_id.0));
                    }
                } else {
                    // This makes every message sent by the bot get deleted if 🚫 is on the reactions.
                    // aka If you react with 🚫 on any message sent by the bot, it will get deleted.
                    // This is helpful for antispam and anti illegal content measures.
                    if s == "🚫" {
                        let msg = ctx.http.as_ref().get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
                            .expect("Error while obtaining message");
                        if msg.author.id == ctx.cache.read().user.id {
                            let _ = msg.delete(&ctx);
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



/// The main function!
/// Here's where everything starts.
/// This main function is a little special, as it returns Result, which allows ? to be used for
/// error handling.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Opens the config.toml file and reads it's content
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // gets the discord and osu api tokens from config.toml
    let tokens = contents.parse::<Value>().unwrap();
    let bot_token = tokens["discord"].as_str().unwrap();
    // Defines a client with the token obtained from the config.toml file.
    // This also starts up the Event Handler structure defined earlier.
    let mut client = Client::new(
        bot_token,
        Handler)?;

    // Closure to define global data.
    {
        let mut data = client.data.write();
        data.insert::<DatabaseConnection>(Arc::clone(&Arc::new(RwLock::new(get_database()?)))); // Make the database connection global.
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager)); // Make the shard manager global.
        data.insert::<Tokens>(tokens);

        let mut dispatcher: Dispatcher<DispatchEvent> = Dispatcher::default();
        dispatcher.num_threads(4).expect("Could not construct threadpool");
        data.insert::<DispatcherKey>(Arc::new(RwLock::new(dispatcher)));

        {
            let mut file = File::open("boorus.json").unwrap();
            let mut raw_data = String::new();
            file.read_to_string(&mut raw_data).unwrap();

            let boorus: BooruRaw = serde_json::from_str(&raw_data.as_str())
                .expect("JSON was not well-formatted");

            let mut all_names = HashSet::new();

            for boorus_list in boorus.boorus.iter() {
                for booru in boorus_list.names.iter() {
                    all_names.insert(booru.to_owned());
                }
            }

            data.insert::<BooruList>(boorus.boorus);
            data.insert::<BooruCommands>(all_names);
        }

        {
            let db_client = Arc::clone(data.get::<DatabaseConnection>().expect("no database connection found"));
            let raw_annoyed_channels = {
                let mut db_client = db_client.write();
                db_client.query("SELECT channel_id from annoyed_channels", &[])?
            };
            let mut annoyed_channels = HashSet::new();
            
            for row in raw_annoyed_channels {
                annoyed_channels.insert(row.get::<_, i64>(0) as u64);
            }

            data.insert::<AnnoyedChannels>(annoyed_channels);
        }
    }

    &client.threadpool.set_num_threads(8);
    
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
            .prefixes(vec![".", "arc!"]) // Add a list of prefixes to be used to invoke commands.
            .on_mention(Some(bot_id)) // Add a bot mention as a prefix.
            .with_whitespace(true) // Allow a whitespace between the prefix and the command name.
            //.dynamic_prefixes({
            //    let vec = [",,,", ",,,,"];
            //    let mut index = 0;

            //    let x = |_ctx: &mut Context, msg: &Message| {
            //        if msg.is_private() {
            //            return Some(",".to_owned());
            //        } else {
            //            let guild_id = msg.guild_id.unwrap_or(GuildId(0));
            //            if guild_id.0 == 182892283111276544 {
            //                let indexed = vec[0];
            //                let pick = Some(indexed.to_owned());
            //                index += 1;
            //                return pick;
            //        }
            //    } 
            //    return Some(",,".to_owned());
            //    };
            //    vec![x]
            //})
            .owners(owners) // Defines the owners, this can be later used to make owner specific commands.
        )

        // This is for errors that happen before command execution.
        .on_dispatch_error(|ctx, msg, error| {
            eprintln!("{:?}", error);
            match error {
                // Notify the user if the reason of the command failing to execute was because of
                // inssufficient arguments.
                DispatchError::NotEnoughArguments { min, given } => {
                    let s = format!("I need {} arguments to run this command, but i was only given {}.", min, given);

                    let _ = msg.channel_id.say(&ctx, s);
                },
                _ => eprintln!("Unhandled dispatch error."),
            }
        })
        
        // This lambda/closure function executes every time a command finishes executing.
        // It's used here to handle errors that happen in the middle of the command.
        .after(|ctx, msg, _cmd_name, error| {
            if let Err(why) = &error {
                eprintln!("{:?}", &error);
                let err = format!("{}", why.0);
                let _ = msg.channel_id.say(&ctx, &err);
            }
        })

        // Small error event that triggers when a command doesn't exist.
        //.unrecognised_command(|_, _, unknown_command_name| {
        //    eprintln!("Could not find command named '{}'", unknown_command_name);
        //})

        .group(&META_GROUP) // Load `Meta` command group
        .group(&FUN_GROUP) // Load `Fun` command group
        .group(&OSU_GROUP) // Load `osu!` command group
        .group(&MOD_GROUP) // Load `moderation` command group
        .group(&SANKAKU_GROUP) // Load `SankakuComplex` command group
        .group(&ALLBOORUS_GROUP) // Load `Boorus` command group
        .group(&CONFIGURATION_GROUP) // Load `Configuration` command group
        .group(&IMAGEMANIPULATION_GROUP) // Load `image manipulaiton` command group
        .help(&MY_HELP) // Load the custom help.
    );

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        eprintln!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
