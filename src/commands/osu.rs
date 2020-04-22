/// This is the file containing all the osu! related commands.

use crate::{
    ConnectionPool,
    Tokens,
    MY_HELP,
    OSU_GROUP,
};

use serenity::{
    http::Http,
    utils::Colour,
    prelude::Context,
    model::channel::{
        Message,
        ReactionType,
    },
    framework::standard::{
        Args,
        Delimiter,
        CommandResult,
        macros::command,
    },
};

use sqlx;
//use futures::TryStreamExt;
//use futures::stream::StreamExt;

use std::{
    sync::Arc,
    collections::HashSet,
    time::Duration,
};

// Used to format the numbers on the embeds.
use num_format::{
    Locale,
    ToFormattedString,
};

use regex::Regex;
use reqwest;
use serde::Deserialize;

// This is a map to convert the bitwhise number obtained from the api
// To the mods it represents.
// With the short and long versions of the mod names.
//
// This is a module so it can make the compiler not complain about the naming of the constants.
mod bitwhise_mods {
    #![allow(non_upper_case_globals)]
    use bitflags::bitflags;
    
    bitflags! {
        pub struct LongMods: u32 {
            const None           = 0;
            const NoFail         = 1;
            const Easy           = 2;
            const TouchDevice    = 4;
            const Hidden         = 8;
            const HardRock       = 16;
            const SuddenDeath    = 32;
            const DoubleTime     = 64;
            const Relax          = 128;
            const HalfTime       = 256;
            const Nightcore      = 512;
            const Flashlight     = 1024;
            const Autoplay       = 2048;
            const SpunOut        = 4096;
            const Relax2         = 8192;    // Autopilot
            const Perfect        = 16384;
            const Key4           = 32768;
            const Key5           = 65536;
            const Key6           = 131_072;
            const Key7           = 262_144;
            const Key8           = 524_288;
            const FadeIn         = 1_048_576;
            const Random         = 2_097_152;
            const Cinema         = 4_194_304;
            const Target         = 8_388_608;
            const Key9           = 16_777_216;
            const KeyCoop        = 33_554_432;
            const Key1           = 67_108_864;
            const Key3           = 134_217_728;
            const Key2           = 268_435_456;
            const ScoreV2        = 536_870_912;
            const Mirror         = 1_073_741_824;
        }
    }
    bitflags! {
        pub struct ShortMods: u32 {
            const NM = 0;
            const NF = 1;
            const EZ = 2;
            const TD = 4;
            const HD = 8;
            const HR = 16;
            const SD = 32;
            const DT = 64;
            const RX = 128;
            const HT = 256;
            const NC = 512;
            const FL = 1024;
            const AT = 2048;
            const SO = 4096;
            const AP = 8192;
            const PF = 16384;
            const K4 = 32768;
            const K5 = 65536;
            const K6 = 131_072;
            const K7 = 262_144;
            const K8 = 524_288;
            const FI = 1_048_576;
            const RD = 2_097_152;
            const CN = 4_194_304;
            const TP = 8_388_608;
            const K9 = 16_777_216;
            const CO = 33_554_432;
            const K1 = 67_108_864;
            const K3 = 134_217_728;
            const K2 = 268_435_456;
            const V2 = 536_870_912;
            const MR = 1_073_741_824;
        }
    }
}


// JSON Structure of the osu! user API request.
#[derive(Deserialize, PartialEq, Debug)]
struct OsuUserData {
    user_id: String,
    username: String,
    join_date: String,
    country: String,
    count300: Option<String>,
    count100: Option<String>,
    count50: Option<String>,
    playcount: Option<String>,
    ranked_score: Option<String>,
    total_score: Option<String>,
    pp_rank: Option<String>,
    level: Option<String>,
    pp_raw: Option<String>,
    accuracy: Option<String>,
    count_rank_ss: Option<String>,
    count_rank_ssh: Option<String>,
    count_rank_s: Option<String>,
    count_rank_sh: Option<String>,
    count_rank_a: Option<String>,
    total_seconds_played: Option<String>,
    pp_country_rank: Option<String>,
}

// JSON Structure of the osu! scores API request.
#[derive(Deserialize, PartialEq, Debug)]
struct OsuScores {
    score_id: String,
    score: String,
    username: String,
    maxcombo: String,
    count50: String,
    count100: String,
    count300: String,
    countmiss: String,
    countkatu: String,
    countgeki: String,
    perfect: String,
    enabled_mods: String,
    user_id: String,
    date: String,
    rank: String,
    pp: String,
    replay_available: String,
}

