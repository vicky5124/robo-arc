// crate = main.rs
use crate::{
    // import the idol and chan commands.
    // so they can be used for the picture command.
    commands::sankaku::{
        idol,
        chan,
    },
    // import the utils::booru module for all the argument and tags blacklisting.
    utils::{
        booru,
        basic_functions::capitalize_first,
        booru::{
            SAFE_BANLIST,
            UNSAFE_BANLIST,
        }
    },
    // import all the types that are used for the global data.
    Booru,
    BooruList,
    BooruCommands,
    ConnectionPool,
};

use std::{
    collections::HashMap,
    time::Duration,
};

use sqlx;
use futures::TryStreamExt;
use futures::stream::StreamExt;

use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        Delimiter,
        CommandResult,
        macros::command,
    },
};
// rand crate, used to select a random post from the request data.
use rand::Rng;

// reqwest is a crate used to do http requests.
// used to request posts matching the specified tags on the selected site.
use reqwest::{
    Client as ReqwestClient,
    header::*,
    Url as ReqUrl,
};
// quick_xml is an xml sedrde library.
// used to deserialize the xml data into structs.
use quick_xml;
// serde or SerializerDeserializer, is a library to srialize data structures into structs.
use serde::{
    Deserialize,
    Serialize,
};
use serde_aux::prelude::*;
use serde_json;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct NHentaiTags {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct NHentaiTitle {
    english: String,
    japanese: Option<String>,
    pretty: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct NHentaiImage {
    t: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct NHentaiImages {
    pages: Vec<NHentaiImage>,
    cover: NHentaiImage,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct NHentaiSearchResult {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    id: String,
    #[serde(deserialize_with = "deserialize_string_from_number")]
    media_id: String,
    title: NHentaiTitle,
    images: NHentaiImages,
    tags: Vec<NHentaiTags>,
    num_pages: u64,
    num_favorites: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct NHentaiSearch {
    result: Vec<NHentaiSearchResult>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct ScoreData {
    total: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Url {
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Tags {
    general: Option<Vec<Option<String>>>,
}

// defining the Post type to be used for the xml deserialized on the Posts vector.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Post {
    score: Option<String>,
    actual_score: Option<String>,
    source: Option<String>,
    sources: Option<Vec<String>>,
    rating: Option<String>,
    sample_url: Option<String>,
    file_url: Option<String>,
    sample: Option<Url>,
    file: Option<Url>,
    tags: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct PostE621 {
    score: Option<ScoreData>,
    actual_score: Option<String>,
    source: Option<String>,
    sources: Option<Vec<String>>,
    rating: Option<String>,
    sample_url: Option<String>,
    file_url: Option<String>,
    sample: Option<Url>,
    file: Option<Url>,
    tags: Option<Tags>,
    string_tags: Option<String>,
}

// defining the Posts vector to Deserialize the requested xml list.
#[derive(Debug, Deserialize, PartialEq, Clone)]
struct Posts {
    post: Vec<Post>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct PostsE621 {
    posts: Vec<PostE621>,
}

// Function to get the booru data and send it.
pub async fn get_booru(ctx: &mut Context, msg: &Message, booru: &Booru, args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: behoimi needs login.

    let channel = &ctx.http.get_channel(msg.channel_id.0).await?; // Gets the channel object to be used for the nsfw check.
    // Checks if the command was invoked on a DM
    let dm_channel = msg.guild_id == None;

    // Obtains a list of tags from the arguments.
    let raw_tags = {
        // if the channel is nsfw or a dm, parse for nsfw tags.
        if channel.is_nsfw() || dm_channel {
            let mut raw_tags = booru::obtain_tags_unsafe(args).await;
            booru::illegal_check_unsafe(&mut raw_tags).await
        // else, parse for safe tags.
        } else {
            let mut raw_tags = booru::obtain_tags_safe(args).await;
            booru::illegal_check_safe(&mut raw_tags).await
        }
    };

    // TODO: replace this with Url::parse_with_params
    let mut tags = raw_tags.iter().map(|x| format!("{}+", x)).collect::<String>();
    tags.pop();

    let reqwest = ReqwestClient::new();
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0".parse().unwrap());

    let page: usize = 0;

    let url = if booru.typ == 1 {
        format!("https://{}/index.php?page=dapi&s=post&q=index&tags={}&pid={}&limit=50", booru.url, tags, page)
    } else if booru.typ == 2 {
        format!("https://{}/post/index.xml?tags={}&page={}&limit=50", booru.url, tags, page)
    } else if booru.typ == 3 {
        format!("http://{}/post/index.xml?tags={}&page={}&limit=50", booru.url, tags, page)
    } else if booru.typ == 4 {
        format!("http://{}/posts.json?tags={}&page={}&limit=50", booru.url, tags, page)
    } else {
        "https://safebooru.org/index.php?page=dapi&s=post&q=index".to_string()
    };

    // Send a request with the parsed url, and return the output text.
    let resp = reqwest.get(&url)
        .headers(headers.clone())
        .send()
        .await?
        .text()
        .await?;

    // deserialize the request XML into the Posts struct.
    let xml = if booru.typ == 4 {
        let mut posts: PostsE621 = serde_json::from_str(&resp.as_str())?;
        for (index, post) in posts.clone().posts.iter().enumerate() {
            posts.posts[index].actual_score = Some(
                if let Some(score_data) = post.score.clone() {
                    score_data.total.to_string()
                } else {
                    "0".to_string()
                }
            );
            posts.posts[index].score = None;
            posts.posts[index].source = if let Some(sources) = post.sources.clone() {
                if !sources.is_empty() {
                    Some(sources[0].clone())
                } else {
                    None
                }
            } else {
                None
            };
            posts.posts[index].sample_url = Some(post.sample.clone().unwrap().url.unwrap_or_default());
            posts.posts[index].file_url = Some(post.file.clone().unwrap().url.unwrap_or_default());
            posts.posts[index].string_tags = Some({
                if let Some(t) = post.tags.clone() {
                    t.general.unwrap_or(vec![Some("gore".to_string())]).iter().map(|tag| format!("{} ", tag.as_ref().unwrap_or(&"gore".to_string()))).collect::<String>()
                } else {
                    "gore".to_string()
                }
            });
        }

        let mut new_raw_posts = serde_json::to_string(&posts)?;
        new_raw_posts = new_raw_posts.replace("\"posts\"", "\"post\"");
        new_raw_posts = new_raw_posts.replace("\"score\"", "\"__\"");
        new_raw_posts = new_raw_posts.replace("\"tags\"", "\"___\"");
        new_raw_posts = new_raw_posts.replace("\"string_tags\"", "\"tags\"");
        new_raw_posts = new_raw_posts.replace("\"actual_score\"", "\"score\"");
        let new_posts: Posts = serde_json::from_str(&new_raw_posts)?;

        new_posts
    } else  {
        let mut posts: Posts = quick_xml::de::from_str(&resp.as_str())?;
        for (index, post) in posts.clone().post.iter().enumerate() {
            posts.post[index].actual_score = post.score.clone();
        }
        posts
    };

    // gets a random post from the vector.
    let choice;
    {
        let mut y = 1;
        loop {
            let r = rand::thread_rng().gen_range(0, xml.post.len()); 
            let x = xml.post[r].clone();
            let mut is_unsafe = false;
            y += 1;

            if channel.is_nsfw() || dm_channel {
                for tag in x.clone().tags.unwrap_or("gore".to_string()).split(' ').into_iter() {
                    if UNSAFE_BANLIST.contains(&tag) {
                        is_unsafe = true;
                    }
                }
            } else {
                for tag in x.clone().tags.unwrap_or("gore".to_string()).split(' ').into_iter() {
                    if SAFE_BANLIST.contains(&tag) {
                        is_unsafe = true;
                    }
                }
            }
            if !is_unsafe {
                choice = x;
                break;
            }
            if y > (&resp.len()*2) {
                msg.channel_id.say(&ctx, "All the content matching the requested tags is too big to be sent or illegal.").await?;
                return Ok(());
            }
        }
    };

    // define both url types.
    let full_size = if booru.url != "danbooru.donmai.us" {
        (*choice.file_url.as_ref().unwrap()).to_string() // full size image
    } else {
        (*choice.file_url.as_ref().unwrap_or(&"https://5124.mywire.org/HDD/nope.jpg".to_string())).to_string() // full size image
    };
    // sample size image, return fullsize if there's no sample.
    let sample_size = if let Some(u) = &choice.sample_url {
        if booru.url == "furry.booru.org" || booru.url == "realbooru.com" || booru.url == "safebooru.org" {
            let status = reqwest::get(u).await?.status();
            if status == 404 {
                u.replace(".png",  ".jpg")
            } else {
                u.to_owned()
            }
        } else {
            u.to_owned()
        }
    } else {
        full_size.clone()
    };
    
    // Sets the score the post has. this score is basically how many favorites the post has.
    let mut score = choice.actual_score.unwrap_or("0".to_string()).to_string();
    if score == "".to_string() {
        score = "0".to_string();
    }
    let score_string = score.to_string();
    // Changes the single letter ratings into the more descriptive names.
    let rating = match &choice.rating.unwrap_or_default()[..] {
        "s" => "Safe".to_string(),
        "q" => "Questionable".to_string(),
        "e" => "Explicit".to_string(),
        _ => String::new(),
    };

    // Addes a source field to the embed if available.
    let mut fields = vec![
        ("Rating", &rating, true),
        ("Score", &score_string, true),
    ];

    // Check if there's a source to get added to the fields.
    let text;
    if let Some(s) = &choice.source {
        if s != &"".to_string() {
            text = format!("[Here]({})", &s);
            fields.push(("Source", &text, true));
        }
    }

    // https://github.com/serenity-rs/serenity/blob/current/examples/11_create_message_builder/src/main.rs
    // builds a message with an embed containing any data used.
    msg.channel_id.send_message(&ctx, |m| { // say method doesn't work for the message builder.
        m.embed( |e| {
            e.description(format!("[Sample]({}) | [Full Size]({})", &sample_size, &full_size));
            e.image(sample_size);
            e.fields(fields);

            e
        });

        m
    }).await?;


    Ok(())
}

/// Sends a random picture from the first page of the booru selected with the specific tags.
/// Usage: `.booru_name tag tag tag`
/// 
/// ```
/// idol feet -x stockings
/// e621 paws
/// chan -x
/// r32
/// ```
/// 
/// The currently available boorus are:
/// __Working:__
/// `SafeBooru` - Safe only booru.
/// `SankakuChan` - Largest, most popular booru, limited to 4 tags.
/// `GelBooru` - One of the most popular boorus.
/// `KonaChan` - Quality Moderated, Girls only booru.
/// `YandeRe` - Quality Moderated booru.
/// `Rule34` - If it exist, there's porn of it.
/// `DanBooru` - Very popular booru, limited to only 2 tags.
/// `HypnoBooru` - A booru that hosts all sorts of hypno based content.
/// `FurryBooru` - Second largest Furry booru.
/// `e621` - Largest Furry booru.
/// `RealBooru` - Very large IRL booru.
/// `IdolComplex` - Largest IRL booru, very asian based.
/// 
/// __Broken:__
/// `Behoimi` - IRL, Mostly cosplays booru.
/// Broken due to access restrictions.
///
/// ----------
///
/// Available parameters:
/// `-x` Explicit
/// `-q` Questionable
/// `-s` Safe. 
/// `-n` Non Safe (Random between E or Q)
/// 
/// Inspired by -GN's [WaifuBot](https://github.com/isakvik/waifubot/)
#[command]
#[usage("test")]
#[usage("testing")]
#[aliases("picture", "pic", "booru", "boorus")]
pub async fn booru_command(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // open the context data lock in read mode.
    let data = ctx.data.read().await;
    // get the database connection from the context data.
    let pool = data.get::<ConnectionPool>().unwrap();
    // get the list of booru commands.
    let commands = data.get::<BooruCommands>();
    // get the data from "boorus.json"
    let boorus = data.get::<BooruList>().unwrap();

    // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let author_id = *msg.author.id.as_u64() as i64; 

    // read from the database, and obtain the booru from the user.
    let data = sqlx::query!("SELECT booru FROM best_bg WHERE user_id = $1", author_id)
        .fetch_optional(pool)
        .boxed()
        .await?;

    // get the booru and tags from the database.
    let mut booru = if let Some(result) = data {
        Some(result.booru.unwrap())
    } else {
        None
    };

    // if the user doesn't have a configured default booru, default to sankaku.
    if booru == None {
        booru = Some("default".to_string());
    }

    // unwrap the option from tags and booru.
    let booru = booru.unwrap();

    // if the preffered booru is idol, invoke the idol command.
    if booru == "idol" {
        idol(&mut ctx.clone(), &msg, args).await?;
    // if the preffered booru is sankaku or chan, invoke the chan command.
    } else if booru == "sankaku" || booru == "chan" {
        chan(&mut ctx.clone(), &msg, args).await?;
    // if the command is a part of the boorus.json file, invoke the get_booru() function.
    } else if commands.as_ref().unwrap().contains(&booru.to_string()) {
        // obtain the rest of the data from the boorus.json file, of the specific booru.
        let b: Booru = {
            let mut x = Booru::default();

            for b in boorus {
                if b.names.contains(&booru.to_string()) {
                    x = b.clone()
                }
            }
            x
        };
        // invoke the get_booru command with the args and booru.
        get_booru(&mut ctx.clone(), &msg, &b, args).await?;
    // else, the booru they have configured is not supported or it's not configured, so we default to chan.
    } else {
        let msg = if booru == "default" {
            msg.reply(&ctx, "No configured booru was found. Defaulting to SankakuChan").await?
        } else {
            msg.reply(&ctx, "An invalid booru name was found. Defaulting to SankakuChan").await?
        };
        chan(&mut ctx.clone(), &msg, args).await?;
        msg.delete(&ctx).await?;
    }

    Ok(())
}

/// Sends a picture of your best girl!
/// 
/// You can configure your best girl with this command:
/// `config user best_girl {booru tags of your best girl}`
/// Usage: `bestgirl <additional tags>`
#[command]
#[aliases(bg, bestgirl, waifu, wife)]
pub async fn best_girl(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // open the context data lock in read mode.
    let data = ctx.data.read().await;
    // get the database connection from the context data.
    let pool = data.get::<ConnectionPool>().unwrap();
    // get the list of booru commands.
    let commands = data.get::<BooruCommands>();
    // get the data from "boorus.json"
    let boorus = data.get::<BooruList>().unwrap();

    // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let author_id = *msg.author.id.as_u64() as i64; 

    // read from the database, and obtain the best girl and booru from the user.
    let data = sqlx::query!("SELECT best_girl, booru FROM best_bg WHERE user_id = $1", author_id)
        .fetch(pool).boxed().try_next().await?;

    let (tags, mut booru);

    // if the data is not empty, aka if the user is on the database already, tell them to get one.
    let row = if let Some(x) = data {
        x
    } else {
        msg.reply(&ctx, "You don't have any waifu :(\nBut don't worry! You can get one using `.conf user best_girl your_best_girl_tag`").await?;
        return Ok(());
    };

    if row.best_girl == None {
        msg.reply(&ctx, "You don't have any waifu :(\nBut don't worry! You can get one using `.conf user best_girl your_best_girl_tag`").await?;
        return Ok(());
    }

    // get the booru and tags from the database.
    tags = row.best_girl;
    booru = row.booru;

    // if the user doesn't have a configured default booru, default to sankaku.
    if booru == None {
        booru = Some("sankaku".to_string());
    }

    // unwrap the option from tags and booru.
    let mut tags = tags.unwrap();
    let booru = booru.unwrap();

    {
        // get the first tag from the tags.
        let mut name = (*tags.split(' ').collect::<Vec<&str>>().first().unwrap()).to_string();
        // if the tag has a copyright, format it as such.
        name = name.replace("_(", " from ");
        // remove the last ) in case of having a copyright.
        name = name.replace(")", "");
        // replace all the _ with spaces.
        name = name.replace("_", " ");
        // add an exclamation mark to the end.
        name += "!";
        // output should look like "Kou from granblue fantasy!" from the original "koy_(granblue_fantasy)"
        msg.channel_id.say(&ctx, capitalize_first(&name).await).await?;
    }

    // combine the command arguments with the saved tags on the database.
    let a = args.message();
    tags = format!("{} {}", a, tags);

    // create an Args object with the tags.
    let args_tags = Args::new(&tags, &[Delimiter::Single(' ')]);

    // if the preffered booru is idol, invoke the idol command.
    if booru == "idol" {
        idol(&mut ctx.clone(), &msg, args_tags).await?;
    // if the preffered booru is sankaku or chan, invoke the chan command.
    } else if booru == "sankaku" || booru == "chan" {
        chan(&mut ctx.clone(), &msg, args_tags).await?;
    // if the command is a part of the boorus.json file, invoke the get_booru() function.
    } else if commands.as_ref().unwrap().contains(&booru.to_string()) {
        // obtain the rest of the data from the boorus.json file, of the specific booru.
        let b: Booru = {
            let mut x = Booru::default();

            for b in boorus {
                if b.names.contains(&booru.to_string()) {
                    x = b.clone()
                }
            }
            x
        };
        // invoke the get_booru command with the args and booru.
        get_booru(&mut ctx.clone(), &msg, &b, args_tags).await?;
    // else, the booru they have configured is not supported, so we default to chan.
    } else {
        msg.reply(&ctx, "An invalid booru name was found. Defaulting to SankakuChan").await?;
        chan(&mut ctx.clone(), &msg, args_tags).await?;
    }

    Ok(())
}

/// Sends a picture of your best boy!
/// 
/// You can configure your best boy with this command:
/// `config user best_boy {booru tags of your best boy}`
/// Usage: `bestboy <additional tags>`
#[command]
#[aliases(bb, bestboy, husbando, husband)]
pub async fn best_boy(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // open the context data lock in read mode.
    let data = ctx.data.read().await;
    // get the database connection from the context data.
    let pool = data.get::<ConnectionPool>().unwrap();
    // get the list of booru commands.
    let commands = data.get::<BooruCommands>();
    // get the data from "boorus.json"
    let boorus = data.get::<BooruList>().unwrap();

    // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let author_id = *msg.author.id.as_u64() as i64; 

    // read from the database, and obtain the best boy and booru from the user.
    let data = sqlx::query!("SELECT best_boy, booru FROM best_bg WHERE user_id = $1", author_id)
        .fetch(pool).boxed().try_next().await?;

    let (tags, mut booru);

    // if the data is not empty, aka if the user is on the database already, tell them to get one.
    let row = if let Some(x) = data {
        x
    } else {
        msg.reply(&ctx, "You don't have any husbando :(\nBut don't worry! You can obtain one with the power of the internet running the command\n`conf user best_boy your_best_boy_tag`").await?;
        return Ok(());
    };

    if row.best_boy == None {
        msg.reply(&ctx, "You don't have any husbando :(\nBut don't worry! You can obtain one with the power of the internet running the command\n`conf user best_boy your_best_boy_tag`").await?;
        return Ok(());
    }

    // get the booru and tags from the database.
    tags = row.best_boy;
    booru = row.booru;

    // if the user doesn't have a configured default booru, default to sankaku.
    if booru == None {
        booru = Some("sankaku".to_string());
    }

    // unwrap the option from tags and booru.
    let mut tags = tags.unwrap();
    let booru = booru.unwrap();

    {
        // get the first tag from the tags.
        let mut name = (*tags.split(' ').collect::<Vec<&str>>().first().unwrap()).to_string();
        // if the tag has a copyright, format it as such.
        name = name.replace("_(", " from ");
        // remove the last ) in case of having a copyright.
        name = name.replace(")", "");
        // replace all the _ with spaces.
        name = name.replace("_", " ");
        // add an exclamation mark to the end.
        name += "!";
        // output should look like "Kou from granblue fantasy!" from the original "koy_(granblue_fantasy)"
        msg.channel_id.say(&ctx, capitalize_first(&name).await).await?;
    }

    // combine the command arguments with the saved tags on the database.
    let a = args.message();
    tags = format!("{} {}", a, tags);

    // create an Args object with the tags.
    let args_tags = Args::new(&tags, &[Delimiter::Single(' ')]);

    // if the preffered booru is idol, invoke the idol command.
    if booru == "idol" {
        idol(&mut ctx.clone(), &msg, args_tags).await?;
    // if the preffered booru is sankaku or chan, invoke the chan command.
    } else if booru == "sankaku" || booru == "chan" {
        chan(&mut ctx.clone(), &msg, args_tags).await?;
    // if the command is a part of the boorus.json file, invoke the get_booru() function.
    } else if commands.as_ref().unwrap().contains(&booru.to_string()) {
        // obtain the rest of the data from the boorus.json file, of the specific booru.
        let b: Booru = {
            let mut x = Booru::default();

            for b in boorus {
                if b.names.contains(&booru.to_string()) {
                    x = b.clone()
                }
            }
            x
        };
        // invoke the get_booru command with the args and booru.
        get_booru(&mut ctx.clone(), &msg, &b, args_tags).await?;
    // else, the booru they have configured is not supported, so we default to chan.
    } else {
        msg.reply(&ctx, "An invalid booru name was found. Defaulting to SankakuChan").await?;
        chan(&mut ctx.clone(), &msg, args_tags).await?;
    }

    Ok(())
}

/// An nHentai reader within discord!
/// Use the left and right arrow reactions to go to the previous or next page respectively.
/// On search, use ✅ to select the current item.
/// For a structure on how the search works, read [this](https://nhentai.net/info/)
///
/// Usage:
/// Use an id directly:
/// `nhentai 259670`
/// Get a random manga:
/// `nhentai random`
/// Search mangas:
/// `nhentai feet trap`
#[command]
#[aliases(nh, nhentai)]
#[min_args(1)]
pub async fn n_hentai(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(channel) = msg.channel(&ctx).await {
        if !channel.is_nsfw() && !channel.guild().is_none() {
            msg.channel_id.say(&ctx, "This command can only be ran on nsfw channels or in dm's").await?;
            return Ok(());
        }
    }

    let reqwest = ReqwestClient::new();
    let mut bot_msg = msg.channel_id.say(&ctx, "Obtaining manga... Please wait.").await?;

    let urls = {
        let search = args.message();
        if let Ok(num) = search.parse::<u64>() {
            let api_url = format!("https://nhentai.net/api/gallery/{}", num);
            let post_url = format!("https://nhentai.net/g/{}", num);
            let mut map = HashMap::new();
            map.insert("api_url", api_url);
            map.insert("post_url", post_url);
            map
        } else {
            if &search == &"random" {
                let post_url = reqwest.get("https://nhentai.net/random/")
                    .send()
                    .await?
                    .url()
                    .to_string();

                let num = post_url.split('/').into_iter().nth(4).unwrap();
                let api_url = format!("https://nhentai.net/api/gallery/{}", num);

                let mut map = HashMap::new();
                map.insert("api_url", api_url.to_string());
                map.insert("post_url", post_url);
                map
            } else {
                let url = ReqUrl::parse_with_params("https://nhentai.net/api/galleries/search",
                    &[("query", search)])?;

                let resp = reqwest.get(url)
                    .send()
                    .await?
                    .json::<NHentaiSearch>()
                    .await?;

                if resp.result.is_empty() {
                    bot_msg.edit(&ctx, |m| m.content("There are no search results.")).await?;
                    return Ok(());
                }

                let mut index = 0;
                let id;

                bot_msg.react(&ctx, "⬅️").await?;
                bot_msg.react(&ctx, "➡️").await?;
                bot_msg.react(&ctx, "✅").await?;

                bot_msg.edit(&ctx, |m| {
                    m.content(&resp.result[index].id);
                    m.embed(|e| {
                        e.title(&resp.result[index].title.english);
                        e.url(format!("https://nhentai.net/g/{}/", &resp.result[index].id));
                        e.description(format!("{}", {
                            let mut tags = resp.result[index].tags.iter().map(|t| format!("{}, ", t.name)).collect::<String>();
                            tags.pop(); tags.pop();
                            tags
                        }));

                        let format = {
                            let t = &resp.result[index].images.cover.t;
                            if t == &"j".to_string() {
                                "jpg"
                            } else if t == &"p".to_string() {
                                "png"
                            } else {
                                "jpg"
                            }
                        };

                        e.image(format!("https://t.nhentai.net/galleries/{}/cover.{}", &resp.result[index].media_id, format));
                        e.footer(|f| {
                            f.text(format!("{} Pages | {} Favs", &resp.result[index].num_pages, &resp.result[index].num_favorites))
                        })
                    })
                }).await?;

                loop {
                    if let Some(reaction) = &bot_msg.await_reaction(&ctx).author_id(msg.author.id.0).timeout(Duration::from_secs(30)).await {
                        let emoji = &reaction.as_inner_ref().emoji;

                        match emoji.as_data().as_str() {
                            "⬅️" => { 
                                if index != 0 {
                                    index -= 1;
                                }
                            },
                            "➡️" => { 
                                if index != resp.result.len() - 1 {
                                    index += 1;
                                }
                            },
                            "✅" => {
                                id = resp.result[index].id.clone();
                                break;
                            },
                            _ => (),
                        }
                        bot_msg.edit(&ctx, |m| {
                            m.content(&resp.result[index].id);
                            m.embed(|e| {
                                e.title(&resp.result[index].title.english);
                                e.url(format!("https://nhentai.net/g/{}/", &resp.result[index].id));
                                e.description(format!("{}", {
                                    let mut tags = resp.result[index].tags.iter().map(|t| format!("{}, ", t.name)).collect::<String>();
                                    tags.pop(); tags.pop();
                                    tags
                                }));

                                let format = {
                                    let t = &resp.result[index].images.cover.t;
                                    if t == &"j".to_string() {
                                        "jpg"
                                    } else if t == &"p".to_string() {
                                        "png"
                                    } else {
                                        "jpg"
                                    }
                                };

                                e.image(format!("https://t.nhentai.net/galleries/{}/cover.{}", &resp.result[index].media_id, format));
                                e.footer(|f| {
                                    f.text(format!("{} Pages | {} Favs", &resp.result[index].num_pages, &resp.result[index].num_favorites))
                                })
                            })
                        }).await?;
                        reaction.as_inner_ref().delete(&ctx).await?;
                    } else {
                        bot_msg.edit(&ctx, |m| {
                            m.content("Timeout")
                        }).await?;
                        ctx.http.edit_message(bot_msg.channel_id.0, bot_msg.id.0, &serde_json::json!({"flags" : 4})).await?;
                        bot_msg.delete_reactions(&ctx).await?;
                        return Ok(());
                    };
                }

                let api_url = format!("https://nhentai.net/api/gallery/{}", id);
                let post_url = format!("https://nhentai.net/g/{}", id);
                let mut map = HashMap::new();
                map.insert("api_url", api_url);
                map.insert("post_url", post_url);
                map
            }
        }
    };

    let resp = reqwest.get(&urls["api_url"])
        .send()
        .await?
        .json::<NHentaiSearchResult>()
        .await?;

    bot_msg.react(&ctx, "⬅️").await?;
    bot_msg.react(&ctx, "➡️").await?;

    let mut index = 0;

    bot_msg.edit(&ctx, |m| {
        m.content(&resp.id);
        m.embed(|e| {
            e.title(&resp.title.pretty);
            e.url(format!("https://nhentai.net/g/{}/", &resp.id));
            e.description(format!("{}", {
                let mut tags = resp.tags.iter().map(|t| format!("{}, ", t.name)).collect::<String>();
                tags.pop(); tags.pop();
                tags
            }));

            let format = {
                let t = &resp.images.cover.t;
                if t == &"j".to_string() {
                    "jpg"
                } else if t == &"p".to_string() {
                    "png"
                } else {
                    "jpg"
                }
            };

            e.image(format!("https://i.nhentai.net/galleries/{}/{}.{}", &resp.media_id, index+1, format));
            e.footer(|f| {
                f.text(format!("{}/{} | {} Favs", index + 1,&resp.num_pages, &resp.num_favorites))
            })
        })
    }).await?;

    loop {
        if let Some(reaction) = &bot_msg.await_reaction(&ctx).author_id(msg.author.id.0).timeout(Duration::from_secs(120)).await {
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "⬅️" => { 
                    if index != 0 {
                        index -= 1;
                    }
                },
                "➡️" => { 
                    if index != resp.images.pages.len() - 1 {
                        index += 1;
                    }
                },
                _ => (),
            }

            reaction.as_inner_ref().delete(&ctx).await?;

            bot_msg.edit(&ctx, |m| {
                m.content(&resp.id);
                m.embed(|e| {
                    e.title(&resp.title.pretty);
                    e.url(format!("https://nhentai.net/g/{}/", &resp.id));
                    e.description("");

                    let format = {
                        let t = &resp.images.pages[index].t;
                        if t == &"j".to_string() {
                            "jpg"
                        } else if t == &"p".to_string() {
                            "png"
                        } else {
                            "jpg"
                        }
                    };

                    e.image(format!("https://i.nhentai.net/galleries/{}/{}.{}", &resp.media_id, index+1, format));
                    e.footer(|f| {
                        f.text(format!("{}/{} | {} Favs", index + 1,&resp.num_pages, &resp.num_favorites))
                    })
                })
            }).await?;

        } else {
            bot_msg.edit(&ctx, |m| {
                m.embed(|e| {
                    e.title(&resp.title.pretty);
                    e.url(format!("https://nhentai.net/g/{}/", &resp.id));
                    e.description(format!("{}", {
                        let mut tags = resp.tags.iter().map(|t| format!("{}, ", t.name)).collect::<String>();
                        tags.pop(); tags.pop();
                        tags
                    }))
                })
            }).await?;
            bot_msg.delete_reactions(&ctx).await?;
            break
        };
    }

    Ok(())
}
