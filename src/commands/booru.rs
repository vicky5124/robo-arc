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
    },
    // import all the types that are used for the global data.
    Booru,
    BooruList,
    BooruCommands,
    DatabaseConnection,
};

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
};
// quick_xml is an xml sedrde library.
// used to deserialize the xml data into structs.
use quick_xml::de::from_str;
// serde or SerializerDeserializer, is a library to srialize data structures into structs.
use serde::{
    Deserialize,
    Serialize,
};

// defining the Post type to be used for the xml deserialized on the Posts vector.
#[derive(Serialize, Deserialize, PartialEq)]
struct Post {
    score: String, // i32
    source: Option<String>,
    rating: String,
    sample_url: Option<String>,
    file_url: Option<String>,
}

// defining the Posts vector to Deserialize the requested xml list.
#[derive(Deserialize, PartialEq)]
struct Posts {
    post: Vec<Post>,
}

// Function to get the booru data and send it.
pub async fn get_booru(ctx: &mut Context, msg: &Message, booru: &Booru, args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: behoimi needs login, e621 changed the api.

    let channel = &ctx.http.get_channel(msg.channel_id.0).await?; // Gets the channel object to be used for the nsfw check.
    // Checks if the command was invoked on a DM
    let dm_channel = if msg.guild_id == None { true } else { false };

    // Obtains a list of tags from the arguments.
    let raw_tags = {
        // if the channel is nsfw or a dm, parse for nsfw tags.
        if channel.is_nsfw().await || dm_channel {
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

    let page: usize = 1;
    let url;

    if booru.typ == 1 {
        url = format!("https://{}/index.php?page=dapi&s=post&q=index&tags={}&pid={}&limit=50", booru.url, tags, page);
    } else if booru.typ == 2 {
        url = format!("https://{}/post/index.xml?tags={}&page={}&limit=50", booru.url, tags, page);
    } else if booru.typ == 3 {
        url = format!("http://{}/post/index.xml?tags={}&page={}&limit=50", booru.url, tags, page);
    } else {
        url = "https://safebooru.org/index.php?page=dapi&s=post&q=index".to_string();
    }

    // Send a request with the parsed url, and return the output text.
    let resp = reqwest.get(&url)
        .headers(headers.clone())
        .send()
        .await?
        .text()
        .await?;

    // deserialize the request XML into the Posts struct.
    let xml: Posts = from_str(&resp.as_str())?;

    // gets a random post from the vector.
    let r = rand::thread_rng().gen_range(0, xml.post.len()); 
    let choice = &xml.post[r];

    // define both url types.
    let full_size = if booru.url != "danbooru.donmai.us" {
        (*choice.file_url.as_ref().unwrap()).to_string() // full size image
    } else {
        (*choice.file_url.as_ref().unwrap_or(&"https://5124.mywire.org/HDD/nope.jpg".to_string())).to_string() // full size image
    };
    // sample size image, return fullsize if there's no sample.
    let sample_size = if let Some(u) = &choice.sample_url {
        if booru.url == "furry.booru.org" || booru.url == "realbooru.com" || booru.url == "safebooru.org" {
            u.replace(".png",  ".jpg")
        } else {
            u.to_owned()
        }
    } else {
        full_size.clone()
    };
    
    // Sets the score the post has. this score is basically how many favorites the post has.
    let mut score = &choice.score.as_str()[..];
    if score == "" {
        score = "0";
    }
    let score_string = score.to_string();
    // Changes the single letter ratings into the more descriptive names.
    let rating = match &choice.rating[..] {
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
/// .idol feet -x stockings
/// .e621 paws
/// .chan -x
/// .r32
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
/// `RealBooru` - Very large IRL booru.
/// `IdolComplex` - Largest IRL booru, very asian based.
/// 
/// __Broken:__
/// `e621` - Largest Furry booru.
/// Broken due to new api.
///
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
#[aliases("picture", "pic", "booru", "boorus")]
pub async fn booru_command(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

/// Sends a picture of your best girl!
/// 
/// You can configure your best girl with this command:
/// `.config user best_girl <booru tag of your best girl>`
#[command]
#[aliases(bg, bestgirl, waifu, wife)]
pub async fn best_girl(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // open the context data lock in read mode.
    let data = ctx.data.read().await;
    // get the database connection from the context data.
    let client = data.get::<DatabaseConnection>().unwrap();
    // get the list of booru commands.
    let commands = data.get::<BooruCommands>();
    // get the data from "boorus.json"
    let boorus = data.get::<BooruList>().unwrap();

    // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let author_id = *msg.author.id.as_u64() as i64; 

    // read from the database, and obtain the best girl and booru from the user.
    let data ={
        let client = client.write().await;
        client.query("SELECT best_girl, booru FROM best_bg WHERE user_id = $1",
                     &[&author_id]).await?
    };
    let (tags, mut booru);

    // if the data is not empty, aka if the user is on the database already, tell them to get one.
    if data.is_empty() {
        msg.reply(&ctx, "You don't have any waifu :(\nBut don't worry! You can get one using `.conf user best_girl your_best_girl_tag`").await?;
        return Ok(());
    }

    // get the booru and tags from the database.
    let row = data.first().unwrap();
    tags = row.get::<_, Option<&str>>(0);
    booru = row.get::<_, Option<&str>>(1);

    // if the user had only a configured booru, but not a best girl, tell the user to get a best girl.
    if tags == None {
        msg.reply(&ctx, "You don't have any waifu :(\nBut don't worry! You can get one using `.conf user best_girl your_best_girl_tag`").await?;
        return Ok(());
    }

    // if the user doesn't have a configured default booru, default to sankaku.
    if booru == None {
        booru = Some("sankaku");
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
    let args_tags = format!("{} {}", a, tags);
    tags = args_tags.as_str();

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
/// `.config user best_boy <booru tag of your best boy>`
#[command]
#[aliases(bb, bestboy, husbando, husband)]
pub async fn best_boy(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // This is the exact same as best_girl, so read the comments of that command instead.
    let data = ctx.data.read().await; // set mutable global data.
    let client = data.get::<DatabaseConnection>().unwrap(); // get the database connection from the global data.
    let commands = data.get::<BooruCommands>();
    let boorus = data.get::<BooruList>().unwrap();

    let author_id = *msg.author.id.as_u64() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let data ={
        let client = client.write().await;
        client.query("SELECT best_boy, booru FROM best_bg WHERE user_id = $1",
                     &[&author_id]).await?
    };
    let (tags, mut booru);

    if data.is_empty() { // if the data is not empty, aka if the user is on the database already
        msg.reply(&ctx, "You don't have any husbando :(\nBut don't worry! You can obtain one with the power of the internet running the command\n`.conf user best_boy your_best_boy_tag`").await?;
        return Ok(());
    } else {
        let row = data.first().unwrap();
        tags = row.get::<_, Option<&str>>(0);
        booru = row.get::<_, Option<&str>>(1);

        if tags == None {
            msg.reply(&ctx, "You don't have any husbando :(\nBut don't worry! You can obtain one with the power of the internet running the command\n`.conf user best_boy your_best_boy_tag`").await?;
            return Ok(());
        }

        if booru == None {
            booru = Some("sankaku");
        } 
    }
    let mut tags = tags.unwrap();
    let booru = booru.unwrap();

    {
        let mut name = tags.split(' ').collect::<Vec<&str>>().first().unwrap().to_string();
        name = name.replace("_(", " from ");
        name = name.replace(")", "");
        name = name.replace("_", " ");
        name += "!";
        msg.channel_id.say(&ctx, capitalize_first(&name).await).await?;
    }

    let a = args.message();
    let args_tags = format!("{} {}", a, tags);
    tags = args_tags.as_str();

    let args_tags = Args::new(&tags, &[Delimiter::Single(' ')]);

    if booru == "idol" {
        idol(&mut ctx.clone(), &msg, args_tags).await?;
    } else if booru == "sankaku" || booru == "chan" {
        chan(&mut ctx.clone(), &msg, args_tags).await?;
    } else if commands.as_ref().unwrap().contains(&booru.to_string()) {
        let b: Booru = {
            let mut x = Booru::default();

            for b in boorus {
                if b.names.contains(&booru.to_string()) {
                    x = b.clone()
                }
            }
            x
        };
        get_booru(&mut ctx.clone(), &msg.clone(), &b, args_tags).await?;
    } else {
        msg.reply(&ctx, "An invalid booru name was found. Defaulting to SankakuChan").await?;
        chan(&mut ctx.clone(), &msg, args_tags).await?;
    }

    Ok(())
}
