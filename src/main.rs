#![feature(core_intrinsics)]
#![feature(async_closure)]
#![feature(once_cell)]
#![type_length_limit = "1331829"]
#![allow(clippy::unusual_byte_groupings)]
//! This is a discord bot made with `serenity.rs` as a Rust learning project.
//! If you see a lot of different ways to do the same thing, specially with error handling,
//! this is indentional, as it helps me remember the concepts that rust provides, so they can be
//! used in the future for whatever they could be needed.
//!
//! This is lisenced with the copyleft license Mozilla Public License Version 2.0

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;

pub mod commands; // Load the commands module
pub mod config;
pub mod events;
pub mod framework;
pub mod framework_methods;
pub mod global_data;
pub mod logging;
pub mod notifications;
pub mod utils; // Load the utils module

use crate::config::*;
use crate::events::*;
use crate::framework::*;
use crate::framework_methods::*;
use crate::global_data::*;

use utils::database::*; // Obtain the get_database function from the utilities. // Obtain the capitalize_first function from the utilities.

use std::{
    collections::HashSet, // Low cost indexable lists.
    // For saving / reading files
    fs::File,
    io::prelude::*,
    //
    // For having refferences between threads
    sync::Arc,
    time::Instant,
};

use tokio::sync::Mutex;

use tracing::Level;

use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
//use tracing_futures::Instrument;

use lavalink_rs::LavalinkClient;
use reqwest::header;
use songbird::SerenityInit;

// Serenity! what make's the bot function. Discord API wrapper.
use serenity::{
    client::{
        bridge::gateway::GatewayIntents,
        ClientBuilder, // To create a client that runs eveyrthing.
    },
    framework::standard::StandardFramework,
    http::Http,
    model::id::UserId,
    prelude::RwLock,
};

