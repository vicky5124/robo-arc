#![feature(vec_remove_item)]
/// This is a discord bot made with `serenity.rs` as a Rust learning project.
/// If you see a lot of different ways to do the same thing, specially with error handling,
/// this is indentional, as it helps me remember the concepts that rust provides, so they can be
/// used in the future for whatever they could be needed.
///
/// This is lisenced with the copyleft license Mozilla Public License Version 2.0

mod utils; // Load the utils module
mod commands; // Load the commands module
mod notifications;

// Import this 2 commands in specific with a different name
// as they interfere with the configuration commands that are also being imported.
use commands::booru::{
    BEST_BOY_COMMAND as BG_COMMAND,
    BEST_GIRL_COMMAND as BB_COMMAND,
};
use commands::meta::PREFIX_COMMAND as PREFIXES_COMMAND;

use crate::notifications::{
    notification_loop,
    TwitchStreamData,
};

use commands::booru::*; // Import everything from the booru module.
use commands::sankaku::*; // Import everything from the sankaku booru module.
use commands::osu::*; // Import everything from the osu module.
use commands::meta::*; // Import everything from the meta module.
use commands::image_manipulation::*; // Import everything from the image manipulation module.
use commands::fun::*; // Import everything from the fun module.
use commands::moderation::*; // Import everything from the moderation module.
use commands::configuration::*; // Import everything from the configuration module.
use commands::music::*; // Import everything from the configuration module.
use commands::dictionary::*; // Import everything from the dictionary module.
use commands::serenity_docs::*; // Import everything from the serenity_docs module.

use utils::database::obtain_pool; // Obtain the get_database function from the utilities.
use utils::basic_functions::capitalize_first; // Obtain the capitalize_first function from the utilities.

use std::{
    collections::{
        HashSet, // Low cost indexable lists.
        //HashMap,
    },
    // For saving / reading files
    fs::File,
    io::prelude::*,

    // For having refferences between threads
    sync::Arc,
    convert::TryInto,
    time::{
        Instant,
        Duration,
    },

    net::SocketAddr,
    str::FromStr,
};

use tokio::sync::Mutex;

use dotenv;
use tracing::{
    // Log macros.
    info,
    trace,
    error,
    // Others
    Level,
    instrument
};

use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};
use tracing_log::LogTracer;
//use tracing_futures::Instrument;
//use log;

use sqlx::PgPool; // PostgreSQL Pool Structure
use futures::TryStreamExt;
use futures::stream::StreamExt;

use toml::Value; // To parse the data of .toml files
use serde_json; // To parse the data of .json files (where's serde_toml smh)
use serde::{
    Deserialize, // To deserialize data into structures
    Serialize,
};

use warp::{
    Filter,
    reply::json,
    reply::Json
};

use serenity_lavalink::LavalinkClient;