// JSON Structure of the osu! user recent plays API request.
#[derive(Deserialize, PartialEq, Debug, Clone)]
struct OsuUserRecentData {
    beatmap_id: String,
    score: String,
    maxcombo: String,
    count50: String,
    count100: String,
    count300: String,
    countmiss: String,
    countkatu: String,
    countgeki: String,
    perfect: String,
    enabled_mods: String,
    user_id: String,
    date: String,
    rank: String,
}

// JSON Structure of the osu! beatmap API request.
#[derive(Deserialize, PartialEq, Debug)]
struct OsuBeatmapData {
    approved: String,
    submit_date: String,
    approved_date: String,
    last_update: String,
    artist: String,
    beatmap_id: String,
    beatmapset_id: String,
    bpm: String,
    creator: String,
    creator_id: String,
    difficultyrating: String,
    diff_aim: String,
    diff_speed: String,
    diff_size: String,
    diff_overall: String,
    diff_approach: String,
    diff_drain: String,
    hit_length: String,
    source: String,
    genre_id: String,
    language_id: String,
    title: String,
    total_length: String,
    version: String,
    file_md5: String,
    mode: String,
    tags: String,
    favourite_count: String,
    rating: String,
    playcount: String,
    passcount: String,
    count_normal: String,
    count_slider: String,
    count_spinner: String,
    max_combo: String,
    download_unavailable: String,
    audio_unavailable: String,
}

// Data Structure of the data obtained on the database.
#[derive(Default, Clone)] // Default is a trait that sets the default value for each type.
struct OsuUserDBData {
    osu_id: i32, // 0
    name: String, // String::new()
    old_name: String,
    mode: Option<i32>, // None
    pp: Option<bool>,
    short_recent: Option<bool>,
}

struct OsuUserRawDBData {
    osu_id: i32, // 0
    osu_username: String, // String::new()
    mode: Option<i32>, // None
    pp: Option<bool>,
    short_recent: Option<bool>,
}

struct OsuUserRawDBDataMinimal {
    osu_username: String, // String::new()
    pp: Option<bool>,
}

// Centralized data, to be used for the events.
#[derive(Default, Clone)]
pub struct EventData {
    user_db_data: Option<OsuUserDBData>,
    user_recent_raw: Option<Vec<OsuUserRecentData>>,
    osu_key: Option<String>,
}

fn pacman(value: &str) -> String {
    let x = value.split('.').nth(1).unwrap()[..3].parse::<u32>().unwrap();
    let tm = x / 50;

    let mut s = "".to_string();

    for _ in 0..tm {
        s += ". ";
    }
    s += "C ";

    for _ in tm..20 {
        s += "o ";
    }
    s
}

fn seconds_to_days(seconds: u64) -> String {
    let days = seconds / 60 / 60 / 24;
    let hours = seconds / 3600 % 24;
    let minutes = seconds % 3600 / 60;
    let sec = seconds % 3600 % 60;

    if days == 0 {
        format!("{}:{}:{}", hours, minutes, sec)
    } else {
        format!("{}D {}:{}:{}", days, hours, minutes, sec)
    }
}

// Calculates the accuracy % from the number of 300's 100's 50's and misses.
async fn acc_math(score_300: f32, score_100: f32, score_50: f32, _miss: f32) -> f32 {
    let mix = score_300  + score_100  + score_50  + _miss ;

    let pcount50 = score_50  / mix * (100.0 / 6.0);
    let pcount100 = score_100  / mix * (100.0 / 3.0);
    let pcount300 = score_300  / mix * 100.0;

    let acc: f32 = pcount50 + pcount100 + pcount300;
    acc
}

// Calculates the progress on the map with the number of notes hit over the number of notes the map has.
async fn progress_math(count_normal: f32, count_slider: f32, count_spinner: f32, score_300: f32, score_100: f32, score_50: f32, _miss: f32) -> f32 {
    let all_the_things = count_normal + count_slider + count_spinner;
    let everything = score_300 + score_100 + score_50 + _miss;
    everything / all_the_things * 100.0
}

// Obtains the long named version of the mods
async fn _get_mods_long(value: u32) -> String {
    use bitwhise_mods::LongMods;

    let mods = LongMods::from_bits_truncate(value);
    format!("{:?}", mods)
}

