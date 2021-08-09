//! This is the file containing all the osu! related commands.

use crate::{
    global_data::{DatabasePool, Tokens},
    utils::basic_functions::{pacman, seconds_to_days},
    utils::osu::*,
    MY_HELP, OSU_GROUP,
};

use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult, Delimiter},
    http::Http,
    model::channel::{Message, ReactionType},
    prelude::Context,
    utils::Colour,
};

//use futures::TryStreamExt;
//use futures::stream::StreamExt;

use std::{collections::HashSet, sync::Arc, time::Duration};

// Used to format the numbers on the embeds.
use num_format::{Locale, ToFormattedString};

use reqwest::Url;
use serde::Deserialize;

use clap::{App, Arg};

#[derive(Default, Debug)]
struct OsuData {
    id: i32,
    username: String,
    pp: bool,
}

// JSON Structure of the osu! user API request.
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
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
    pp: Option<String>,
    replay_available: String,
}

// JSON Structure of the osu! user recent plays API request.
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Deserialize, Debug)]
struct OsuBeatmapData {
    approved: String,
    submit_date: String,
    approved_date: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct OsuUserBest {
    beatmap_id: String,
    score_id: String,
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
    pp: String,
    replay_available: String,
}

// Data Structure of the data obtained on the database.
#[derive(Default, Clone)] // Default is a trait that sets the default value for each type.
struct OsuUserDBData {
    osu_id: i32,  // 0
    name: String, // String::new()
    old_name: String,
    mode: Option<i32>, // None
    pp: Option<bool>,
    short_recent: Option<bool>,
}

struct OsuUserRawDBData {
    osu_id: i32,          // 0
    osu_username: String, // String::new()
    mode: Option<i32>,    // None
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

// Calculates the accuracy % from the number of 300's 100's 50's and misses.
async fn acc_math(score_300: f32, score_100: f32, score_50: f32, _miss: f32) -> f32 {
    let mix = score_300 + score_100 + score_50 + _miss;

    let pcount50 = score_50 / mix * (100.0 / 6.0);
    let pcount100 = score_100 / mix * (100.0 / 3.0);
    let pcount300 = score_300 / mix * 100.0;

    let acc: f32 = pcount50 + pcount100 + pcount300;
    acc
}

// Calculates the progress on the map with the number of notes hit over the number of notes the map has.
pub fn progress_math(
    count_normal: f32,
    count_slider: f32,
    count_spinner: f32,
    score_300: f32,
    score_100: f32,
    score_50: f32,
    _miss: f32,
) -> f32 {
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
async fn get_osu_id(
    name: &str,
    osu_key: &str,
) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let resp = get_osu_user(&name, osu_key).await?;

    if !resp.is_empty() {
        let id: i32 = resp[0].user_id.trim().parse()?;
        Ok(id)
    } else {
        Ok(0)
    }
}

// Requests to the api the user data
async fn get_osu_user(
    name: &str,
    osu_key: &str,
) -> Result<Vec<OsuUserData>, Box<dyn std::error::Error + Send + Sync>> {
    let url = Url::parse_with_params(
        "https://osu.ppy.sh/api/get_user",
        &[("k", osu_key), ("u", name), ("type", "string")],
    )?;
    let resp = reqwest::get(url).await?.json::<Vec<OsuUserData>>().await?;
    Ok(resp)
}

// Requests to the api the user data
async fn get_osu_username(
    id: &i32,
    osu_key: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = Url::parse_with_params(
        "https://osu.ppy.sh/api/get_user",
        &[("k", osu_key), ("u", &id.to_string()), ("type", "id")],
    )?;
    let resp = reqwest::get(url).await?.json::<Vec<OsuUserData>>().await?;

    if let Some(x) = resp.get(0) {
        Ok(x.username.to_string())
    } else {
        Ok(String::new())
    }
}

// Requests to the api the scores of a map
async fn get_osu_scores(
    user_id: i32,
    user_name: &str,
    map_id: u64,
    mode: i32,
    osu_key: &str,
) -> Result<Vec<OsuScores>, Box<dyn std::error::Error + Send + Sync>> {
    let url = if user_id != 0 {
        Url::parse_with_params(
            "https://osu.ppy.sh/api/get_scores",
            &[
                ("k", osu_key),
                ("u", &user_id.to_string()),
                ("b", &map_id.to_string()),
                ("m", &mode.to_string()),
                ("type", "id"),
            ],
        )?
    } else {
        Url::parse_with_params(
            "https://osu.ppy.sh/api/get_scores",
            &[
                ("k", osu_key),
                ("u", user_name),
                ("b", &map_id.to_string()),
                ("m", &mode.to_string()),
                ("type", "string"),
            ],
        )?
    };

    let resp = reqwest::get(url).await?.json::<Vec<OsuScores>>().await?;
    Ok(resp)
}

// Requests to the api the recent plays of a user
async fn get_osu_user_recent(
    user_id: i32,
    osu_key: &str,
) -> Result<Vec<OsuUserRecentData>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://osu.ppy.sh/api/get_user_recent?k={}&u={}&type=id&limit=50",
        osu_key, user_id
    );
    let resp = reqwest::get(&url)
        .await?
        .json::<Vec<OsuUserRecentData>>()
        .await?;
    Ok(resp)
}