// Because "boorus.json" boorus key is a list of values.
#[derive(Debug, Deserialize)]
struct BooruRaw {
    boorus: Vec<Booru>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsuTokenSend {
    pub client_id: u16,
    pub client_secret: String,
    pub grant_type: String,
    pub scope: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsuTokenRecv {
    pub token_type: String,
    pub expires_in: u32,
    pub access_token: String,
}

// The main function!
// Here's where everything starts.
// This main function is a little special, as it returns Result
// which allows ? to be used for error handling.
#[tokio::main(worker_threads = 8)]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Opens the config.toml file and reads it's content
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // gets the data from the config.toml file
    let configuration = toml::from_str::<ConfigurationData>(&contents).unwrap();

    if configuration.enable_tracing {
        LogTracer::init()?;

        // obtains the tracing level from the config
        let level = match configuration.trace_level.as_str() {
            "error" => Level::ERROR,
            "warn" => Level::WARN,
            "info" => Level::INFO,
            "debug" => Level::DEBUG,
            "trace" => Level::TRACE,
            _ => Level::INFO,
        };

        info!("Tracer initialized with level {}.", level);

        if dotenv::dotenv().is_ok() {
            let subscriber = FmtSubscriber::builder()
                .with_env_filter(EnvFilter::from_default_env())
                .finish();
            tracing::subscriber::set_global_default(subscriber)?;
        } else {
            let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
            tracing::subscriber::set_global_default(subscriber)?;
        };

        info!("Subscriber initialized.");
    }

    // obtains the discord token from the config
    let bot_token = configuration.discord.to_string();
    // Defines a client with the token obtained from the config.toml file.
    // This also starts up the Event Handler structure defined earlier.

    let http = Http::new_with_token(&bot_token);

    // Obtains and defines the owner/owners of the Bot Application
    // and the bot id.
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Time to configure the Command Framework!
    // This is what allows for easier and faster commaands.
    let std_framework = StandardFramework::new() // Create a new framework
        .configure(|c| {
            c.prefix("") // Remove the default prefix.
                .on_mention(Some(bot_id)) // Add a bot mention as a prefix.
                .dynamic_prefix(dynamic_prefix)
                .with_whitespace(true) // Allow a whitespace between the prefix and the command name.
                .owners(owners) // Defines the owners, this can be later used to make owner specific commands.
                .case_insensitivity(true) // Makes the prefix and command be case insensitive.
                .blocked_users(vec![UserId(135423120268984330)].into_iter().collect())
        })
        .on_dispatch_error(on_dispatch_error)
        .unrecognised_command(unrecognised_command)
        .before(before)
        .after(after)
        .bucket("permanent_ban", |b| b.delay(30).time_span(30).limit(1))
        .await
        .group(&META_GROUP) // Load `Meta` command group
        .group(&FUN_GROUP) // Load `Fun` command group
        .group(&MUSIC_GROUP) // Load `music` command group
        .group(&MOD_GROUP) // Load `moderation` command group
        .group(&OSU_GROUP) // Load `osu!` command group
        .group(&NEWOSU_GROUP) // Load `new osu!` command group
        .group(&SANKAKU_GROUP) // Load `SankakuComplex` command group
        .group(&ALLBOORUS_GROUP) // Load `Boorus` command group
        .group(&IMAGEMANIPULATION_GROUP) // Load `image manipulaiton` command group
        .group(&GAMES_GROUP) // Load `games` command group
        .group(&SERENITYDOCS_GROUP) // Load `serenity_docs` command group
        .group(&CONFIGURATION_GROUP) // Load `Configuration` command group
        .help(&MY_HELP); // Load the custom help command.

    let mut client = ClientBuilder::new(&bot_token)
        .event_handler(Handler {
            run_loops: Mutex::new(true),
        })
        .raw_event_handler(logging::events::RawHandler)
        .framework(std_framework)
        .register_songbird()
        .application_id(bot_id.0)
        .intents({
            let mut intents = GatewayIntents::all();
            //intents.remove(GatewayIntents::GUILD_PRESENCES);
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

        // Add the databases connection pools to the data.
        let pg_pool = obtain_postgres_pool().await?;
        data.insert::<DatabasePool>(pg_pool.clone());

        let redis_pool = obtain_redis_pool().await?;
        data.insert::<CachePool>(redis_pool);

        // Add the shard manager to the data.
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));

        // Add the tokens to the data.
        data.insert::<Tokens>(Arc::new(configuration.clone()));

        // Add the sent streams.
        data.insert::<SentTwitchStreams>(Arc::new(RwLock::new(Vec::new())));

        data.insert::<Uptime>(Arc::new(Instant::now()));

        {
            // T 0 D 0: get the real shard amount.
            let host = configuration.lavalink.host;
            let port = configuration.lavalink.port;
            let password = configuration.lavalink.password;

            let lava_client = LavalinkClient::builder(bot_id.0)
                .set_host(host.to_string())
                .set_password(password.to_string())
                .set_port(port)
                .build(LavalinkHandler)
                .await?;

            data.insert::<Lavalink>(lava_client);
        }

        {
            // Open the boorus.json file
            let mut file = File::open("boorus.json").unwrap();
            // read the content of the file to a string.
            let mut raw_data = String::new();
            file.read_to_string(&mut raw_data).unwrap();

            // Serialize the json string into a BooruRaw struct.
            let boorus: BooruRaw =
                serde_json::from_str(raw_data.as_str()).expect("JSON was not well-formatted");

            // Add every command on the data to a HashSet
            let mut all_names = HashSet::new();
            for boorus_list in boorus.boorus.iter() {
                for booru in boorus_list.names.iter() {
                    all_names.insert(booru.to_owned());
                }
            }

            // Add the json file struct and the HashSet of all the commands to the data.
            data.insert::<BooruList>(Arc::new(boorus.boorus));
            data.insert::<BooruCommands>(Arc::new(all_names));
        }

        {
            let annoyed_channels = {
                // obtain all the channels where the bot is allowed to be annoyed on from the db.
                let raw_annoyed_channels = sqlx::query!("SELECT channel_id from annoyed_channels")
                    .fetch_all(&pg_pool)
                    .await?;

                // add every channel id from the db to a HashSet.
                let mut annoyed_channels = HashSet::new();
                for row in raw_annoyed_channels {
                    annoyed_channels.insert(row.channel_id as u64);
                }

                annoyed_channels
            };

            // Insert the HashSet of annoyed channels to the data.
            data.insert::<AnnoyedChannels>(Arc::new(RwLock::new(annoyed_channels)));
        }

        {
            let base_client = reqwest::Client::new();

            let send_data = OsuTokenSend {
                client_id: configuration.osu.client_id,
                client_secret: configuration.osu.client_secret,
                grant_type: "client_credentials".to_string(),
                scope: "public".to_string(),
            };

            let res = base_client
                .post("https://osu.ppy.sh/oauth/token")
                .json(&send_data)
                .send()
                .await?
                .json::<OsuTokenRecv>()
                .await?;

            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::AUTHORIZATION,
                format!("{} {}", res.token_type, res.access_token)
                    .parse()
                    .unwrap(),
            );

            let client = reqwest::Client::builder()
                .default_headers(headers)
                .build()?;

            data.insert::<OsuHttpClient>(Arc::new(RwLock::new(client)));
        }
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start_autosharded().await {
        eprintln!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