// Obtains the short named version of the mods
async fn get_mods_short(value: u32) -> String {
    use bitwhise_mods::ShortMods;

    let mods = ShortMods::from_bits_truncate(value);
    format!("{:?}", mods)
}



// This function simply calls the osu! api to get the id of the user from a username.
async fn get_osu_id(name: &str, osu_key: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let resp = get_osu_user(&name, osu_key).await?;

    if !resp.is_empty() {
        let id: i32 = resp[0].user_id.trim().parse()?;
        Ok(id)
    } else {
        Ok(0)
    }
}


// Requests to the api the user data
async fn get_osu_user(name: &str, osu_key: &str) -> Result<Vec<OsuUserData>, Box<dyn std::error::Error>> {
    let re = Regex::new("[^0-9A-Za-z\\[\\]'_ ]").unwrap();
    let mut sanitized_name = re.replace_all(name, "").into_owned();
    sanitized_name = sanitized_name.replace(" ", "%20");

    let url = format!("https://osu.ppy.sh/api/get_user?k={}&u={}&type=string", osu_key, sanitized_name);
    let resp = reqwest::get(&url)
        .await?
        .json::<Vec<OsuUserData>>()
        .await?;
    Ok(resp)
}

// Requests to the api the scores of a map 
async fn get_osu_scores(user_id: i32, user_name: &str, map_id: u64, mode: i32, osu_key: &str) -> Result<Vec<OsuScores>, Box<dyn std::error::Error>> {
    let url = if user_id != 0 {
        format!("https://osu.ppy.sh/api/get_scores?k={}&u={}&b={}&m={}&type=id", osu_key, user_id, map_id, mode)
    } else {
        format!("https://osu.ppy.sh/api/get_scores?k={}&u={}&b={}&m={}&type=string", osu_key, user_name, map_id, mode)
    };

    let resp = reqwest::get(&url)
        .await?
        .json::<Vec<OsuScores>>()
        .await?;
    Ok(resp)
}

// Requests to the api the recent plays of a user
async fn get_osu_user_recent(user_id: i32, osu_key: &str) -> Result<Vec<OsuUserRecentData>, Box<dyn std::error::Error>> {
    let url = format!("https://osu.ppy.sh/api/get_user_recent?k={}&u={}&type=id", osu_key, user_id);
    let resp = reqwest::get(&url)
        .await?
        .json::<Vec<OsuUserRecentData>>()
        .await?;
    Ok(resp)
}

// Requests to the api the data of a beatmap
async fn get_osu_beatmap(beatmap_id: &str, osu_key: &str) -> Result<Vec<OsuBeatmapData>, Box<dyn std::error::Error>> {
    let url = format!("https://osu.ppy.sh/api/get_beatmaps?k={}&b={}", osu_key, beatmap_id);
    let resp = reqwest::get(&url)
        .await?
        .json::<Vec<OsuBeatmapData>>()
        .await?;
    Ok(resp)
}

