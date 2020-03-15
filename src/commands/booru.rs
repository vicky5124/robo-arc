use crate::{
    commands::sankaku::{
        idol,
        chan,
    },
    utils::{
        booru,
        basic_functions::capitalize_first,
    },
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
use rand::Rng;

use reqwest::{
    blocking::Client as ReqwestClient,
    header::*,
};
use quick_xml::de::from_str;
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
    file_url: String,
}

// defining the Posts vector to Deserialize the requested xml list.
#[derive(Deserialize, PartialEq)]
struct Posts {
    post: Vec<Post>,
}

pub fn get_booru(ctx: &mut Context, msg: &Message, booru: &Booru, args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: behoimi needs login

    let channel = &ctx.http.get_channel(msg.channel_id.0)?; // Gets the channel object to be used for the nsfw check.
    // Checks if the command was invoked on a DM
    let dm_channel: bool;
    if msg.guild_id == None {
        dm_channel = true;
    } else {
        dm_channel = false;
    }

    let raw_tags = {
        if channel.is_nsfw() || dm_channel {
            let mut raw_tags = booru::obtain_tags_unsafe(args);
            booru::illegal_check_unsafe(&mut raw_tags)
        } else {
            let mut raw_tags = booru::obtain_tags_safe(args);
            booru::illegal_check_safe(&mut raw_tags)
        }
    };

    let mut tags = raw_tags.iter().map(|x| format!("{}+", x)).collect::<String>();
    tags.pop();

    let reqwest = ReqwestClient::new();
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0".parse().unwrap());

    let page = 1;
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

    let resp = reqwest.get(&url)
        .headers(headers.clone())
        .send()?
        .text()?;
    
    let xml: Posts = from_str(&resp.to_owned()[..])?;

    // gets a random post from the vector.
    let r = rand::thread_rng().gen_range(0, xml.post.len()); 
    let choice = &xml.post[r];

    // define both url types.
    let full_size = &choice.file_url;
    let sample_size = if let Some(u) = &choice.sample_url {
        u.to_owned()
    } else {
        full_size.clone()
    };
    
    // Sets the score and rating for ease of uses
    let score = &choice.score;
    let rating = match &choice.rating[..] {
        "s" => "Safe".to_string(),
        "q" => "Questionable".to_string(),
        "e" => "Explicit".to_string(),
        _ => String::new(),
    };

    // Addes a source field to the embed if available.
    let mut fields = vec![
        ("Rating", &rating, true),
        ("Score", &score, true),
    ];

    // Check if there's a source to get added to the fields.
    let text;
    if let Some(s) = &choice.source {
        if s != &"".to_string() {
            text = format!("[Here]({})", &s);
            &fields.push(("Source", &text, true));
        }
    }

    // https://github.com/serenity-rs/serenity/blob/current/examples/11_create_message_builder/src/main.rs
    msg.channel_id.send_message(&ctx, |m| { // say method doesn't work for the message builder.
        m.embed( |e| {
            e.description(format!("[Sample]({}) | [Full Size]({})", &sample_size, &full_size));
            e.image(sample_size);
            e.fields(fields);

            e
        });

        m
    })?;


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
/// `Behoimi` - IRL, Mostly cosplays booru.
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
pub fn booru_command(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

/// Sends a picture of your best girl!
/// 
/// You can configure your best girl with this command:
/// `.config user best_girl <booru tag of your best girl>`
#[command]
#[aliases(bg, bestgirl, waifu, wife)]
pub fn best_girl(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read(); // set mutable global data.
    let client = data.get::<DatabaseConnection>().unwrap(); // get the database connection from the global data.
    let commands = data.get::<BooruCommands>();
    let boorus = data.get::<BooruList>().unwrap();

    let author_id = msg.author.id.as_u64().clone() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let data ={
        let mut client = client.write();
        client.query("SELECT best_girl, booru FROM best_bg WHERE user_id = $1",
                     &[&author_id])?
    };
    let (tags, mut booru);

    if data.is_empty() { // if the data is not empty, aka if the user is on the database already
        &msg.reply(&ctx, "You don't have any waifu :(\nBut don't worry! You can get one using `.conf user best_girl your_best_girl_tag`")?;
        return Ok(());
    } else {
        let row = data.first().unwrap();
        tags = row.get::<_, Option<&str>>(0);
        booru = row.get::<_, Option<&str>>(1);

        if tags == None {
            &msg.reply(&ctx, "You don't have any waifu :(\nBut don't worry! You can get one using `.conf user best_girl your_best_girl_tag`")?;
            return Ok(());
        }

        if booru == None {
            booru = Some("sankaku");
        } 
    }
    let mut tags = tags.unwrap();
    let booru = booru.unwrap();

    {
        let mut name = tags.split(" ").collect::<Vec<&str>>().first().unwrap().to_string();
        name = name.replace("_(", " from ");
        name = name.replace("_", " ");
        name = name.replace(")", "");
        name += "!";
        &msg.channel_id.say(&ctx, capitalize_first(&name))?;
    }

    let a = args.message();
    let args_tags = format!("{} {}", a, tags);
    tags = args_tags.as_str();

    let args_tags = Args::new(&tags, &[Delimiter::Single(' ')]);

    if booru == "idol" {
        idol(&mut ctx.clone(), &msg, args_tags)?;
    } else if booru == "sankaku" || booru == "chan" {
        chan(&mut ctx.clone(), &msg, args_tags)?;
    } else {
        if commands.as_ref().unwrap().contains(&booru.to_string()) {
            let b: Booru = {
                let mut x = Booru::default();

                for b in boorus {
                    if b.names.contains(&booru.to_string()) {
                        x = b.clone()
                    }
                }
                x
            };
            get_booru(&mut ctx.clone(), &msg.clone(), &b, args_tags)?;
        } else {
            &msg.reply(&ctx, "An invalid booru name was found. Defaulting to SankakuChan")?;
            chan(&mut ctx.clone(), &msg, args_tags)?;
        }
    }

    Ok(())
}

/// Sends a picture of your best boy!
/// 
/// You can configure your best boy with this command:
/// `.config user best_boy <booru tag of your best boy>`
#[command]
#[aliases(bb, bestboy, husbando, husband)]
pub fn best_boy(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read(); // set mutable global data.
    let client = data.get::<DatabaseConnection>().unwrap(); // get the database connection from the global data.
    let commands = data.get::<BooruCommands>();
    let boorus = data.get::<BooruList>().unwrap();

    let author_id = msg.author.id.as_u64().clone() as i64; // get the author_id as a signed 64 bit int, because that's what the database asks for.
    let data ={
        let mut client = client.write();
        client.query("SELECT best_boy, booru FROM best_bg WHERE user_id = $1",
                     &[&author_id])?
    };
    let (tags, mut booru);

    if data.is_empty() { // if the data is not empty, aka if the user is on the database already
        &msg.reply(&ctx, "You don't have any husbando :(\nBut don't worry! You can obtain one with the power of the internet running the command\n`.conf user best_boy your_best_boy_tag`")?;
        return Ok(());
    } else {
        let row = data.first().unwrap();
        tags = row.get::<_, Option<&str>>(0);
        booru = row.get::<_, Option<&str>>(1);

        if tags == None {
            &msg.reply(&ctx, "You don't have any husbando :(\nBut don't worry! You can obtain one with the power of the internet running the command\n`.conf user best_boy your_best_boy_tag`")?;
            return Ok(());
        }

        if booru == None {
            booru = Some("sankaku");
        } 
    }
    let mut tags = tags.unwrap();
    let booru = booru.unwrap();

    {
        let mut name = tags.split(" ").collect::<Vec<&str>>().first().unwrap().to_string();
        name = name.replace("_(", " from ");
        name = name.replace("_", " ");
        name = name.replace(")", "");
        name += "!";
        &msg.channel_id.say(&ctx, capitalize_first(&name))?;
    }

    let a = args.message();
    let args_tags = format!("{} {}", a, tags);
    tags = args_tags.as_str();

    let args_tags = Args::new(&tags, &[Delimiter::Single(' ')]);

    if booru == "idol" {
        idol(&mut ctx.clone(), &msg, args_tags)?;
    } else if booru == "sankaku" || booru == "chan" {
        chan(&mut ctx.clone(), &msg, args_tags)?;
    } else {
        if commands.as_ref().unwrap().contains(&booru.to_string()) {
            let b: Booru = {
                let mut x = Booru::default();

                for b in boorus {
                    if b.names.contains(&booru.to_string()) {
                        x = b.clone()
                    }
                }
                x
            };
            get_booru(&mut ctx.clone(), &msg.clone(), &b, args_tags)?;
        } else {
            &msg.reply(&ctx, "An invalid booru name was found. Defaulting to SankakuChan")?;
            chan(&mut ctx.clone(), &msg, args_tags)?;
        }
    }

    Ok(())
}
