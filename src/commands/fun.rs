use std::process::Command;
use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        CommandError,
        macros::command,
    },
};
use qrcode::{
    QrCode,
    render::unicode,
};
use reqwest::{
    blocking::Client as ReqwestClient,
    Url,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct UrbanDict {
    definition: String,
    permalink: String,
    thumbs_up: u32,
    thumbs_down: u32,
    author: String,
    written_on: String,
    example: String,
    word: String,
}

#[derive(Deserialize)]
struct UrbanList {
    list: Vec<UrbanDict>
}

#[command]
fn qr(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let words = args.message();

    let code = QrCode::new(words).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    msg.channel_id.say(&ctx, format!(">>> ```{}```", image))?;
    Ok(())
}

#[command]
#[aliases(define)]
fn urban(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let term = args.message();
    let url = Url::parse_with_params("http://api.urbandictionary.com/v0/define",
                                     &[("term", term)])?;

    let reqwest = ReqwestClient::new();
    let resp = reqwest.get(url)
        .send()?
        .json::<UrbanList>()?;

    if resp.list.len() == 0 {
        msg.channel_id.say(&ctx, format!("The term '{}' has no Urban Definitions", term))?;
    } else {
        let choice = &resp.list[0];
        let parsed_definition = &choice.definition.replace("[", "").replace("]", "");
        let parsed_example = &choice.example.replace("[", "").replace("]", "");
        let mut fields = vec![
            ("Definition", parsed_definition, false),
        ];
        if parsed_example != &"".to_string() {
            fields.push(("Example", parsed_example, false));
        }

        if let Err(why) = msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title(&choice.word);
                e.url(&choice.permalink);
                e.description(format!("submitted by **{}**\n\n:thumbsup: **{}** ┇ **{}** :thumbsdown:\n", &choice.author, &choice.thumbs_up, &choice.thumbs_down));
                e.fields(fields);
                e.timestamp(choice.clone().written_on.to_owned());
                e
            });
            m
        }) {
            if "Embed too large.".to_string() == why.to_string() {
                msg.channel_id.say(&ctx, &choice.permalink)?;
            } else {
                return Err(CommandError(why.to_string()));
            }
        };
    }

    Ok(())
}

/// Translates a text to the specified language.
/// Ex: `.translate ja Hello, World!`
#[command]
#[aliases(trans)]
#[min_args(2)]
fn translate(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut dest = args.single::<String>()?;
    let args_text = args.rest();

    dest = match dest.as_str() {
        "jp" => "ja".to_string(),
        "kr" => "ko".to_string(),
        _ => dest,
    };

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .arg("/C")
                .arg(format!("./translate.py \"{}\" {}", &args_text, dest).as_str())
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("sh")
                .arg("-c")
                .arg(format!("./translate.py \"{}\" {}", &args_text, dest).as_str())
                .output()
                .expect("failed to execute process")
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let resp = text.split("'").nth(1).unwrap();

    let fields = vec![
        ("Original Text", &args_text, false),
        ("Translation", &resp, false),
    ];

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.fields(fields);
            e
        });
        m
    })?;
    Ok(())
}

/// Searches a term on duckduckgo.com, for you.
///
/// Usage: `.ddg hello world`
#[command]
#[min_args(1)]
#[aliases(ddg, duck, duckduckgo, search, better_than_google, betterthangoogle)]
fn duck_duck_go(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let url = Url::parse_with_params("https://lmddgtfy.net/",
                                     &[("q", args.message())])?;
    &msg.channel_id.say(&ctx, url)?;

    Ok(())
}

/// Encrypts a message. **NOT WORKING**
/// 15 character limit.
/// Usage: `.encrypt Hello!`
/// 
/// You can decrypt the message with .decrypt
#[command]
#[min_args(1)]
fn encrypt(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let message = args.message();
    let bytes = message.as_bytes();
    let encrypted_bytes = bytes.iter().map(|b| format!("{}", b)).collect::<String>();
    let encrypted_message = encrypted_bytes.parse::<u128>()? << 1;
    &msg.channel_id.say(&ctx, format!("`{:X}`", encrypted_message))?;
    Ok(())
}
/// Decrypts and encrypted message. **NOT WORKING**
/// Usage: `.decrypt FBACB56A78BAFCD8239012F`
#[command]
fn decrypt(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let message = args.message();
    let bytes = message.as_bytes();
    let encrypted_bytes = bytes.iter().map(|b| format!("{}", b)).collect::<String>();
    let encrypted_message = encrypted_bytes.parse::<u128>()? >> 1;
    &msg.channel_id.say(&ctx, format!("`{:X}`", encrypted_message))?;
    Ok(())
}