// Builds the short version of the recent embed and edits the specified message with it.
async fn short_recent_builder(http: Arc<Http>, event_data: &EventData, bot_msg: Message, index: usize) -> Result<(), Box<dyn std::error::Error>> {
    let user_data = event_data.user_db_data.as_ref().unwrap();
    let user_recent_raw = event_data.user_recent_raw.as_ref().unwrap();
    let osu_key = event_data.osu_key.as_ref().unwrap();

    let user_recent = &user_recent_raw[index];
    let user_raw = get_osu_user(&user_data.name, &osu_key).await?;
    let user = &user_raw[0];

    let beatmap_raw = get_osu_beatmap(&user_recent.beatmap_id, &osu_key).await?;
    let beatmap = &beatmap_raw[0];

    let accuracy = acc_math(user_recent.count300.parse()?, user_recent.count100.parse()?, user_recent.count50.parse()?, user_recent.countmiss.parse()?).await;

    let progress: f32 = progress_math(beatmap.count_normal.parse()?, beatmap.count_slider.parse()?, beatmap.count_spinner.parse()?,user_recent.count300.parse()?, user_recent.count100.parse()?, user_recent.count50.parse()?, user_recent.countmiss.parse()?).await;

    let attempts = index;
    let mods: String = get_mods_short(user_recent.enabled_mods.parse()?).await;

    let rating_url = if user_recent.rank == "F" {
        String::from("https://5124.mywire.org/HDD/Downloads/BoneF.png")
    } else {
        format!("https://s.ppy.sh/images/{}.png", user_recent.rank.to_uppercase())
    };

    bot_msg.clone().edit(http.clone(), |m| { // say method doesn't work for the message builder.
        m.content(format!("`{}`", beatmap.beatmap_id));
        m.embed( |e| {
            e.color(Colour::new(user.user_id.parse().unwrap()));
            e.title(format!("{} - {} [**{}**]\nby {}",
                            beatmap.artist, beatmap.title, beatmap.version, beatmap.creator));
            e.url(format!("https://osu.ppy.sh/b/{}", beatmap.beatmap_id));
            e.description(format!("**{}** ┇ **x{} / {}**\n**{:.2}%** ┇ {} - {} - {} - {}\n Recent #{} ━ Progress: {:.2}%",
                                  user_recent.score.parse::<u32>().expect("NaN").to_formatted_string(&Locale::en), user_recent.maxcombo, beatmap.max_combo, accuracy, user_recent.count300, user_recent.count100, user_recent.count50, user_recent.countmiss, attempts + 1, progress));
            e.timestamp(user_recent.date.clone());
            e.thumbnail(format!("https://b.ppy.sh/thumb/{}l.jpg", beatmap.beatmapset_id));
            e.author( |a| {
                a.name(&user.username);
                a.url(format!("https://osu.ppy.sh/u/{}", user.user_id));
                a.icon_url(format!("https://a.ppy.sh/{}", user.user_id));

                a
            });
            if user_data.pp == Some(true) {
                e.footer(|f| {
                    f.text(format!("PP | NEW_PP | {:.4}* | {}", beatmap.difficultyrating, mods));
                    f.icon_url(&rating_url);

                    f
            });
            } else {
                e.footer(|f| {
                    f.text(format!("{:.4}* | {}", beatmap.difficultyrating, mods));
                    f.icon_url(&rating_url);

                    f
                });
            }

            e
        });

        m
    }).await?;
    Ok(())
}