// Requests to the api the data of a beatmap
async fn get_osu_beatmap(
    beatmap_id: &str,
    osu_key: &str,
) -> Result<Vec<OsuBeatmapData>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://osu.ppy.sh/api/get_beatmaps?k={}&b={}",
        osu_key, beatmap_id
    );
    let resp = reqwest::get(&url)
        .await?
        .json::<Vec<OsuBeatmapData>>()
        .await?;

    Ok(resp)
}

// Requests to the api the top plays or best plays of a user (by pp weight).
async fn get_osu_user_best(
    user_id: &i32,
    osu_key: &str,
) -> Result<Vec<OsuUserBest>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://osu.ppy.sh/api/get_user_best?k={}&u={}&type=id&limit=100&m=0",
        osu_key, user_id
    );
    let resp = reqwest::get(&url).await?.json::<Vec<OsuUserBest>>().await?;
    Ok(resp)
}

// Builds the short version of the recent embed and edits the specified message with it.
async fn short_recent_builder(
    http: Arc<Http>,
    event_data: &EventData,
    bot_msg: Message,
    index: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_data = event_data.user_db_data.as_ref().unwrap();
    let user_recent_raw = event_data.user_recent_raw.as_ref().unwrap();
    let osu_key = event_data.osu_key.as_ref().unwrap();

    let user_recent = &user_recent_raw[index];
    let user_raw = get_osu_user(&user_data.name, &osu_key).await?;
    let user = &user_raw[0];

    let beatmap_raw = get_osu_beatmap(&user_recent.beatmap_id, &osu_key).await?;
    let beatmap = &beatmap_raw[0];

    let accuracy = acc_math(
        user_recent.count300.parse()?,
        user_recent.count100.parse()?,
        user_recent.count50.parse()?,
        user_recent.countmiss.parse()?,
    )
    .await;

    let progress: f32 = progress_math(
        beatmap.count_normal.parse()?,
        beatmap.count_slider.parse()?,
        beatmap.count_spinner.parse()?,
        user_recent.count300.parse()?,
        user_recent.count100.parse()?,
        user_recent.count50.parse()?,
        user_recent.countmiss.parse()?,
    );

    let attempts = index;
    let mods: String = get_mods_short(user_recent.enabled_mods.parse()?).await;

    let rating_url = if user_recent.rank == "F" {
        String::from("https://5124.mywire.org/HDD/Downloads/BoneF.png")
    } else {
        format!(
            "https://s.ppy.sh/images/{}.png",
            user_recent.rank.to_uppercase()
        )
    };

    bot_msg.clone().edit(http.clone(), |m| { // say method doesn't work for the message builder.
        m.content(format!("Beatmap ID: `{}` - BTW, Try out `new_recent`!", beatmap.beatmap_id));
        m.embed( |e| {
            e.color(Colour::new({
                let colour = user.user_id.parse().unwrap();
                if colour > 16777215 {
                    15227880
                } else {
                    colour
                }
            }));
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
                    let mut pp = PpCalculation::default();

                    pp.score_mods = {
                        let split = mods.split(" | ");
                        split.map(|i| {
                            if i != "NM" || i != "V2" {
                                i.to_string()
                            } else {
                                String::new()
                            }
                        }).collect::<Vec<String>>()
                    };
                    pp.score_max_combo = user_recent.maxcombo.parse::<f64>().unwrap();
                    pp.score_great = user_recent.count300.parse::<f64>().unwrap();
                    pp.score_good = user_recent.count100.parse::<f64>().unwrap();
                    pp.score_meh = user_recent.count50.parse::<f64>().unwrap();
                    pp.score_miss = user_recent.countmiss.parse::<f64>().unwrap();

                    pp.map_aim_strain = beatmap.diff_aim.parse::<f64>().unwrap();
                    pp.map_speed_strain = beatmap.diff_speed.parse::<f64>().unwrap();

                    pp.map_max_combo = beatmap.max_combo.parse::<f64>().unwrap();
                    pp.map_ar = beatmap.diff_approach.parse::<f64>().unwrap();
                    pp.map_od = beatmap.diff_overall.parse::<f64>().unwrap();

                    pp.map_circles = beatmap.count_normal.parse::<f64>().unwrap();
                    pp.map_sliders = beatmap.count_slider.parse::<f64>().unwrap();
                    pp.map_spinners = beatmap.count_spinner.parse::<f64>().unwrap();

                    pp.progress = progress as f64;

                    let v1_pp = pp.calculate();
                    pp.score_mods.push("V2".to_string());
                    let v2_pp = pp.calculate();

                    f.text(format!("{:.2}pp | {:.2} sv2 pp | {:.4}* | {}", v1_pp, v2_pp, beatmap.difficultyrating, mods));
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
/// `osuc vicky5124 pp=false short_recent=yes`
/// `osuc [ Frost ] mode=mania pp=yes recent=false`
#[command]
#[aliases(
    "osuc",
    "config_osu",
    "configosu",
    "configureosu",
    "configo",
    "setosu",
    "osuset",
    "set_osu",
    "osu_set"
)]
async fn configure_osu(ctx: &Context, msg: &Message, arguments: Args) -> CommandResult {
    let osu_key = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap().clone(); // get the tokens from the global data.
        tokens.old_osu.to_string()
    };

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let author_id = *msg.author.id.as_u64() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let data =
        sqlx::query!("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", // query the SQL to the database.
            author_id)
        .fetch_optional(&pool)
        .boxed()
        .await?; // The arguments on this array will go to the respective calls as $ in the database (arrays start at 1 in this case reeeeee)
    let empty_data: bool;

    let mut user_data = OsuUserDBData::default(); // generate a basic structure with the default values.

    if let Some(row) = data {
        // if the data is not empty, aka if the user is on the database already
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
                    _ => Some(true),
                }

                // if the argument starts with the keyword short_recent OR recent
            } else if arg.starts_with("short_recent=")
                || arg.starts_with("recent=")
                || arg.starts_with("short=")
            {
                let x: &str = arg.split('=').nth(1).unwrap();
                user_data.short_recent = match x {
                    "n" | "no" | "false" | "0" => Some(false),
                    _ => Some(true),
                }
            } else if arg.starts_with("mode=") {
                let x: &str = arg.split('=').nth(1).unwrap();
                user_data.mode = match x {
                    "0" | "std" | "standard" => Some(0),
                    "1" | "taiko" => Some(1),
                    "2" | "ctb" | "catch" => Some(2),
                    "3" | "mania" => Some(3),
                    _ => Some(0),
                }

                // this triggers if the argument was not a keyword argument and adds the argument to
                // the username adding a space.
            } else if empty_data {
                user_data.name += arg;
            } else {
                user_data.name = if user_data.name == user_data.old_name {
                    arg.to_string()
                } else {
                    user_data.name + " " + arg
                };
            }
        }
    } else if empty_data {
        // sends the help of the command
        let a = Args::new("configure_osu", &[Delimiter::Single(' ')]);
        (MY_HELP.fun)(ctx, msg, a, &MY_HELP.options, &[&OSU_GROUP], HashSet::new()).await?;
        return Ok(());
    } else {
        // gets the current configuration of the user
        let current_conf = format!(
            "
            **User ID**: {}
            **Username**: {}
            **Mode ID**: {}
            **Show PP**? {}
            **Short recent**? {}",
            user_data.osu_id,
            user_data.name,
            user_data.mode.unwrap(),
            user_data.pp.unwrap(),
            user_data.short_recent.unwrap()
        );

        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Your current configuration:");
                    e.description(current_conf)
                })
            })
            .await?;

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
        .execute(&pool)
        .await?;

    // if the id obtained is 0, it means the user doesn't exist.
    if user_data.osu_id == 0 {
        msg.channel_id
            .say(
                ctx,
                "It looks like your osu ID is 0, Is the Username correct?",
            )
            .await?;
    }

    let current_conf = format!(
        "
        **User ID**: {}
        **Username**: {}
        **Mode ID**: {}
        **Show PP**? {}
        **Short recent**? {}",
        user_data.osu_id,
        user_data.name,
        user_data.mode.unwrap(),
        user_data.pp.unwrap(),
        user_data.short_recent.unwrap()
    );

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Successfully changed your configuration!");
                e.description(current_conf)
            })
        })
        .await?;

    Ok(())
}

