use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};

use reqwest::Client as ReqwestClient;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DictionaryElement {
    pub word: String,
    pub phonetic: Option<String>,
    pub origin: Option<String>,
    pub meanings: Vec<Meaning>,
}

#[derive(Debug, Deserialize)]
pub struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: String,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Deserialize)]
pub struct Definition {
    pub definition: String,
    pub synonyms: Option<Vec<String>>,
    pub example: Option<String>,
}

async fn define(ctx: &Context, msg: &Message, lang: &str, word: String) -> CommandResult {
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/{}/{}", lang, word);

    let reqwest = ReqwestClient::new();

    let resp = reqwest.get(&url)
        .send()
        .await?
        .json::<Vec::<DictionaryElement>>()
        .await;

    let definitions = if let Ok(x) = resp { x } else {
        msg.channel_id.say(ctx, "That word does not exist.").await?;
        return Ok(());
    };

    for definition in &definitions {
        dbg!(definition);
    }

    Ok(())
}

#[command]
#[aliases(dict)]
#[min_args(1)]
async fn dictionary(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let lang = args.single_quoted::<String>()?;
    match lang.as_str() {
        "en" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "en", word).await
        },
        "es" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "es", word).await
        },
        "fr" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "fr", word).await
        },
        "ja" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "ja", word).await
        },
        "ru" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "ru", word).await
        },
        "de" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "de", word).await
        },
        "it" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "it", word).await
        },
        "ko" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "ko", word).await
        },
        "ar" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "ar", word).await
        },
        "tr" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "tr", word).await
        },
        "zh" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "zh", word).await
        },
        "hi" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "hi", word).await
        },
        "pt" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "pt", word).await
        },
        _ => {
            let word = lang;
            define(ctx, msg, "en", word).await
        },
    }
}