/// Command to configure an osu! user for the bot to know about your prefferences.
/// This supports various keyword parameters, this are:
/// `mode=` To set your osu! gamemode.
/// `pp=` To show or not show any pp related features for your account.
/// `short_recent=` To display the short version of the recent command with less information, but more cozy.
/// 
/// - Everything else that is not keyworded will become your username.
/// - Keyword arguments are not required, they will default to `std, true, true` respectively.
/// 
/// Example usages:
/// `osuc Majorowsky`
/// `osuc nitsuga5124 pp=false short_recent=yes`
/// `osuc [ Frost ] mode=mania pp=yes recent=false`
#[command]
#[aliases("osuc", "config_osu", "configosu", "configureosu", "configo", "setosu", "osuset", "set_osu", "osu_set")]
async fn configure_osu(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {
    let osu_key = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap().clone(); // get the tokens from the global data.
        tokens["osu"].as_str().unwrap().to_string()
    };

    let data = ctx.data.write().await; // set mutable global data.
    let pool = data.get::<ConnectionPool>().unwrap(); // get the database connection from the global data.

    let author_id = *msg.author.id.as_u64() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let data = sqlx::query!("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", // query the SQL to the database.
                            author_id).fetch_optional(pool).boxed().await?; // The arguments on this array will go to the respective calls as $ in the database (arrays start at 1 in this case reeeeee)
    let empty_data: bool;

    let mut user_data = OsuUserDBData::default(); // generate a basic structure with the default values.

    if let Some(row) = data { // if the data is not empty, aka if the user is on the database already
        empty_data = false;
        // Parses the database result into each of the pieces of data on the structure.
        user_data.osu_id = row.osu_id;
        user_data.name = row.osu_username;
        user_data.old_name = user_data.name.clone();
        user_data.mode = row.mode;
        user_data.pp = row.pp;
        user_data.short_recent = row.short_recent;
    } else {
        empty_data = true;
    }
    
    // if there where arguments on the command (aka the user wants to modify a value)
    if !arguments.is_empty() {
        // Transforms the given arguments as a vector
        let args = arguments.raw_quoted().collect::<Vec<&str>>();
        
        // iterates over all the arguments on the list
        for arg in args {
            // if the argument is the keyword PP
            if arg.starts_with("pp=") {
                // Split the argument on the first = and get everything after it.
                let x: &str = arg.split('=').nth(1).unwrap();
                // Match the text after =
                user_data.pp = match x {
                    // if x == "n" || x == "no" ... {user_data.pp = Some(false)}
                    "n" | "no" | "false" | "0" => Some(false),
                    // else Some(true)
                    _ => Some(true)
                }

            // if the argument starts with the keyword short_recent OR recent
            } else if arg.starts_with("short_recent=") || arg.starts_with("recent=") || arg.starts_with("short=") { 
                let x: &str = arg.split('=').nth(1).unwrap();
                user_data.short_recent = match x {
                    "n" | "no" | "false" | "0" => Some(false),
                    _ => Some(true)
                }

            } else if arg.starts_with("mode=") { 
                let x: &str = arg.split('=').nth(1).unwrap();
                user_data.mode = match x {
                    "0" | "std" | "standard" => Some(0),
                    "1" | "taiko" => Some(1),
                    "2" | "ctb" | "catch" => Some(2),
                    "3" | "mania" => Some(3),
                    _ => Some(0)
                }
            
            // this triggers if the argument was not a keyword argument and adds the argument to
            // the username adding a space.
            } else if empty_data {
                user_data.name += arg;
            } else {
                user_data.name = if user_data.name == user_data.old_name {arg.to_string()} else {user_data.name + " " + arg};
            }
        }
    } else if empty_data {
        // sends the help of the command
        let a = Args::new("configure_osu", &[Delimiter::Single(' ')]);
        (MY_HELP.fun)(&mut ctx.clone(), &msg, a, &MY_HELP.options, &[&OSU_GROUP], HashSet::new()).await?;
        return Ok(());
    } else {
        // gets the current configuration of the user
        let current_conf = format!("
Your current configuration:
```User ID: '{}'
Username: '{}'
Mode ID: '{}'
Show PP? '{}'
Short recent? '{}'```",
            user_data.osu_id, user_data.name, user_data.mode.unwrap(), user_data.pp.unwrap(), user_data.short_recent.unwrap()
        );
        // and sends it.
        msg.channel_id.say(&ctx, current_conf).await?;
        return Ok(());
    }

    // calls the get_osu_id function to get the id of the user.
    user_data.osu_id = get_osu_id(&user_data.name, &osu_key).await?;

    // applies the default values in case of being not specified.
    user_data.pp = match &user_data.pp {
        None => Some(true),
        Some(b) => Some(*b),
    };
    user_data.mode = match &user_data.mode {
        None => Some(0),
        Some(b) => Some(*b),
    };
    user_data.short_recent = match &user_data.short_recent {
        None => Some(true),
        Some(b) => Some(*b),
    };

    // Insert a row to the table, but if it conflicts, update the existing one.
    sqlx::query!("INSERT INTO osu_user (osu_id, osu_username, pp, mode, short_recent, discord_id) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (discord_id) DO UPDATE SET osu_id = $1, osu_username = $2, pp = $3, mode = $4, short_recent = $5",
        user_data.osu_id, user_data.name, user_data.pp.unwrap(), user_data.mode.unwrap(), user_data.short_recent.unwrap(), author_id)
        .execute(pool)
        .await?;
   
    // if the id obtained is 0, it means the user doesn't exist.
    if user_data.osu_id == 0 {
        msg.channel_id.say(&ctx, "It looks like your osu ID is 0, Is the Username correct?").await?;
    }

    let current_conf = format!("
Successfully changed your configuration to this:
```User ID: '{}'
Username: '{}'
Mode ID: '{}'
Show PP? '{}'
Short recent? '{}'```",
        user_data.osu_id, user_data.name, user_data.mode.unwrap(), user_data.pp.unwrap(), user_data.short_recent.unwrap()
    );

    msg.channel_id.say(&ctx, current_conf).await?;

    Ok(())
}

