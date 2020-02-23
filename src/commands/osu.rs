/// This is the file containing all the osu! related commands.

use crate::{
    DatabaseConnection,
    Tokens,
    RecentIndex,
};
use serenity::{
    http::Http,
    utils::Colour,
    prelude::{
        Context,
        TypeMapKey,
        RwLock,
        ShareMap,
    },
    model::{
        channel::{
            Message,
            ReactionType,
        },
        prelude:: {
            MessageId,
            ChannelId,
        },
    },
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};
use hey_listen::sync::{
    ParallelDispatcher as Dispatcher,
    ParallelDispatcherRequest as DispatcherRequest
};
use std::{
    thread,
    sync::Arc,
    hash::{
        Hash,
        Hasher,
    },
};
use num_format::{
    Locale,
    ToFormattedString,
};
use regex::Regex;
use reqwest;
use serde::Deserialize;



#[derive(Clone)]
pub enum DispatchEvent {
    ReactEvent(MessageId, ReactionType, bool),
}

impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (DispatchEvent::ReactEvent(self_message_id, self_emoji, _),
            DispatchEvent::ReactEvent(other_message_id, other_emoji, _)) => {
                self_message_id == other_message_id &&
                self_emoji == other_emoji
            }
        }
    }
}

impl Eq for DispatchEvent {}

impl Hash for DispatchEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DispatchEvent::ReactEvent(msg_id, user_id, _) => {
                msg_id.hash(state);
                user_id.hash(state);
            }
        }
    }
}


pub struct DispatcherKey;
impl TypeMapKey for DispatcherKey {
    type Value = Arc<RwLock<Dispatcher<DispatchEvent>>>;
}



pub fn left_reaction_event(http: Arc<Http>, channel: ChannelId, data: Arc<RwLock<ShareMap>>, _command: &str, event_data: EventData) ->
    Box<dyn Fn(&DispatchEvent) -> Option<DispatcherRequest> + Send + Sync> {

    Box::new(move |event| {
        let mut kill = false;
        if let DispatchEvent::ReactEvent(_, _, true) = event {
            kill = true;
        }
        let msg_id = match event {
            DispatchEvent::ReactEvent(m, _, _) => m.0,
        };
        let mut wdata = data.write();
        let hm = wdata.get_mut::<RecentIndex>().unwrap();

        let index = hm.entry(msg_id).or_insert(0);

        let msg = match http.clone().get_message(channel.0, msg_id){
            Err(why) => {
                println!("Could not obtain message: {}", why);
                return Some(DispatcherRequest::StopListening);
            },
            Ok(x) => x
        };

        if kill {
            hm.remove_entry(&msg_id);
            Some(DispatcherRequest::StopListening)
        } else {
            *index -= 1;
            let _ = short_recent_builder(http.clone(), &event_data, msg.clone(), *index);
            None
        }
    })
}

pub fn right_reaction_event(http: Arc<Http>, channel: ChannelId, data: Arc<RwLock<ShareMap>>, _command: &str, event_data: EventData) ->
    Box<dyn Fn(&DispatchEvent) -> Option<DispatcherRequest> + Send + Sync> {

    Box::new(move |event| {
        let mut kill = false;
        if let DispatchEvent::ReactEvent(_, _, true) = event {
            kill = true;
        }
        let msg_id = match event {
            DispatchEvent::ReactEvent(m, _, _) => m.0,
        };
        let mut wdata = data.write();
        let hm = wdata.get_mut::<RecentIndex>().unwrap();

        let index = hm.entry(msg_id).or_insert(0);

        let msg = match http.clone().get_message(channel.0, msg_id){
            Err(why) => {
                println!("Could not obtain message: {}", why);
                return Some(DispatcherRequest::StopListening);
            },
            Ok(x) => x
        };

        if kill {
            hm.remove_entry(&msg_id);
            Some(DispatcherRequest::StopListening)
        } else {
            *index += 1;
            let _ = short_recent_builder(http.clone(), &event_data, msg.clone(), *index);
            None
        }
   })
}



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
            const Key6           = 131072;
            const Key7           = 262144;
            const Key8           = 524288;
            const FadeIn         = 1048576;
            const Random         = 2097152;
            const Cinema         = 4194304;
            const Target         = 8388608;
            const Key9           = 16777216;
            const KeyCoop        = 33554432;
            const Key1           = 67108864;
            const Key3           = 134217728;
            const Key2           = 268435456;
            const ScoreV2        = 536870912;
            const Mirror         = 1073741824;
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
            const AP = 8192;//Autopilot
            const PF = 16384;
            const K4 = 32768;
            const K5 = 65536;
            const K6 = 131072;
            const K7 = 262144;
            const K8 = 524288;
            const FI = 1048576;
            const RD = 2097152;
            const CN = 4194304;
            const TP = 8388608;
            const K9 = 16777216;
            const CO = 33554432;
            const K1 = 67108864;
            const K3 = 134217728;
            const K2 = 268435456;
            const V2 = 536870912;
            const MR = 1073741824;
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

