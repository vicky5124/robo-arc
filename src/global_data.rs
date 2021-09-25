use crate::{notifications::TwitchStreamData, Booru, ConfigurationData};

use std::{collections::{HashSet, HashMap}, sync::Arc, time::Instant};

use tokio::sync::{Mutex, RwLock};

use serenity::{client::bridge::gateway::ShardManager, prelude::TypeMapKey, model::id::GuildId};
use songbird::Call;

use darkredis::ConnectionPool as RedisPool;
use lavalink_rs::LavalinkClient;
use reqwest::Client as ReqwestClient;
use sqlx::PgPool; // PostgreSQL Pool Structure

// Defining the structures to be used for "global" data
// this data is not really global, it's just shared with Context.data
pub struct ShardManagerContainer; // Shard manager to use for the latency.
pub struct DatabasePool; // A pool of connections to the database.
pub struct CachePool; // The connection to the redis cache database.
pub struct Tokens; // For the configuration found on "config.toml"
pub struct AnnoyedChannels; // This is a HashSet of all the channels the bot is allowed to be annoyed on.
pub struct BooruList; // This is a HashSet of all the boorus found on "boorus.json"
pub struct BooruCommands; // This is a HashSet of all the commands/aliases found on "boorus.json"
pub struct Lavalink; //  This is the struct for the lavalink client.
pub struct SongbirdCalls; //  This is the struct to store the current songbird connections.
pub struct SentTwitchStreams; //  This is the struct for the stream data that has already been sent.
pub struct Uptime; //  This is for the startup time of the bot.
pub struct OsuHttpClient; // This is the HTTP client to comunicate with osu! API v2.

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for DatabasePool {
    type Value = PgPool;
}

impl TypeMapKey for CachePool {
    type Value = RedisPool;
}

impl TypeMapKey for Tokens {
    type Value = Arc<ConfigurationData>;
}

impl TypeMapKey for AnnoyedChannels {
    type Value = Arc<RwLock<HashSet<u64>>>;
}

impl TypeMapKey for BooruList {
    type Value = Arc<Vec<Booru>>;
}

impl TypeMapKey for BooruCommands {
    type Value = Arc<HashSet<String>>;
}

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

impl TypeMapKey for SongbirdCalls {
    type Value = Arc<RwLock<HashMap<GuildId, Arc<Mutex<Call>>>>>;
}


impl TypeMapKey for SentTwitchStreams {
    type Value = Arc<RwLock<Vec<TwitchStreamData>>>;
}

impl TypeMapKey for Uptime {
    type Value = Arc<Instant>;
}

impl TypeMapKey for OsuHttpClient {
    type Value = Arc<RwLock<ReqwestClient>>;
}