/// Shows your osu! profile or of the user specified user.username
///
/// You can use `.osuc` to configure your osu! profile.
/// 
/// Affected parameters for configuration:
/// - PP: To know if the bot should display the PP stadistics.
///
/// Usage:
/// `osu_profile`
/// `osu_profile -GN`
#[command]
#[aliases("oprofile", "oprof", "osuprofile", "osuprof", "osu_prof", "osu_p", "osup", "osu_p", "osu")]
async fn osu_profile(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // Obtains the osu! api key from the "global" data
    let osu_key: &str = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap().clone(); // get the tokens from the global data.
        &tokens["osu"].as_str().unwrap().to_string()
    };
    
    // Obtain the client connection from the "global" data
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().expect("no database connection found"); // get the database connection from the global data.

    let mut username = args.message().replace(" ", "_");
    let author_id = *msg.author.id.as_u64() as i64;

    // get the author_id as a signed 64 bit int, because that's what the database asks for.


    let user_data = {
        if username.is_empty() {
            sqlx::query_as!(OsuUserRawDBDataMinimal, "SELECT osu_username, pp FROM osu_user WHERE discord_id = $1", author_id)
                .fetch_optional(pool)
                .boxed()
                .await?
        } else {
            sqlx::query_as!(OsuUserRawDBDataMinimal, "SELECT osu_username, pp FROM osu_user WHERE osu_username = $1", username)
                .fetch_optional(pool)
                .boxed()
                .await?
        }
    };

    let mut pp = true;

    if let Some(row) = user_data {
        pp = row.pp.unwrap_or(true);
        if username.is_empty() {
            username = row.osu_username;
        }
    } else {
        if username.is_empty() {
            if let Some(m) = msg.member(&ctx).await {
                username = m.display_name().await.to_string();
            } else {
                username = msg.author.name.to_string();
            }
        }
    }

    let resp = get_osu_user(&username, &osu_key).await?;

    let user = if !resp.is_empty() { &resp[0] } else {
        msg.channel_id.say(&ctx, format!("A user with the name of `{}` was not found.", username)).await?;
        return Ok(());
    };

    let country_url = format!("https://raw.githubusercontent.com/stevenrskelton/flag-icon/master/png/75/country-squared/{}.png", &user.country.to_lowercase());
    
    if let None = user.total_score {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.timestamp(user.join_date.clone());
                e.thumbnail(format!("https://a.ppy.sh/{}", &user.user_id));
                e.author(|a| {
                    a.name(&user.username);
                    a.url(format!("https://osu.ppy.sh/u/{}", &user.user_id));
                    a.icon_url(country_url)
                })
            })
        }).await?;

    } else {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.color(Colour::new(user.user_id.parse::<u32>().expect("The ID was too large for u32 :thinking:")));
                e.timestamp(user.join_date.clone());
                e.author(|a| {
                    a.name(&user.username);
                    a.url(format!("https://osu.ppy.sh/u/{}", &user.user_id));
                    a.icon_url(country_url)
                });
                e.image(format!("https://a.ppy.sh/{}", &user.user_id));

                e.description({
                    let mut s = format!("
                            **{} -- {} -- {}** == 300/100/50
                            **{:.2}% -- {}** Plays
                            Total = **{}** ┇ Ranked = **{}**
                            Played **{}** seconds or: **{:?}**
                            **L{}** > Next level: `{}`
                        ",

                        &user.count300.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        &user.count100.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        &user.count50.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        &user.accuracy.as_ref().unwrap().parse::<f32>().expect("NAN"),
                        &user.playcount.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        &user.total_score.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        &user.ranked_score.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        &user.total_seconds_played.as_ref().unwrap().parse::<u64>().expect("NaN").to_formatted_string(&Locale::en),
                        seconds_to_days(user.total_seconds_played.as_ref().unwrap().parse::<u64>().expect("NaN")),
                        user.level.as_ref().unwrap().parse::<f32>().expect("NaN") as u8,
                        pacman(&user.level.as_ref().unwrap()),
                    );
                    if pp {
                        s += &format!("Global: #**{}** | Country: #**{}**",
                            &user.pp_rank.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                            &user.pp_country_rank.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        );
                    } s
                });
                e.footer(|f| {
                    if pp {
                        f.text(format!("PP:{} | SSH:{} | SS:{} | SH:{} | S:{}",
                               &user.pp_raw.as_ref().unwrap(),
                               &user.count_rank_ssh.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                               &user.count_rank_ss.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                               &user.count_rank_sh.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                               &user.count_rank_s.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        ))
                    } else {
                        f.text(format!("SSH:{} | SS:{} | SH:{} | S:{}",
                               &user.count_rank_ssh.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                               &user.count_rank_ss.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                               &user.count_rank_sh.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                               &user.count_rank_s.as_ref().unwrap().parse::<u128>().expect("NaN").to_formatted_string(&Locale::en),
                        ))
                    }
                })
            })
        }).await?;
    }

    Ok(())
}