#[derive(Default, Clone)]
pub struct EventData {
    user_db_data: Option<OsuUserDBData>,
    user_recent_raw: Option<Vec<OsuUserRecentData>>,
    osu_key: Option<String>,
}

fn acc_math(_300: f32, _100: f32, _50: f32, _miss: f32) -> f32 {
    let mix = _300  + _100  + _50  + _miss ;

    let pcount50 = _50  / mix * (100.0 / 6.0);
    let pcount100 = _100  / mix * (100.0 / 3.0);
    let pcount300 = _300  / mix * 100.0;

    let acc: f32 = pcount50 + pcount100 + pcount300;
    acc
}

fn progress_math(count_normal: f32, count_slider: f32, count_spinner: f32, _300: f32, _100: f32, _50: f32, _miss: f32) -> f32 {
    let all_the_things = count_normal + count_slider + count_spinner;
    let everything = _300 + _100 + _50 + _miss;
    let progress = everything / all_the_things * 100.0;
    
    progress
}

fn _get_mods_long(value: u32) -> String {
    use bitwhise_mods::LongMods;

    let mods = LongMods::from_bits_truncate(value);
    format!("{:?}", mods)
}

fn get_mods_short(value: u32) -> String {
    use bitwhise_mods::ShortMods;

    let mods = ShortMods::from_bits_truncate(value);
    format!("{:?}", mods)
}



// This function simply calls the osu! api to get the id of the user from a username.
fn get_osu_id(name: &String, osu_key: &String) -> Result<i32, Box<dyn std::error::Error>> {
    let resp = get_osu_user(&name, osu_key)?;

    if resp.len() != 0 {
        let id: i32 = resp[0].user_id.trim().parse()?;
        return Ok(id);
    } else {
        return Ok(0);
    }
}



fn get_osu_user(name: &String, osu_key: &String) -> Result<Vec<OsuUserData>, Box<dyn std::error::Error>> {
    let re = Regex::new("[^0-9A-Za-z\\[\\]'_ ]").unwrap();
    let mut sanitized_name = re.replace_all(name, "").into_owned();
    sanitized_name = sanitized_name.replace(" ", "%20");

    let url = format!("https://osu.ppy.sh/api/get_user?k={}&u={}&type=string", osu_key, sanitized_name);
    let resp = reqwest::blocking::get(&url)?
        .json::<Vec<OsuUserData>>()?;

    Ok(resp)
}

fn get_osu_user_recent(user_id: i32, osu_key: &String) -> Result<Vec<OsuUserRecentData>, Box<dyn std::error::Error>> {
    let url = format!("https://osu.ppy.sh/api/get_user_recent?k={}&u={}&type=id", osu_key, user_id);
    let resp = reqwest::blocking::get(&url)?
        .json::<Vec<OsuUserRecentData>>()?;
    Ok(resp)
}

fn get_osu_beatmap(beatmap_id: &String, osu_key: &String) -> Result<Vec<OsuBeatmapData>, Box<dyn std::error::Error>> {
    let url = format!("https://osu.ppy.sh/api/get_beatmaps?k={}&b={}", osu_key, beatmap_id);
    let resp = reqwest::blocking::get(&url)?
        .json::<Vec<OsuBeatmapData>>()?;
    Ok(resp)
}