// Serenity! what make's the bot function. Discord API wrapper.
use serenity::{
    async_trait,
    utils::Colour, // To change the embed help color
    client::{
        Client, // To create a client that runs eveyrthing.
        bridge::{
            gateway::{
                ShardManager, // To manage shards, or in the case of this small bot, just to get the latency of it for ping.
                GatewayIntents,
            },
            voice::ClientVoiceManager,
        },
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
        id::{
            UserId,
            ChannelId,
            GuildId,
        },
        event::VoiceServerUpdateEvent,
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
        Reason,
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
    post_url: String,
}

// Because "boorus.json" boorus key is a list of values.
#[derive(Debug, Deserialize)]
struct BooruRaw {
    boorus: Vec<Booru>,
}

#[derive(Serialize)]
struct Allow {
    allowed: bool,
}

// Defining the structures to be used for "global" data
// this data is not really global, it's just shared with Context.data
struct ShardManagerContainer; // Shard manager to use for the latency.
struct ConnectionPool; // The connection to the database, because having multiple connections is a bad idea.
struct Tokens; // For the configuration found on "config.toml"
struct AnnoyedChannels; // This is a HashSet of all the channels the bot is allowed to be annoyed on.
struct BooruList; // This is a HashSet of all the boorus found on "boorus.json"
struct BooruCommands; // This is a HashSet of all the commands/aliases found on "boorus.json"
struct NotificationStatus; // This is the status of the thread checking for notifications.
struct VoiceManager; //  This is the struct for the voice manager.
struct Lavalink; //  This is the struct for the lavalink client.
struct SentTwitchStreams; //  This is the struct for the stream data that has already been sent.
struct Uptime; //  This is for the startup time of the bot.
struct VoiceGuildUpdate; //  This is for the startup time of the bot.

// Implementing a type for each structure
// This is made to make a Map<Struct, TypeValue>
impl TypeMapKey for ShardManagerContainer {
    // Mutex allows for the arc refference to be mutated
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for ConnectionPool {
    // RwLock (aka Read Write Lock) makes the data only modifyable by 1 thread at a time
    // So you can only have the lock open with write a single use at a time.
    // You can have multiple reads, but you can't read as soon as the lock is opened for writing.
    //type Value = Arc<RwLock<PgPool>>;
    type Value = PgPool;
}

impl TypeMapKey for Tokens {
    // Value is not to be confused with the other values.
    // This Value is toml::Value
    type Value = Value;
}

impl TypeMapKey for AnnoyedChannels {
    type Value = RwLock<HashSet<u64>>;
}

impl TypeMapKey for BooruList {
    type Value = Vec<Booru>; 
}

impl TypeMapKey for BooruCommands {
    type Value = HashSet<String>; 
}

impl TypeMapKey for NotificationStatus {
    type Value = bool;
}

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for Lavalink {
    type Value = Arc<RwLock<LavalinkClient>>;
}

impl TypeMapKey for SentTwitchStreams {
    type Value = Arc<RwLock<Vec<TwitchStreamData>>>;
}

impl TypeMapKey for Uptime {
    type Value = Instant;
}

impl TypeMapKey for VoiceGuildUpdate {
    type Value = Arc<RwLock<HashSet<GuildId>>>;
}


#[group("Master")]
#[sub_groups(Meta, Sankaku, Osu, Fun, Music, AllBoorus, ImageManipulation, Mod, SerenityDocs)]
struct Master;

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
#[commands(configure_osu, recent, score, osu_profile)]
struct Osu;

// The Booru command group.
// This group will contain every single command from every booru that gets implemented.
// As you can see on the last line, the description also supports url markdown.
#[group("Image Boards")]
#[description = "All the booru related commands.\n\
Available parameters:
`-x` Explicit
`-q` Questionable
`-s` Safe
`-n` Non Safe (Random between E or Q)

Inspired by -GN's WaifuBot ([source](https://github.com/isakvik/waifubot/))"]
#[commands(booru_command, BB, BG, n_hentai)] // We imported BB_COMMAND and BG_COMMAND, but this macro automatically adds _COMMAND, so we don't put that.
struct AllBoorus;

// The Image Manipulation command group.
// This group contains all the commands that manipulate images.
#[group("Image Manipulation")]
#[description = "All the image manipulaiton based commands."]
#[commands(gray, pride, pride_pre_grayscaled)]
struct ImageManipulation;

// The FUN command group.
// Where all the random commands go into lol
#[group("Fun")]
#[description = "All the random and fun commands."]
#[commands(profile, qr, urban, dictionary, translate, duck_duck_go, encrypt, decrypt, calculator, tic_tac_toe, higher_or_lower)]
struct Fun;

// The moderation command group.
#[group("Moderation")]
#[description = "All the moderation related commands."]
#[commands(kick, ban, clear)]
struct Mod;

// The music command group.
#[group("Music")]
#[description = "All the voice and music related commands."]
#[only_in("guilds")]
#[commands(join, leave, play, play_playlist, pause, stop, skip, seek, shuffle, queue, now_playing)]
struct Music;

#[group("Serenity Documentation")]
#[description = "All the commands related to serenity's documentation."]
#[commands(example, rtfm)]
struct SerenityDocs;

// The configuration command.
// Technically a group, but it only has a single command.
#[group("Configuration")]
#[description = "All the configuration related commands.
Basic usage:
`config user VALUE DATA`
`config guild VALUE DATA`
`config channel VALUE DATA`"]
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
All the command examples through out the help will be shown without prefix, use whatever prefix is configured on the server instead.

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
    ctx: &Context,
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

        let ctx = Arc::new(ctx);
        let ctx_clone = Arc::clone(&ctx);

        let (info, web_server_info) = {
            let read_data = ctx.data.read().await;
            let config = read_data.get::<Tokens>().unwrap();
            (
                config["presence"].clone(),
                config["web_server"].clone(),
            )
        };

        tokio::spawn(async move {
            let guilds = ctx_clone.clone().cache
                .guilds()
                .await
                .iter()
                .map(|i| i.0)
                .collect::<Vec<_>>();

            let routes = warp::path::param().map(move |guild_id: u64| -> Json {
                let data = Allow {
                    allowed: guilds.contains(&guild_id.clone())
                };
            
                json(&data)
            });

            let ip = web_server_info["server_ip"].as_str().unwrap();
            let port = web_server_info["server_port"].as_integer().unwrap();

            warp::serve(routes)
                .run(SocketAddr::from_str(
                    format!("{}:{}", ip, port).as_str()
                ).unwrap()
            ).await;
        });

        if info["play_or_listen"].as_str().unwrap() == "playing" {
            ctx.set_presence(
                Some(Activity::playing(info["status"].as_str().unwrap())),
                OnlineStatus::Online
            ).await;
        } else if info["play_or_listen"].as_str().unwrap() == "listening" {
            ctx.set_presence(
                Some(Activity::listening(info["status"].as_str().unwrap())),
                OnlineStatus::Online
            ).await;
        }


        let status = {
            let read_data = ctx.data.read().await;
            read_data.get::<NotificationStatus>().unwrap().clone()
        };

        println!("{} is ready to rock!", ready.user.name);

        if !status {
            let ctx_clone = Arc::clone(&ctx);

            let notification_loop = tokio::spawn(async move {notification_loop(ctx_clone).await});
            {
                let mut data = ctx.data.write().await;
                data.insert::<NotificationStatus>(true);
            }
            let _ = notification_loop.await;
            {
                let mut data = ctx.data.write().await;
                data.insert::<NotificationStatus>(false);
            }
        }

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

        // Read the global data lock
        let data_read = ctx.data.read().await;

        // Get the list of channels where the bot is allowed to be annoying
        let annoyed_channels = data_read.get::<AnnoyedChannels>().unwrap();
        // if the channel the message was sent on is on the list
        if (annoyed_channels.read().await).contains(&msg.channel_id.0) {
            // NO U
            if msg.content == "no u" {
                let _ = msg.reply(&ctx, "no u").await; // reply pings the user
            // AYY LMAO
            } else if msg.content == "ayy" {
                let _ = msg.channel_id.say(&ctx, "lmao").await; // say just send the message

            }
        }
        if msg.content.contains("discordapp.com/channels/") {
            let mut splits = msg.content.split('/');
            if splits.clone().count() == 7 {
                let channel_id  = splits.nth(5).unwrap_or("0").parse::<u64>().expect("NaN");
                if let Ok(chan) = ChannelId(channel_id).to_channel(&ctx).await {
                    if chan.is_nsfw() {
                        let _ = msg.react(&ctx, '🇳').await;
                        let _ = msg.react(&ctx, '🇸').await;
                        let _ = msg.react(&ctx, '🇫').await;
                        let _ = msg.react(&ctx, '🇼').await;
                    }
                }
            }
        }

        if msg.content.starts_with("3.14") || msg.content.starts_with("3,14") {
            let content = msg.content.replace(",", ".");
            let pif = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679821480865132823066470938446095505822317253594081284811174502841027019385211055596446229489549303819644288109756659334461284756482337867831652712019091456485669234603486104543266482133936072602491412737245870066063155881748815209209628292540917153643678925903600113305305488204665213841469519415116094330572703657595919530921861173819326117931051185480744623799627495673518857527248912279381830119491298336733624406566430860213949463952247371907021798609437027705392171762931767523846748184676694051320005681271452635608277857713427577896091736371787214684409012249534301465495853710507922796892589235420199561121290219608640344181598136297747713099605187072113499999983729780499510597317328160963185950244594553469083026425223082533446850352619311881710100031378387528865875332083814206171776691473035982534904287554687311595628638823537875937519577818577805321712268066130019278766111959092164201989";

            let l = if pif.len() > content.len() {
                content.len()
            } else {
                pif.len()
            };


            let mut correct = true;

            for i in 0..l {
                if pif.chars().into_iter().nth(i) != content.chars().into_iter().nth(i) {
                    correct = false;
                }
            }

            if correct {
                let _ = msg.react(&ctx, '✅').await;
            } else {
                let _ = msg.react(&ctx, '❌').await;
            }
        }

        if msg.guild_id.unwrap_or_default().0 == 159686161219059712 {
            if msg.content.contains("ping me on nsfw!") {
                let _ = ChannelId(354294536198946817).say(&ctx, format!("<@{}>", msg.author.id)).await;
            } else if msg.content.contains("ping him on nsfw!") {
                let _ = ChannelId(354294536198946817).say(&ctx, "<@299624139633721345>").await;
            }
        }
    }

    /// on_raw_reaction_add event on d.py
    /// This function triggers every time a reaction gets added to a message.
    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // Ignores all reactions that come from the bot itself.
        if &add_reaction.user_id.0 == ctx.cache.current_user().await.id.as_u64() {
            return;
        }

        // gets the message the reaction happened on
        let msg = if let Ok(x) = ctx.http.as_ref()
            .get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
            .await { x } else {
                return;
        };

        // Obtain the "global" data in read mode
        let data_read = ctx.data.read().await;

        // Check if the channel is on the list of channels that can be annoyed
        let annoyed_channels = data_read.get::<AnnoyedChannels>().unwrap();
        let annoy = (annoyed_channels.read().await).contains(&msg.channel_id.0);

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
                        if msg.author.id == ctx.cache.current_user().await.id {
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

    async fn voice_server_update(&self, ctx: Context, voice: VoiceServerUpdateEvent) {
        if let Some(guild_id) = voice.guild_id {
            let data = ctx.data.read().await;
            let voice_server_lock = data.get::<VoiceGuildUpdate>().unwrap();
            let mut voice_server = voice_server_lock.write().await;
            voice_server.insert(guild_id);
        }
    }
}


// This is for errors that happen before command execution.
#[hook]
async fn on_dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        // Notify the user if the reason of the command failing to execute was because of
        // inssufficient arguments.
        DispatchError::NotEnoughArguments { min, given } => {
            let s = {
                if given == 0  && min == 1{
                    format!("I need an argument to run this command")
                } else if given == 0 {
                    format!("I need atleast {} arguments to run this command", min)
                } else {
                    format!("I need {} arguments to run this command, but i was only given {}.", min, given)
                }
            };
            // Send the message, but supress any errors that may occur.
            let _ = msg.channel_id.say(ctx, s).await;
        },
        DispatchError::IgnoredBot {} => {
            return;
        },
        DispatchError::CheckFailed(_, reason) => {
            if let Reason::User(r) = reason {
                let _ = msg.channel_id.say(ctx, r).await;
            }
        },
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
async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    if let Some(guild_id) = msg.guild_id {
        let data_read = ctx.data.read().await;
        let pool = data_read.get::<ConnectionPool>().unwrap();

        let disallowed_commands = sqlx::query!("SELECT disallowed_commands FROM prefixes WHERE guild_id = $1", guild_id.0 as i64)
            .fetch_optional(pool)
            .await
            .unwrap();

        if let Some(x) = disallowed_commands {
            if let Some(disallowed_commands) = x.disallowed_commands {
                if disallowed_commands.contains(&cmd_name.to_string()) {
                    let _ = msg.reply(ctx, "This command has been disabled by an administrtor of this guild.").await;
                    return false;
                }
            }
        }

        if cmd_name == "play" || cmd_name == "play_playlist" {
            let manager_lock = ctx.data.read().await
                .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
            let mut manager = manager_lock.lock().await;

            if manager.get(guild_id).is_none() {
                if let Err(why) =  _join(ctx, msg, &mut manager).await {
                    error!("While running command: {}", cmd_name);
                    error!("{:?}", why);
                    return false;
                }
                std::mem::drop(manager);
                loop {
                    let data = ctx.data.read().await;
                    let vgu_lock = data.get::<VoiceGuildUpdate>().unwrap();
                    let mut vgu = vgu_lock.write().await;
                    if !vgu.contains(&guild_id) {
                        tokio::time::delay_for(Duration::from_millis(500)).await;
                    } else {
                        vgu.remove(&guild_id);
                        break;
                    }
                }
            }
        }
    }

    info!("Running command: {}", &cmd_name);
    trace!("Message: {}", &msg.content);
    true
}