/// Shows your osu! profile or of the user specified user.username
///
/// You can use `osuc` to configure your osu! profile.
///
/// Affected parameters for configuration:
/// - PP: To know if the bot should display the PP stadistics.
///
/// Usage:
/// `osu_profile`
/// `osu_profile -GN`
#[command]
#[aliases(
    "oprofile",
    "oprof",
    "osuprofile",
    "osuprof",
    "osu_prof",
    "osu_p",
    "osup",
    "osu_p",
    "osu"
)]
async fn osu_profile(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Obtains the osu! api key from the "global" data
    let osu_key = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap().clone(); // get the tokens from the global data.
        tokens.old_osu.to_string()
    };

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let mut username = args.message().replace(" ", "_");
    let author_id = *msg.author.id.as_u64() as i64;

    // get the author_id as a signed 64 bit int, because that's what the database asks for.

    let user_data = {
        if username.is_empty() {
            sqlx::query_as!(
                OsuUserRawDBDataMinimal,
                "SELECT osu_username, pp FROM osu_user WHERE discord_id = $1",
                author_id
            )
            .fetch_optional(&pool)
            .await?
        } else {
            sqlx::query_as!(
                OsuUserRawDBDataMinimal,
                "SELECT osu_username, pp FROM osu_user WHERE osu_username = $1",
                username
            )
            .fetch_optional(&pool)
            .await?
        }
    };

    let mut pp = true;

    if let Some(row) = user_data {
        pp = row.pp.unwrap_or(true);
        if username.is_empty() {
            username = row.osu_username;
        }
    } else if username.is_empty() {
        if let Ok(m) = msg.member(ctx).await {
            username = m.display_name().to_string();
        } else {
            username = msg.author.name.to_string();
        }
    }

    let resp = get_osu_user(&username, &osu_key).await?;

    let user = if !resp.is_empty() {
        &resp[0]
    } else {
        msg.channel_id
            .say(
                ctx,
                format!("A user with the name of `{}` was not found.", username),
            )
            .await?;
        return Ok(());
    };

    let country_url = format!("https://raw.githubusercontent.com/stevenrskelton/flag-icon/master/png/75/country-squared/{}.png", &user.country.to_lowercase());

    if user.total_score.is_none() {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.timestamp(user.join_date.clone());
                    e.thumbnail(format!("https://a.ppy.sh/{}", &user.user_id));
                    e.author(|a| {
                        a.name(&user.username);
                        a.url(format!("https://osu.ppy.sh/u/{}", &user.user_id));
                        a.icon_url(country_url)
                    })
                })
            })
            .await?;
    } else {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.color(Colour::new({
                        let colour = user.user_id.parse().unwrap();
                        if colour > 16777215 {
                            15227880
                        } else {
                            colour
                        }
                    }));
                    e.timestamp(user.join_date.clone());
                    e.author(|a| {
                        a.name(&user.username);
                        a.url(format!("https://osu.ppy.sh/u/{}", &user.user_id));
                        a.icon_url(country_url)
                    });
                    e.image(format!("https://a.ppy.sh/{}", &user.user_id));

                    e.description({
                        let mut s = format!(
                            "
                                **{} -- {} -- {}** == 300/100/50
                                **{:.2}% -- {}** Plays
                                Total = **{}** ┇ Ranked = **{}**
                                Played **{}** seconds or: **{:?}**
                                **L{}** > Next level: `{}`
                                ",
                            &user
                                .count300
                                .as_ref()
                                .unwrap()
                                .parse::<u128>()
                                .expect("NaN")
                                .to_formatted_string(&Locale::en),
                            &user
                                .count100
                                .as_ref()
                                .unwrap()
                                .parse::<u128>()
                                .expect("NaN")
                                .to_formatted_string(&Locale::en),
                            &user
                                .count50
                                .as_ref()
                                .unwrap()
                                .parse::<u128>()
                                .expect("NaN")
                                .to_formatted_string(&Locale::en),
                            &user.accuracy.as_ref().unwrap().parse::<f32>().expect("NAN"),
                            &user
                                .playcount
                                .as_ref()
                                .unwrap()
                                .parse::<u128>()
                                .expect("NaN")
                                .to_formatted_string(&Locale::en),
                            &user
                                .total_score
                                .as_ref()
                                .unwrap()
                                .parse::<u128>()
                                .expect("NaN")
                                .to_formatted_string(&Locale::en),
                            &user
                                .ranked_score
                                .as_ref()
                                .unwrap()
                                .parse::<u128>()
                                .expect("NaN")
                                .to_formatted_string(&Locale::en),
                            &user
                                .total_seconds_played
                                .as_ref()
                                .unwrap()
                                .parse::<u64>()
                                .expect("NaN")
                                .to_formatted_string(&Locale::en),
                            seconds_to_days(
                                user.total_seconds_played
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u64>()
                                    .expect("NaN")
                            ),
                            user.level.as_ref().unwrap().parse::<f32>().expect("NaN") as u8,
                            pacman(&user.level.as_ref().unwrap()),
                        );
                        if pp {
                            s += &format!(
                                "Global: #**{}** | Country: #**{}**",
                                &user
                                    .pp_rank
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                                &user
                                    .pp_country_rank
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                            );
                        }
                        s
                    });
                    e.footer(|f| {
                        if pp {
                            f.text(format!(
                                "PP:{} | SSH:{} | SS:{} | SH:{} | S:{}",
                                &user.pp_raw.as_ref().unwrap(),
                                &user
                                    .count_rank_ssh
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                                &user
                                    .count_rank_ss
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                                &user
                                    .count_rank_sh
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                                &user
                                    .count_rank_s
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                            ))
                        } else {
                            f.text(format!(
                                "SSH:{} | SS:{} | SH:{} | S:{}",
                                &user
                                    .count_rank_ssh
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                                &user
                                    .count_rank_ss
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                                &user
                                    .count_rank_sh
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                                &user
                                    .count_rank_s
                                    .as_ref()
                                    .unwrap()
                                    .parse::<u128>()
                                    .expect("NaN")
                                    .to_formatted_string(&Locale::en),
                            ))
                        }
                    })
                })
            })
            .await?;
    }

    Ok(())
}