/// Obtains your score on the specified beatmap id
/// The number that's sent along with the recent command is the beatmap id of it.
///
/// You can use `.osuc` to configure your osu! profile.
/// 
/// Affected parameters for configuration:
/// - Mode: To specify the gamemode the score was on.
/// - PP: To know if the bot should display the PP stadistics.
///
/// Usage: `score 124217`
#[command]
#[aliases("compare")]
async fn score(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let bmap_id = match args.parse::<u64>() {
        Err(_) => {
            msg.reply(&ctx, "An invalid id was provided").await?;
            return Ok(());
        },
        Ok(x) => x,
    };

    // Obtains the osu! api key from the "global" data
    let osu_key = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap().clone(); // get the tokens from the global data.
        tokens["osu"].as_str().unwrap().to_string()
    };
    
    // Obtain the client connection from the "global" data
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().expect("no database connection found"); // get the database connection from the global data.

    // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let author_id = *msg.author.id.as_u64() as i64;

    let user_data = sqlx::query_as!(OsuUserRawDBData, "SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", author_id)
        .fetch_optional(pool)
        .boxed()
        .await?;

    let mut pp = true;
    let mut mode = 0;
    let username;
    let mut osu_id = 0;

    if let Some(row) = user_data {
        pp = row.pp.unwrap_or(true);
        mode = row.mode.unwrap_or(0);
        username = row.osu_username;
        osu_id = row.osu_id;
    } else {
        if let Some(m) = msg.member(&ctx).await {
            username = m.display_name().await.to_string();
        } else {
            username = msg.author.name.to_string();
        }
    }

    let score = get_osu_scores(osu_id, &username, bmap_id, mode, &osu_key).await?;

    let s = if let Some(x) = score.get(0) { x } else {
        msg.channel_id.say(&ctx, format!("The user `{}` does not have any scores on the specified map.", &username)).await?;
        return Ok(());
    };

    let beatmap_raw = get_osu_beatmap(&bmap_id.to_string(), &osu_key).await?;
    let beatmap = &beatmap_raw[0];

    let accuracy = acc_math(s.count300.parse()?, s.count100.parse()?, s.count50.parse()?, s.countmiss.parse()?).await;

    let mods: String = get_mods_short(s.enabled_mods.parse()?).await;

    let rating_url = format!("https://s.ppy.sh/images/{}.png", s.rank.to_uppercase());

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.color(Colour::new(osu_id as u32));

            e.title(format!("{} - {} [**{}**]\nby {}", beatmap.artist, beatmap.title, beatmap.version, beatmap.creator));
            e.url(format!("https://osu.ppy.sh/b/{}", beatmap.beatmap_id));

            e.description(format!("**{}** ┇ **x{} / {}**\n**{:.2}%** ┇ {} - {} - {} - {}",
                s.score.parse::<u32>().expect("NaN").to_formatted_string(&Locale::en),
                s.maxcombo, beatmap.max_combo, accuracy, s.count300, s.count100, s.count50, s.countmiss));
            e.timestamp(s.date.clone());
            e.thumbnail(format!("https://b.ppy.sh/thumb/{}l.jpg", beatmap.beatmapset_id));

            e.author( |a| {
                a.name(username);
                if osu_id == 0 {
                    a.icon_url(format!("https://a.ppy.sh/{}", osu_id + 1));
                } else {
                    a.icon_url(format!("https://a.ppy.sh/{}", osu_id));
                    a.url(format!("https://osu.ppy.sh/u/{}", osu_id));
                }

                a
            });

            e.footer(|f| {
                if pp {
                    f.text(format!("{}pp | {:.4}* | {}", &s.pp, beatmap.difficultyrating, mods));
                } else {
                    f.text(format!("{:.4}* | {}", beatmap.difficultyrating, mods));
                }
                f.icon_url(&rating_url)
            });
            e
        })
    }).await?;

    Ok(())
}