// This function executes every time a command finishes executing.
// It's used here to handle errors that happen in the middle of the command.
#[hook]
async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: CommandResult) {
    // error is the command result.
    // inform the user about an error when it happens.
    if let Err(why) = &error {
        error!("Error while running command {}", &cmd_name);
        error!("{:?}", &error);

        let err = why.0.to_string();
        if let Err(_) = msg.channel_id.say(ctx, &err).await {
            error!("Unable to send messages on channel id {}", &msg.channel_id.0);
        };
    }
}

// Small error event that triggers when a command doesn't exist.
#[hook]
async fn unrecognised_command(ctx: &Context, msg: &Message, command_name: &str) {
    let data_read = ctx.data.read().await;
    if let Some(guild_id) = msg.guild_id {
        let pool = data_read.get::<ConnectionPool>().unwrap();

        let disallowed_commands = sqlx::query!("SELECT disallowed_commands FROM prefixes WHERE guild_id = $1", guild_id.0 as i64)
            .fetch_optional(pool)
            .await
            .unwrap();

        if let Some(x) = disallowed_commands {
            if let Some(disallowed_commands) = x.disallowed_commands {
                if disallowed_commands.contains(&"booru_command".to_string()) {
                    let _ = msg.reply(ctx, "This command has been disabled by an administrtor of this guild.").await;
                    return;
                }
            }
        }
    }

    let commands = data_read.get::<BooruCommands>();
    let boorus = data_read.get::<BooruList>().unwrap();

    if commands.as_ref().unwrap().contains(command_name) {
        let booru: Booru = {
            let mut x = Booru::default();
            for b in boorus {
                if b.names.contains(&command_name.to_string()) {
                    x = b.clone();
                }
            }
            x
        };
        let parameters = msg.content.split(command_name).nth(1).unwrap();
        let params = Args::new(&parameters, &[Delimiter::Single(' ')]);

        let booru = get_booru(ctx, &msg, &booru, params).await;
        if let Err(why) = booru {
            // Handle any error that may occur.
            let why = why.to_string();
            let reason = format!("There was an error executing the command: {}", capitalize_first(&why));
            let _ = msg.channel_id.say(ctx, reason).await;
        }
    }
}

