use crate::Tokens;

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
use tracing::error;
use qrcode::{
    QrCode,
    render::unicode,
};
use reqwest::{
    Client as ReqwestClient,
    Url,
};
use serde::Deserialize;
use crypto::{
    symmetriccipher,
    buffer,
    aes,
    blockmodes
};
use crypto::buffer::{
    ReadBuffer,
    WriteBuffer,
    BufferResult
};
use hex;

static KEY: [u8; 32] =  [244, 129, 85, 125, 252, 92, 208, 68, 29, 125, 160, 4, 146, 245, 193, 135, 12, 68, 162, 84, 202, 123, 90, 165, 194, 126, 12, 117, 87, 195, 9, 202];
static IV: [u8; 16] =  [41, 61, 154, 40, 255, 51, 217, 146, 228, 10, 58, 62, 217, 128, 96, 7];

// Struct used to deserialize the output of the urban dictionary api call...
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

// But it returns a list, so we use this for the request.
#[derive(Deserialize)]
struct UrbanList {
    list: Vec<UrbanDict>
}

// Struct used to deserialize the response from the yandex translate api call.
#[derive(Deserialize)]
struct YandexTranslate {
    code: u16,
    lang: Option<String>,
    text: Option<Vec<String>>,
}

/// Sends a qr code of the term mentioned.
/// Usage: `.qr Hello world!`
#[command]
async fn qr(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let words = args.message();

    let code = QrCode::new(words).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    msg.channel_id.say(&ctx, format!(">>> ```{}```", image)).await?;
    Ok(())
}

/// Defines a term, using the urban dictionary.
/// Usage: `urban lmao`
#[command]
#[aliases(define)]
async fn urban(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let term = args.message();
    let url = Url::parse_with_params("http://api.urbandictionary.com/v0/define",
                                     &[("term", term)])?;

    let reqwest = ReqwestClient::new();
    let resp = reqwest.get(url)
        .send()
        .await?
        .json::<UrbanList>()
        .await?;

    if resp.list.is_empty() {
        msg.channel_id.say(&ctx, format!("The term '{}' has no Urban Definitions", term)).await?;
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
                e.timestamp(choice.clone().written_on);
                e
            });
            m
        }).await {
            if "Embed too large." == why.to_string() {
                msg.channel_id.say(&ctx, &choice.permalink).await?;
            } else {
                return Err(CommandError(why.to_string()));
            }
        };
    }

    Ok(())
}

/// Translates a text to the specified language.
///
/// Usage:
///
/// Translate to japanese:
/// `translate ja Hello, World!`
/// Translate from spanish to japanese:
/// `translate es-en Hola!`
#[command]
#[aliases(trans)]
#[min_args(2)]
async fn translate(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let yandex_token = {
        let data_read = ctx.data.read().await;
        let tokens = data_read.get::<Tokens>().unwrap();
        tokens["yandex"].as_str().unwrap().to_string()
    };

    let mut dest = args.single::<String>()?;
    let text = args.rest();

    dest = match dest.as_str() {
        "jp" => "ja".to_string(),
        "kr" => "ko".to_string(),
        _ => dest,
    };

    let url = Url::parse_with_params("https://translate.yandex.net/api/v1.5/tr.json/translate",
                                     &[
                                        ("key", yandex_token),
                                        ("text", text.to_string()),
                                        ("lang", dest),
                                     ])?;

    let reqwest = ReqwestClient::new();
    let resp = reqwest.get(url)
        .send()
        .await?
        .json::<YandexTranslate>()
        .await?;

    if resp.code == 200 {
        let mut fields = vec![
            ("Original Text", text.to_string() + "\n", false),
        ];

        let mut resp_langs = if let Some(l) = &resp.lang {
            l.split('-').into_iter()
        } else {
            msg.channel_id.say(&ctx, "An invalid destination language was given").await?;
            return Ok(());
        };

        for translated_text in &resp.text.unwrap() {
            fields.push(("Translation", translated_text.to_string(), false));
        }


        msg.channel_id.send_message(&ctx, |m| {
            m.content(format!("From **{}** to **{}**", resp_langs.next().unwrap(), resp_langs.next().unwrap()));
            m.embed(|e| {
                e.fields(fields)
            })
        }).await?;
    } else if resp.code == 404 {
        msg.channel_id.say(&ctx, "The daily translation limit was exceeded.").await?;
    } else if resp.code == 413 {
        msg.channel_id.say(&ctx, "The text length limit was exceeded.").await?;
    } else if resp.code == 422 {
        msg.channel_id.say(&ctx, "The text could not be translated.").await?;
    } else if resp.code == 501 {
        msg.channel_id.say(&ctx, "The specified target language is not supported.").await?;
    } else if resp.code == 502 {
        msg.channel_id.say(&ctx, "The specified language doesn't exist.").await?;
    } else {
        msg.channel_id.say(&ctx, "An unhandled error happened.").await?;
    }

    Ok(())
}

/// Searches a term on duckduckgo.com, for you.
///
/// Usage: `ddg hello world`
#[command]
#[min_args(1)]
#[aliases(ddg, duck, duckduckgo, search, better_than_google, betterthangoogle)]
async fn duck_duck_go(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let url = Url::parse_with_params("https://lmddgtfy.net/",
                                     &[("q", args.message())])?;
    msg.channel_id.say(&ctx, url).await?;

    Ok(())
}

fn encrypt_bytes(data: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let mut encryptor = aes::cbc_encryptor(
            aes::KeySize::KeySize256,
            &KEY,
            &IV,
            blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

fn decrypt_bytes(encrypted_data: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize256,
            &KEY,
            &IV,
            blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}


/// Encrypts a message.
/// You can decrypt the message with `decrypt {hex_hash}`
/// 
/// Usage: `encrypt Jaxtar is Cute!`
#[command]
#[min_args(1)]
async fn encrypt(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let message = args.message();

    let encrypted_data = encrypt_bytes(message.as_bytes()).ok().unwrap();
    let encrypted_data_text = hex::encode(encrypted_data.to_vec());

    msg.channel_id.say(&ctx, format!("`{}`", encrypted_data_text)).await?;
    Ok(())
}

/// Decrypts and encrypted message.
///
/// Usage: `decrypt 36991e919634f4dc933787de47e9cb37`
#[command]
async fn decrypt(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let message = args.message();

    let encrypted_data = hex::decode(&message)?;
    let decrypted_data_bytes = match decrypt_bytes(&encrypted_data[..]) {
        Ok(ok) => ok,
        Err(why) => {
            error!(why);
            msg.channel_id.say(&ctx, format!("An invalid hash was provided. `{:?}`", why)).await?;
            return Ok(());
        },
    };

    let decrypted_data_text = String::from_utf8(decrypted_data_bytes)?;

    msg.channel_id.send_message(&ctx, |m| m.embed(|e| {
        e.title(format!("From `{}`", message));
        e.description(decrypted_data_text)
    })).await?;
    Ok(())
}
