use crate::utils::basic_functions::capitalize_first;

use std::time::Duration;

use serenity::{
    builder::CreateEmbed,
    prelude::Context,
    model::channel::Message,
    model::prelude::ReactionType,
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
    pub part_of_speech: Option<String>,
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

    let mut embeds = Vec::new();

    for definition in &definitions {
        let mut embed = CreateEmbed::default();
        embed.title(capitalize_first(&definition.word));

        if let Some(origin) = &definition.origin {
            if origin != &"".to_string() {
                embed.field("Origin:", &origin, true);
            }
        }

        if let Some(phonetic) = &definition.phonetic {
            if phonetic != &"".to_string() {
                embed.field("Phonetic pronounciation:", &phonetic, true);
            }
        }

        let mut text_definitions = String::new();
        for meaning in &definition.meanings {
            if let Some(pos) = &meaning.part_of_speech {
                if pos != &"".to_string() {
                    text_definitions += &format!("\n\n**{}**:\n", capitalize_first(&pos));
                } else {
                    text_definitions += "\n\n**Unknown**:\n"
                }
            } else {
                text_definitions += "\n\n**Unknown**:\n"
            }

            for definition in &meaning.definitions  {
                text_definitions += "\n**---**\n";
                text_definitions += "- Definition:\n";
                text_definitions += &definition.definition;
                if let Some(example) = &definition.example {
                    if example != &"".to_string() {
                        text_definitions += "\n- Example:\n";
                        text_definitions += &example;
                    }
                }
            }
        }
        embed.description(&text_definitions);

        embeds.push(embed);
    }

    let mut page = 0;

    let mut bot_msg = msg.channel_id.send_message(ctx, |m| m.embed(|mut e| {
        e.0 = embeds[page].0.clone(); e
    })).await?;

    if embeds.len() > 1 {
        let left = ReactionType::Unicode(String::from("⬅️"));
        let right = ReactionType::Unicode(String::from("➡️"));

        let _ = bot_msg.react(ctx, left).await;
        let _ = bot_msg.react(ctx, right).await;

        loop {
            if let Some(reaction) = &bot_msg.await_reaction(ctx).author_id(msg.author.id.0).timeout(Duration::from_secs(120)).await {
                let emoji = &reaction.as_inner_ref().emoji;

                match emoji.as_data().as_str() {
                    "⬅️" => { 
                        if page != 0 {
                            page -= 1;
                        }
                    },
                    "➡️" => { 
                        if page != embeds.len() - 1 {
                            page += 1;
                        }
                    },
                    _ => (),
                }

                bot_msg.edit(ctx, |m| m.embed(|mut e| {
                    e.0 = embeds[page].0.clone(); e
                })).await?;
                let _ = reaction.as_inner_ref().delete(ctx).await;
            } else {
                let _ = bot_msg.delete_reactions(ctx).await;
                break;
            };
        }
    }

    Ok(())
}

/// Gives the definition of a word.
///
/// Usage:
/// `define hello`
/// `define ja こんにちは`
///
/// Supported languages:
/// ```
/// en -> English (Default)
/// es -> Spanish
/// fr -> French
/// it -> Italian
/// de -> German
/// pt -> Brazilian Portuguese
/// ja -> Japanese
/// ko -> Korean
/// zh -> Chinese (Simplified)
/// hi -> Hindi
/// ru -> Russian
/// ar -> Arabic
/// tr -> Turkish
/// ```
#[command]
#[aliases(dict, define)]
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
        "ja" | "jp" => {
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
        "ko" | "kr" => {
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
        "pt" | "br" => {
            let word = args.single_quoted::<String>()?;
            define(ctx, msg, "pt", word).await
        },
        _ => {
            let word = lang;
            define(ctx, msg, "en", word).await
        },
    }
}