#[hook]
async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> { // Custom per guild prefixes.
    //info!("Dynamic prefix call.");
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
        let pool = data.get::<ConnectionPool>().unwrap();
        // Obtain the configured prefix from the database
        let mut db_prefix = sqlx::query!("SELECT prefix FROM prefixes WHERE guild_id = $1", gid)
            .fetch(pool)
            .boxed();

        p = if let Some(result) = db_prefix.try_next().await.expect("Could not query the database") {
            result.prefix.unwrap_or(".".to_string()).to_string()
        } else {
            ".".to_string()
        };

    // If the command was invoked on a dm
    } else {
        p = ".".to_string();
    };
    // dynamic_prefix() needs an Option<String>
    //Some(p.to_lowercase())
    Some(p)
}



// The main function!
// Here's where everything starts.
// This main function is a little special, as it returns Result
// which allows ? to be used for error handling.
#[tokio::main(core_threads=8)]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Opens the config.toml file and reads it's content
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;


    // gets the data from the config.toml file
    let configuration = contents.parse::<Value>().unwrap();
    
    if configuration["enable_tracing"].as_bool().unwrap() {
        LogTracer::init()?;
        trace!("test log");
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

        info!("Tracer initialized.");

        if let Ok(_) = dotenv::dotenv() {
            let subscriber = FmtSubscriber::builder()
                .with_env_filter(EnvFilter::from_default_env())
                .finish();
            tracing::subscriber::set_global_default(subscriber)?;
        } else {
            let subscriber = FmtSubscriber::builder()
                .with_max_level(level)
                .finish();
            tracing::subscriber::set_global_default(subscriber)?;
        };


        info!("Subscriber initialized.");
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
    let std_framework = StandardFramework::new() // Create a new framework
        .configure(|c| c
            //.prefixes(vec![".", "arc!"]) // Add a list of prefixes to be used to invoke commands.
            .on_mention(Some(bot_id)) // Add a bot mention as a prefix.
            .dynamic_prefix(dynamic_prefix)
            .with_whitespace(true) // Allow a whitespace between the prefix and the command name.
            .owners(owners) // Defines the owners, this can be later used to make owner specific commands.
            .case_insensitivity(true) // Makes the prefix and command be case insensitive.
            .blocked_users(vec![UserId(135423120268984330)].into_iter().collect())
        )
        .on_dispatch_error(on_dispatch_error)
        .unrecognised_command(unrecognised_command)
        .before(before)
        .after(after)

        .group(&META_GROUP) // Load `Meta` command group
        .group(&FUN_GROUP) // Load `Fun` command group
        .group(&MUSIC_GROUP) // Load `music` command group
        .group(&OSU_GROUP) // Load `osu!` command group
        .group(&SANKAKU_GROUP) // Load `SankakuComplex` command group
        .group(&ALLBOORUS_GROUP) // Load `Boorus` command group
        .group(&IMAGEMANIPULATION_GROUP) // Load `image manipulaiton` command group
        .group(&MOD_GROUP) // Load `moderation` command group
        .group(&CONFIGURATION_GROUP) // Load `Configuration` command group
        .group(&SERENITYDOCS_GROUP) // Load `serenity_docs` command group
        .help(&MY_HELP); // Load the custom help command.

    let mut client = Client::new(&bot_token)
        .event_handler(Handler)
        .framework(std_framework)
        .add_intent({
            let mut intents = GatewayIntents::all();
            intents.remove(GatewayIntents::GUILD_PRESENCES);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
            intents.remove(GatewayIntents::GUILD_MESSAGE_TYPING);
            intents
        })
        .await?;

    // Block to define global data.
    // and so the data lock is not kept open in write mode.
    {
        // Open the data lock in write mode.
        let mut data = client.data.write().await;

        // Add the database connection to the data.
        let pool = obtain_pool().await?;
        data.insert::<ConnectionPool>(pool.clone());
        // Add the shard manager to the data.
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        // Add the tokens to the data.
        data.insert::<Tokens>(configuration.clone());
        // Add the current status of the notifications.
        data.insert::<NotificationStatus>(false);
        // Add the Voice Manager.
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        // Add the sent streams.
        data.insert::<SentTwitchStreams>(Arc::new(RwLock::new(Vec::new())));
        data.insert::<Uptime>(Instant::now());
        data.insert::<VoiceGuildUpdate>(Arc::new(RwLock::new(HashSet::new())));

        {
            // TODO: get the real shard amount.
            let _shard_count = 1;

            let host = configuration["lavalink"]["host"].as_str().unwrap();
            let port = configuration["lavalink"]["port"].as_integer().unwrap();
            let password = configuration["lavalink"]["password"].as_str().unwrap();

            let mut lava_client = LavalinkClient::new();
            lava_client.bot_id = bot_id;
            lava_client.password = password.to_string();
            lava_client.host = host.to_string();
            lava_client.port = port.try_into().unwrap();
            lava_client.initialize().await?;

            data.insert::<Lavalink>(Arc::new(RwLock::new(lava_client)));
        }

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
            // obtain all the channels where the bot is allowed to be annoyed on from the db.
            let mut raw_annoyed_channels = sqlx::query!("SELECT channel_id from annoyed_channels")
                .fetch(&pool)
                .boxed();

            // add every channel id from the db to a HashSet.
            let mut annoyed_channels = HashSet::new();
            while let Some(row) = raw_annoyed_channels.try_next().await? {
                annoyed_channels.insert(row.channel_id as u64);
            }

            // Insert the HashSet of annoyed channels to the data.
            data.insert::<AnnoyedChannels>(RwLock::new(annoyed_channels));
        }
    }


    // start listening for events by starting a single shard
    if let Err(why) = client.start_autosharded().await {
        eprintln!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