/// Obtains your score on the specified beatmap id
/// The number that's sent along with the recent command is the beatmap id of it.
///
/// You can use `osuc` to configure your osu! profile.
///
/// Affected parameters for configuration:
/// - Mode: To specify the gamemode the score was on.
/// - PP: To know if the bot should display the PP stadistics.
///
/// Usage: `score 124217`
#[command]
#[aliases("compare", "scr")]
async fn score(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let bmap_id = match args.parse::<u64>() {
        Err(_) => {
            msg.reply(ctx, "An invalid id was provided").await?;
            return Ok(());
        }
        Ok(x) => x,
    };

    // Obtains the osu! api key from the "global" data
    let osu_key = {
        let data = ctx.data.read().await; // set inmutable global data.
        let tokens = data.get::<Tokens>().unwrap().clone(); // get the tokens from the global data.
        tokens.old_osu.to_string()
    };

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let author_id = *msg.author.id.as_u64() as i64;

    let user_data = sqlx::query_as!(
        OsuUserRawDBData,
        "SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1",
        author_id
    )
    .fetch_optional(&pool)
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
    } else if let Ok(m) = msg.member(ctx).await {
        username = m.display_name().to_string();
    } else {
        username = msg.author.name.to_string();
    }

    let score = get_osu_scores(osu_id, &username, bmap_id, mode, &osu_key).await?;

    let s = if let Some(x) = score.get(0) {
        x
    } else {
        msg.channel_id
            .say(
                ctx,
                format!(
                    "The user `{}` does not have any scores on the specified map.",
                    &username
                ),
            )
            .await?;
        return Ok(());
    };

    let beatmap_raw = get_osu_beatmap(&bmap_id.to_string(), &osu_key).await?;
    let beatmap = &beatmap_raw[0];

    let accuracy = acc_math(
        s.count300.parse()?,
        s.count100.parse()?,
        s.count50.parse()?,
        s.countmiss.parse()?,
    )
    .await;

    let mods: String = get_mods_short(s.enabled_mods.parse()?).await;

    let rating_url = format!("https://s.ppy.sh/images/{}.png", s.rank.to_uppercase());

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.color(Colour::new(osu_id as u32));

                e.title(format!(
                    "{} - {} [**{}**]\nby {}",
                    beatmap.artist, beatmap.title, beatmap.version, beatmap.creator
                ));
                e.url(format!("https://osu.ppy.sh/b/{}", beatmap.beatmap_id));

                e.description(format!(
                    "**{}** ┇ **x{} / {}**\n**{:.2}%** ┇ {} - {} - {} - {}",
                    s.score
                        .parse::<u32>()
                        .expect("NaN")
                        .to_formatted_string(&Locale::en),
                    s.maxcombo,
                    beatmap.max_combo,
                    accuracy,
                    s.count300,
                    s.count100,
                    s.count50,
                    s.countmiss
                ));
                e.timestamp(s.date.clone());
                e.thumbnail(format!(
                    "https://b.ppy.sh/thumb/{}l.jpg",
                    beatmap.beatmapset_id
                ));

                e.author(|a| {
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
                        f.text(format!(
                            "{}pp | {:.4}* | {}",
                            s.pp.as_ref().unwrap_or(&"0".to_string()),
                            beatmap.difficultyrating,
                            mods
                        ));
                    } else {
                        f.text(format!("{:.4}* | {}", beatmap.difficultyrating, mods));
                    }
                    f.icon_url(&rating_url)
                });
                e
            })
        })
        .await?;

    Ok(())
}

