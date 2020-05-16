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
#[aliases(dictionary, dict_en, dicten, dictionaryen)]
async fn dictionary_en(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "en", word).await
}
#[command]
#[aliases(dict_es, dictes, dictionaryes)]
async fn dictionary_es(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "es", word).await
}
#[command]
#[aliases(dict_fr, dictfr, dictionaryfr)]
async fn dictionary_fr(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "fr", word).await
}
#[command]
#[aliases(dict_ja, dictja, dictionaryja)]
async fn dictionary_ja(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "ja", word).await
}
#[command]
#[aliases(dict_ru, dictru, dictionaryru)]
async fn dictionary_ru(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "ru", word).await
}
#[command]
#[aliases(dict_de, dictde, dictionaryde)]
async fn dictionary_de(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "de", word).await
}
#[command]
#[aliases(dict_it, dictit, dictionaryit)]
async fn dictionary_it(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "it", word).await
}
#[command]
#[aliases(dict_ko, dictko, dictionaryko)]
async fn dictionary_ko(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "ko", word).await
}
#[command]
#[aliases(dict_ar, dictar, dictionaryar)]
async fn dictionary_ar(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "ar", word).await
}
#[command]
#[aliases(dict_tr, dicttr, dictionarytr)]
async fn dictionary_tr(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "tr", word).await
}
#[command]
#[aliases(dict_zh, dictzh, dictionaryzh)]
async fn dictionary_zh(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "zh", word).await
}
#[command]
#[aliases(dict_hi, dicthi, dictionaryhi)]
async fn dictionary_hi(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "hi", word).await
}
#[command]
#[aliases(dict_pt, dictpt, dictionarypt)]
async fn dictionary_pt(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;
    define(ctx, msg, "pt", word).await
}