fn short_recent_builder(http: Arc<Http>, event_data: &EventData, bot_msg: Message, index: usize) -> Result<(), Box<dyn std::error::Error>> {
    let user_data = event_data.user_db_data.as_ref().unwrap();
    let user_recent_raw = event_data.user_recent_raw.as_ref().unwrap();
    let osu_key = event_data.osu_key.as_ref().unwrap();

    let user_recent = &user_recent_raw[index];
    let user_raw = get_osu_user(&user_data.name, &osu_key)?;
    let user = &user_raw[0];

    let beatmap_raw = get_osu_beatmap(&user_recent.beatmap_id, &osu_key)?;
    let beatmap = &beatmap_raw[0];

    let accuracy = acc_math(user_recent.count300.parse()?, user_recent.count100.parse()?, user_recent.count50.parse()?, user_recent.countmiss.parse()?);

    let progress: f32 = progress_math(beatmap.count_normal.parse()?, beatmap.count_slider.parse()?, beatmap.count_spinner.parse()?,
    user_recent.count300.parse()?, user_recent.count100.parse()?, user_recent.count50.parse()?, user_recent.countmiss.parse()?);

    let attempts = index;
    let mods: String = get_mods_short(user_recent.enabled_mods.parse()?);

    let rating_url: String;

    if user_recent.rank == "F" {
        rating_url = String::from("https://5124.mywire.org/HDD/Downloads/BoneF.png");
    } else {
        rating_url = format!("https://s.ppy.sh/images/{}.png", user_recent.rank.to_uppercase());
    }

    bot_msg.clone().edit(http.clone(), |m| { // say method doesn't work for the message builder.
        m.content("");
        m.embed( |e| {
            e.color(Colour::new(user.user_id.parse().unwrap()));
            e.title(format!("{} - {} [**{}**]\nby {}",
                            beatmap.artist, beatmap.title, beatmap.version, beatmap.creator));
            e.url(format!("https://osu.ppy.sh/b/{}", beatmap.beatmap_id));
            e.description(format!("**{}** ┇ **x{} / {}**\n**{:.2}%** ┇ {} - {} - {} - {}\n Recent #{} ━ Progress: {:.2}%",
                                  user_recent.score.parse::<u32>().expect("NaN").to_formatted_string(&Locale::en), user_recent.maxcombo, beatmap.max_combo, accuracy, user_recent.count300, user_recent.count100, user_recent.count50, user_recent.countmiss, attempts, progress));
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
    })?;
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
/// `n!osuc Majorowsky`
/// `n!osuc nitsuga5124 pp=false short_recent=yes`
/// `n!osuc [ Frost ] mode=mania pp=yes recent=false`
#[command]
#[aliases("osuc", "config_osu", "configosu", "configureosu", "configo", "setosu", "osuset", "set_osu", "osu_set")]
fn configure_osu(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {

    let client;
    let osu_key;
    {
        let data = ctx.data.read(); // set inmutable global data.
        osu_key = data.get::<Tokens>().unwrap().clone(); // get the osu! api token from the global data.
    }
    let mut data = ctx.data.write(); // set mutable global data.
    client = data.get_mut::<DatabaseConnection>().unwrap(); // get the database connection from the global data.

    let author_id = msg.author.id.as_u64().clone() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let data ={
        let mut client = client.write();
        client.query("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", // query the SQL to the database.
                     &[&author_id])? // The arguments on this array will go to the respective calls as $ in the database (arrays start at 1 in this case reeeeee)
    };
    let empty_data: bool;

    let mut user_data = OsuUserDBData::default(); // generate a basic structure with the default values.

    if !data.is_empty() { // if the data is not empty, aka if the user is on the database already
        empty_data = false;
        // Parses the database result into each of the pieces of data on the structure.
        for row in data {
            user_data.osu_id = row.get(0);
            user_data.name = row.get(1);
            user_data.old_name = user_data.name.clone();
            user_data.mode = row.get(3);
            user_data.pp = row.get(2);
            user_data.short_recent = row.get(4);
        }
    } else {
        empty_data = true;
    }
    
    // if there where arguments on the command (aka the user wants to modify a value)
    if arguments.len() > 0 {
        // Transforms the given arguments as a vector
        let args = arguments.raw_quoted().collect::<Vec<&str>>();
        
        // iterates over all the arguments on the list
        for arg in args {
            // if the argument is the keyword PP
            if arg.starts_with("pp=") {
                // Split the argument on the first = and get everything after it.
                let x: &str = arg.split("=").nth(1).unwrap();
                // Match the text after =
                user_data.pp = match x {
                    // if x == "n" || x == "no" ... {user_data.pp = Some(false)}
                    "n" | "no" | "false" | "0" => Some(false),
                    // else Some(true)
                    _ => Some(true)
                }

            // if the argument starts with the keyword short_recent OR recent
            } else if arg.starts_with("short_recent=") || arg.starts_with("recent=") { 
                let x: &str = arg.split("=").nth(1).unwrap();
                user_data.short_recent = match x {
                    "n" | "no" | "false" | "0" => Some(false),
                    _ => Some(true)
                }

            } else if arg.starts_with("mode=") { 
                let x: &str = arg.split("=").nth(1).unwrap();
                user_data.mode = match x {
                    "0" | "std" | "standard" => Some(0),
                    "1" | "taiko" => Some(1),
                    "2" | "ctb" | "catch" => Some(2),
                    "3" | "mania" => Some(3),
                    _ => Some(0)
                }
            
            // this triggers if the argument was not a keyword argument and adds the argument to
            // the username adding a space.
            } else {
                if empty_data {
                    user_data.name += arg;
                } else {
                    user_data.name = if user_data.name == user_data.old_name {arg.to_string()} else {user_data.name + " " + arg};
                }
            }
        }
    } else {
        if empty_data {
            // sends the help of the command
            msg.channel_id.say(&ctx, "send help!")?;
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
            msg.channel_id.say(&ctx, current_conf)?;
            return Ok(());
        }
    }
    // calls the get_osu_id function to get the id of the user.
    user_data.osu_id = get_osu_id(&user_data.name, &osu_key)?;

    // applies the default values in case of being not specified.
    user_data.pp = match &user_data.pp {
        None => Some(true),
        Some(b) => Some(b.clone()),
    };
    user_data.mode = match &user_data.mode {
        None => Some(0),
        Some(b) => Some(b.clone()),
    };
    user_data.short_recent = match &user_data.short_recent {
        None => Some(true),
        Some(b) => Some(b.clone()),
    };

    if empty_data {
        // inserts the data because the user is new.
        {
            let mut client = client.write();
            client.execute(
                "INSERT INTO osu_user (osu_id, osu_username, pp, mode, short_recent, discord_id) VALUES ($1, $2, $3, $4, $5, $6)",
                &[&user_data.osu_id, &user_data.name, &user_data.pp.unwrap(), &user_data.mode.unwrap(), &user_data.short_recent.unwrap(), &author_id]
            )?;
        }

    } else {
        // updates the database with the new user data.
        {
            let mut client = client.write();
            client.execute(
                "UPDATE osu_user SET osu_id = $1, osu_username = $2, pp = $3, mode = $4, short_recent = $5 WHERE discord_id = $6",
                &[&user_data.osu_id, &user_data.name, &user_data.pp.unwrap(), &user_data.mode.unwrap(), &user_data.short_recent.unwrap(), &author_id]
            )?;
        }
    }
   
    // if the id obtained is 0, it means the user doesn't exist.
    if user_data.osu_id == 0 {
        msg.channel_id.say(&ctx, "It looks like your osu ID is 0, Is the Username correct?")?;
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

    msg.channel_id.say(&ctx, current_conf)?;

    Ok(())
}

/// Command to show the most recent osu! play.
/// - Due to api limits, this will only work on maps with leaderboard.
/// - This command is able to show failed plays, and show the % of the progress on the map. 
///
/// To use this command, first configure your osu! profile with `.osuc`
/// Affected parameters for configuration:
/// - Mode: To specify the gamemode the play was on.
/// - PP: To know if the bot should display the PP stadistics of the play.
/// - Short Recent: To display the short version of recent instead of the long one.
/// (Currently only short exists.)
///
/// You can also invoke the command specifying a username.
/// Ex: `.recent [ Frost ]`
#[command]
#[aliases("rs", "rc")]
fn recent(ctx: &mut Context, msg: &Message, arguments: Args) -> CommandResult {
    let mut arg_user = String::from("");
    if arguments.len() > 0 {
        let args = arguments.raw_quoted().collect::<Vec<&str>>();
        for i in args {
            arg_user += &format!("{} ", i).to_owned()[..];
        }
        arg_user.pop();
    }

    let osu_key = {
        let data = ctx.data.read(); // set inmutable global data.
        data.get::<Tokens>().unwrap().clone() // get the osu! api token from the global data.
    };

    let client = {
        let rdata = ctx.data.read();

        let client = Arc::clone(rdata.get::<DatabaseConnection>().expect("no database connection found")); // get the database connection from the global data.
        client
    };

    let dispatcher = {
        let mut wdata = ctx.data.write();
        wdata.get_mut::<DispatcherKey>().expect("Expected Dispatcher.").clone()
    };

    let mut user_data = OsuUserDBData::default(); // generate a basic structure with the default values.

    let data;

    if arg_user == "" {
        let author_id = msg.author.id.as_u64().clone() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
        {
            let mut client = client.write();
            data = client.query("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE discord_id = $1", // query the SQL to the database.
                                &[&author_id])?; // The arguments on this array will go to the respective calls as $ in the database (arrays start at 1 in this case reeeeee)

        }
        arg_user = msg.author.name.clone();


    } else {
        {
            let mut client = client.write();
            data = client.query("SELECT osu_id, osu_username, pp, mode, short_recent FROM osu_user WHERE osu_username = $1", // query the SQL to the database.
                                &[&arg_user])?;
        }

    }

    if !data.is_empty() { // if the data is not empty, aka if the user is on the database already
        // Parses the database result into each of the pieces of data on the structure.
        for row in data {
            user_data.osu_id = row.get(0);
            user_data.name = row.get(1);
            user_data.mode = row.get(3);
            user_data.pp = row.get(2);
            user_data.short_recent = row.get(4);
        }
    } else {
        if arg_user == "" {
            msg.channel_id.say(&ctx, "It looks like you don't have a configured osu! username, consider configuring one with `n!osuc`")?;
        }
        user_data.name = arg_user;
        user_data.mode = Some(0);
        user_data.pp = Some(true);
        user_data.short_recent = Some(true);
    }

    if user_data.osu_id == 0 {
        let user_id = get_osu_id(&user_data.name, &osu_key)?;
        if user_id == 0 {
            msg.channel_id.say(&ctx, format!("Could not find any osu! user with the name of '{}'", user_data.name))?;
            return Ok(());
        } else {
            user_data.osu_id = user_id;
        }
    }
    let bot_msg = msg.channel_id.say(&ctx, format!("Obtaining **{}** recent data", user_data.name))?;

    let user_recent_raw = get_osu_user_recent(user_data.osu_id, &osu_key)?;

    if user_recent_raw.len() < 1 {
        bot_msg.clone().edit(&ctx, |m| {
            m.content(format!("The user **{}** has not played in the last 24 hours.", user_data.name));
            m
        })?;
        return Ok(());
    }

    let http = ctx.http.clone();
    let msg = msg.clone();

    let mut event_data = EventData::default();
    event_data.user_db_data = Some(user_data);
    event_data.user_recent_raw = Some(user_recent_raw);
    event_data.osu_key = Some(osu_key);


    short_recent_builder(http.clone(), &event_data, bot_msg.clone(), 0)?;

    let mut timeout = 0;

    bot_msg.react(&ctx, "⬅️")?;
    bot_msg.react(&ctx, "➡️")?;

    let left = ReactionType::Unicode(String::from("⬅️"));
    let right = ReactionType::Unicode(String::from("➡️"));
  
    dispatcher.write()
        .add_fn(
            DispatchEvent::ReactEvent(bot_msg.id, left.clone(), false),
            left_reaction_event(http.clone(), bot_msg.channel_id, ctx.data.clone(), "recent", event_data.clone())
        );
    dispatcher.write()
        .add_fn(
            DispatchEvent::ReactEvent(bot_msg.id, right.clone(), false),
            right_reaction_event(http.clone(), bot_msg.channel_id, ctx.data.clone(), "recent", event_data.clone())
        );

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        timeout += 1;
        if timeout == 500 {
            break;
        }
    }
    dispatcher.write().dispatch_event(&DispatchEvent::ReactEvent(bot_msg.id, left.clone(), true));
    dispatcher.write().dispatch_event(&DispatchEvent::ReactEvent(bot_msg.id, right.clone(), true));

    if msg.guild_id != None{
        bot_msg.delete_reactions(&ctx)?;
    };

    Ok(())
}

#[command]
#[owners_only]
#[aliases("add")]
pub fn react(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let args = args.rest().to_string();

    let dispatcher = {
        let mut ctx = ctx.data.write();
        ctx.get_mut::<DispatcherKey>().expect("Expected Dispatcher.").clone()
    };

    let http = ctx.http.clone();
    let msg = msg.clone();

    let bot_msg = msg.channel_id.say(&http, &args)?;

    let mut timeout = 0;

    bot_msg.react(&ctx, "⬅️")?;
    bot_msg.react(&ctx, "➡️")?;

    let left = ReactionType::Unicode(String::from("⬅️"));
    let right = ReactionType::Unicode(String::from("➡️"));

    let event_data = EventData::default();

    dispatcher.write()
        .add_fn(
            DispatchEvent::ReactEvent(bot_msg.id, left.clone(), false),
            left_reaction_event(http.clone(), bot_msg.channel_id, ctx.data.clone(), "recent", event_data.clone())
        );
    dispatcher.write()
        .add_fn(
            DispatchEvent::ReactEvent(bot_msg.id, right.clone(), false),
            right_reaction_event(http.clone(), bot_msg.channel_id, ctx.data.clone(), "recent", event_data.clone())
        );

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        timeout += 1;
        if timeout == 500 {
            break;
        }
    }
    dispatcher.write().dispatch_event(&DispatchEvent::ReactEvent(bot_msg.id, left.clone(), true));
    dispatcher.write().dispatch_event(&DispatchEvent::ReactEvent(bot_msg.id, right.clone(), true));

    if msg.guild_id != None{
        bot_msg.delete_reactions(&ctx)?;
    };

    Ok(())
}