/// Command to show the most recent osu! play.
/// - Due to api limits, this will only work on maps with leaderboard.
/// - This command is able to show failed plays, and show the percentage of progress.
///
/// To use this command, first configure your osu! profile with `osuc`
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
/// `recent vicky5124`
#[command]
#[aliases("rs", "rc")]
async fn recent(ctx: &Context, msg: &Message, arguments: Args) -> CommandResult {
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
        tokens.old_osu.to_string()
    };

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let mut user_data = OsuUserDBData::default(); // generate a basic structure with the default values.

    let data = if arg_user.is_empty() {
        let author_id = *msg.author.id.as_u64() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
        arg_user = msg.author.name.clone();
        sqlx::query_as!(OsuUserRawDBData, "SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", // query the SQL to the database.
            author_id)
        .fetch_optional(&pool)
        .await? // The arguments on this array will go to the respective calls as $ in the database (arrays start at 1 in this case reeeeee)
    } else {
        sqlx::query_as!(OsuUserRawDBData, "SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE osu_username = $1", // query the SQL to the database.
            arg_user)
        .fetch_optional(&pool)
        .await?
    };

    if let Some(x) = data {
        // if the data is not empty, aka if the user is on the database already
        // Parses the database result into each of the pieces of data on the structure.
        user_data.osu_id = x.osu_id;
        user_data.name = x.osu_username;
        user_data.mode = x.mode;
        user_data.pp = x.pp;
        user_data.short_recent = x.short_recent;
    } else {
        if arg_user.is_empty() {
            msg.channel_id.say(ctx, "It looks like you don't have a configured osu! username, consider configuring one with `osuc`").await?;
        }
        user_data.name = arg_user;
        user_data.mode = Some(0);
        user_data.pp = Some(true);
        user_data.short_recent = Some(true);
    }

    let user_data_name = user_data.name.replace("`", "").replace("@", "@\u{200b}");

    if user_data.osu_id == 0 {
        let user_id = get_osu_id(&user_data.name, &osu_key).await?;
        if user_id == 0 {
            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "Could not find any osu! user with the name of '{}'",
                        user_data_name
                    ),
                )
                .await?;
            return Ok(());
        } else {
            user_data.osu_id = user_id;
        }
    }

    let bot_msg = msg
        .channel_id
        .say(ctx, format!("Obtaining **{}** recent data", user_data_name))
        .await?;

    let user_recent_raw = get_osu_user_recent(user_data.osu_id, &osu_key).await?;

    if user_recent_raw.is_empty() {
        bot_msg
            .clone()
            .edit(ctx, |m| {
                m.content(format!(
                    "The user **{}** has not played in the last 24 hours.",
                    user_data_name
                ));
                m
            })
            .await?;
        return Ok(());
    }

    // Group all the needed data to EventData
    let event_data = EventData {
        user_db_data: Some(user_data),
        user_recent_raw: Some(user_recent_raw.clone()),
        osu_key: Some(osu_key),
    };

    let mut page = 0;

    // Build the initial recent embed
    short_recent_builder(ctx.http.clone(), &event_data, bot_msg.clone(), page).await?;

    // Add left and right reactions, to make the life easier for the user using the event.

    let left = ReactionType::Unicode(String::from("⬅️"));
    let right = ReactionType::Unicode(String::from("➡️"));

    bot_msg.react(ctx, left).await?;
    bot_msg.react(ctx, right).await?;

    loop {
        if let Some(reaction) = &bot_msg
            .await_reaction(ctx)
            .author_id(msg.author.id.0)
            .timeout(Duration::from_secs(20))
            .await
        {
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "➡️" => {
                    if page != 0 {
                        page -= 1;
                    }
                }
                "⬅️" => {
                    if page != user_recent_raw.len() - 1 {
                        page += 1;
                    }
                }
                _ => (),
            }

            if short_recent_builder(ctx.http.clone(), &event_data, bot_msg.clone(), page)
                .await
                .is_err()
            {
                break;
            }
            let _ = reaction.as_inner_ref().delete(ctx).await;
        } else {
            let _ = bot_msg.delete_reactions(ctx).await;
            break;
        };
    }

    Ok(())
}