/// Command to show the most recent osu! play.
/// - Due to api limits, this will only work on maps with leaderboard.
/// - This command is able to show failed plays, and show the percentage of progress.
///
/// To use this command, first configure your osu! profile with `.osuc`
/// Affected parameters for configuration:
/// - Mode: To specify the gamemode the play was on.
/// - PP: To know if the bot should display the PP stadistics of the play.
/// - Short Recent: To display the short version of recent instead of the long one.
/// (Currently only short exists.)
///
/// You can also invoke the command specifying a username.
/// Usage:
/// `recent`
/// `recent [ Frost ]`
/// `recent nitsuga5124`
#[command]
#[aliases("rs", "rc")]
async fn recent(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {
    let mut arg_user = String::from("");
    if !arguments.is_empty() {
        let args = arguments.raw_quoted().collect::<Vec<&str>>();
        for i in args {
            arg_user += &format!("{} ", i).to_owned()[..];
        }
        arg_user.pop();
    }

    // Obtains the osu! api key from the "global" data
    let osu_key = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap().clone(); // get the tokens from the global data.
        tokens["osu"].as_str().unwrap().to_string()
    };
    
    // Obtain the client connection from the "global" data.
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().expect("no database connection found"); // get the database connection from the global data.

    let mut user_data = OsuUserDBData::default(); // generate a basic structure with the default values.


    let data = if arg_user == "" {
        let author_id = *msg.author.id.as_u64() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
        arg_user = msg.author.name.clone();
        sqlx::query_as!(OsuUserRawDBData, "SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", // query the SQL to the database.
                            author_id).fetch_optional(pool).boxed().await? // The arguments on this array will go to the respective calls as $ in the database (arrays start at 1 in this case reeeeee)
    } else {
        sqlx::query_as!(OsuUserRawDBData, "SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE osu_username = $1", // query the SQL to the database.
                            arg_user).fetch_optional(pool).boxed().await?
    };

    if let Some(x) = data { // if the data is not empty, aka if the user is on the database already
        // Parses the database result into each of the pieces of data on the structure.
            user_data.osu_id = x.osu_id;
            user_data.name = x.osu_username;
            user_data.mode = x.mode;
            user_data.pp = x.pp;
            user_data.short_recent = x.short_recent;
    } else {
        if arg_user == "" {
            msg.channel_id.say(&ctx, "It looks like you don't have a configured osu! username, consider configuring one with `n!osuc`").await?;
        }
        user_data.name = arg_user;
        user_data.mode = Some(0);
        user_data.pp = Some(true);
        user_data.short_recent = Some(true);
    }

    if user_data.osu_id == 0 {
        let user_id = get_osu_id(&user_data.name, &osu_key).await?;
        if user_id == 0 {
            msg.channel_id.say(&ctx, format!("Could not find any osu! user with the name of '{}'", user_data.name)).await?;
            return Ok(());
        } else {
            user_data.osu_id = user_id;
        }
    }
    let bot_msg = msg.channel_id.say(&ctx, format!("Obtaining **{}** recent data", user_data.name)).await?;

    let user_recent_raw = get_osu_user_recent(user_data.osu_id, &osu_key).await?;

    if user_recent_raw.is_empty() {
        bot_msg.clone().edit(&ctx, |m| {
            m.content(format!("The user **{}** has not played in the last 24 hours.", user_data.name));
            m
        }).await?;
        return Ok(());
    }

    // Group all the needed data to EventData
    let mut event_data = EventData::default();
    event_data.user_db_data = Some(user_data);
    event_data.user_recent_raw = Some(user_recent_raw.clone());
    event_data.osu_key = Some(osu_key);

    let mut page = 0;

    // Build the initial recent embed
    short_recent_builder(ctx.http.clone(), &event_data, bot_msg.clone(), page).await?;

    // Add left and right reactions, to make the life easier for the user using the event.

    let left = ReactionType::Unicode(String::from("⬅️"));
    let right = ReactionType::Unicode(String::from("➡️"));

    bot_msg.react(&ctx, left).await?;
    bot_msg.react(&ctx, right).await?;

    loop {
        if let Some(reaction) = &bot_msg.await_reaction(&ctx).timeout(Duration::from_secs(20)).await {
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "⬅️" => { 
                    if page != 0 {
                        page -= 1;
                    }
                },
                "➡️" => { 
                    if page != user_recent_raw.len() - 1 {
                        page += 1;
                    }
                },
                _ => (),
            }

            short_recent_builder(ctx.http.clone(), &event_data, bot_msg.clone(), page).await?;
            reaction.as_inner_ref().delete(&ctx).await?;
        } else {
            bot_msg.delete_reactions(&ctx).await?;
            break
        };
    }

    Ok(())
}
