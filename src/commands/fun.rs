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
        let fields = vec![
            ("Definition", parsed_definition, false),
            ("Example", parsed_example, false),
        ];

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