async fn top_play_embed_builder(
    osu_key: &str,
    data: &OsuData,
    play: &OsuUserBest,
    user: &OsuUserData,
    index: usize,
) -> Result<CreateEmbed, Box<dyn std::error::Error + Send + Sync>> {
    let beatmap_raw = get_osu_beatmap(&play.beatmap_id, &osu_key).await?;
    let beatmap = &beatmap_raw[0];

    let mods: String = get_mods_short(play.enabled_mods.parse()?).await;
    let accuracy = acc_math(
        play.count300.parse()?,
        play.count100.parse()?,
        play.count50.parse()?,
        play.countmiss.parse()?,
    )
    .await;
    let rating_url = format!("https://s.ppy.sh/images/{}.png", play.rank.to_uppercase());

    let mut e = CreateEmbed::default();

    e.color(Colour::new(data.id as u32));

    e.author(|a| {
        a.name(format!("Play #{} from \"{}\"", index, user.username));
        a.icon_url(format!("https://a.ppy.sh/{}", user.user_id));
        a.url(format!("https://osu.ppy.sh/u/{}", user.user_id))
    });

    e.title(format!(
        "{} - {} [**{}**]\nby {}",
        beatmap.artist, beatmap.title, beatmap.version, beatmap.creator
    ));
    e.url(format!("https://osu.ppy.sh/b/{}", beatmap.beatmap_id));

    e.description(format!(
        "**{}** ┇ **x{} / {}**\n**{:.2}%** ┇ {} - {} - {} - {}",
        play.score
            .parse::<u32>()
            .expect("NaN")
            .to_formatted_string(&Locale::en),
        play.maxcombo,
        beatmap.max_combo,
        accuracy,
        play.count300,
        play.count100,
        play.count50,
        play.countmiss
    ));
    e.timestamp(play.date.clone());
    e.thumbnail(format!(
        "https://b.ppy.sh/thumb/{}l.jpg",
        beatmap.beatmapset_id
    ));

    e.footer(|f| {
        f.text(format!(
            "{}pp | {:.4}* | {}",
            &play.pp, beatmap.difficultyrating, mods
        ));
        f.icon_url(&rating_url)
    });

    Ok(e)
}

/// Command to show the top plays of a user.
/// - Only osu!std supported.
///
/// To use this command, you may want to configure your osu! profile with `osuc`
/// Affected parameters for configuration:
/// - PP: If set to false, users will not be able to see your top plays.
///
/// You can also invoke the command specifying a username.
/// Usage:
/// `osu_top`
/// `osu!top [ Frost ]`
/// `otop vicky5124`
#[command]
#[aliases("osutop", "otop", "top_plays", "topplays", "toplays", "top", "osu!top")]
async fn osu_top(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut args_user = args.message().to_string();
    if args_user.is_empty() {
        args_user = msg.author.name.to_string();
    }

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let osu_key = {
        let data_read = ctx.data.read().await;
        let tokens = data_read.get::<Tokens>().unwrap().clone();

        tokens.old_osu.to_string()
    };

    let mut config = OsuData::default();

    if let Ok(id) = args_user.parse::<i32>() {
        let data = sqlx::query!("SELECT * FROM osu_user WHERE osu_id = $1", id)
            .fetch_optional(&pool)
            .await?;
        if let Some(info) = data {
            config.username = info.osu_username;
            config.id = info.osu_id;
            config.pp = info.pp.unwrap_or(true);
        } else {
            let data = sqlx::query!("SELECT * FROM osu_user WHERE osu_username = $1", &args_user)
                .fetch_optional(&pool)
                .await?;
            if let Some(info) = data {
                config.username = info.osu_username;
                config.id = info.osu_id;
                config.pp = info.pp.unwrap_or(true);
            } else {
                config.id = id;
                config.username = {
                    config.pp = true;
                    let username = get_osu_username(&id, &osu_key).await?;

                    if username.is_empty() {
                        config.id = get_osu_id(&args_user, &osu_key).await?;
                        args_user
                    } else {
                        username
                    }
                };
            }
        }
    } else {
        let data = sqlx::query!("SELECT * FROM osu_user WHERE osu_username = $1", &args_user)
            .fetch_optional(&pool)
            .await?;
        if let Some(info) = data {
            config.username = info.osu_username;
            config.id = info.osu_id;
            config.pp = info.pp.unwrap_or(true);
        } else {
            config.id = get_osu_id(&args_user, &osu_key).await?;
            config.username = args_user;
            config.pp = true;
        }
    }

    if !config.pp {
        msg.reply(
            ctx,
            format!(
                "The user `{}` does not want anything to do with pp.",
                config.username.replace("@", "@\u{200b}")
            ),
        )
        .await?;
        return Ok(());
    }

    let data = get_osu_user_best(&config.id, &osu_key).await?;

    if data.is_empty() {
        msg.reply(
            ctx,
            format!(
                "The user `{}` does not have any plays in osu!std.",
                config.username.replace("@", "@\u{200b}")
            ),
        )
        .await?;
        return Ok(());
    }

    let mut index = 0;
    let user = &get_osu_user(&config.username, &osu_key).await?[0];

    let embed = top_play_embed_builder(&osu_key, &config, &data[index], &user, index + 1).await?;
    let mut message = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|mut e| {
                e.0 = embed.0;
                e
            })
        })
        .await?;

    let left = ReactionType::Unicode(String::from("⬅️"));
    let right = ReactionType::Unicode(String::from("➡️"));

    message.react(ctx, left).await?;
    message.react(ctx, right).await?;

    loop {
        if let Some(reaction) = &message
            .await_reaction(ctx)
            .author_id(msg.author.id.0)
            .timeout(Duration::from_secs(120))
            .await
        {
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "⬅️" => {
                    if index != 0 {
                        index -= 1;
                    }
                }
                "➡️" => {
                    if index != data.len() - 1 {
                        index += 1;
                    }
                }
                _ => (),
            }

            let embed =
                top_play_embed_builder(&osu_key, &config, &data[index], &user, index + 1).await?;
            message
                .edit(ctx, |m| {
                    m.embed(|mut e| {
                        e.0 = embed.0;
                        e
                    })
                })
                .await?;
            let _ = reaction.as_inner_ref().delete(ctx).await;
        } else {
            let _ = message.delete_reactions(ctx).await;
            break;
        };
    }

    Ok(())
}

#[command]
#[aliases(mappp, mapp, map_pp, beatmappp, beatmapp)]
async fn beatmap_pp(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    println!("{}", args.message());
    let matches_result = App::new("Beatmap PP")
        .arg(
            Arg::with_name("mods")
                .long("mod")
                .short("m")
                .multiple(true)
                .takes_value(true)
                .required(false),
        )
        .get_matches_from_safe(args.message().split(' '));

    let matches = match matches_result {
        Ok(x) => x,
        Err(why) => {
            println!("{}", why);
            return Ok(());
        }
    };

    let mods = if let Some(mods) = matches.values_of("mods") {
        mods.collect::<Vec<&str>>()
    } else {
        vec!["NM"]
    };

    msg.reply(ctx, format!("{:?}", mods)).await?;

    Ok(())
}
