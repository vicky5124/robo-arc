use crate::{
    Tokens,
    ConnectionPool,
    commands::moderation::parse_member,
    utils::basic_functions::string_to_seconds,
};

use std::time::Duration;
use std::fs;

use serenity::{
    prelude::Context,
    model::misc::Mentionable,
    model::channel::{
        Message,
        ReactionType,
    },
    model::user::User,
    model::id::UserId,
    model::id::GuildId,
    framework::standard::{
        Args,
        CommandResult,
        CheckResult,
        CommandError,
        macros::{
            command,
            check,
        },
    },
};

use serde::Deserialize;
use tracing::error;
use qrcode::{
    QrCode,
    render::unicode,
};
use reqwest::{
    Client as ReqwestClient,
    Url,
};

use hex;
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

use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

use fasteval::error::Error;

static KEY: [u8; 32] =  [244, 129, 85, 125, 252, 92, 208, 68, 29, 125, 160, 4, 146, 245, 193, 135, 12, 68, 162, 84, 202, 123, 90, 165, 194, 126, 12, 117, 87, 195, 9, 202];
static IV: [u8; 16] =  [41, 61, 154, 40, 255, 51, 217, 146, 228, 10, 58, 62, 217, 128, 96, 7];

#[check]
#[name = "bot_has_manage_messages"]
async fn bot_has_manage_messages_check(ctx: &Context, msg: &Message) -> CheckResult {
    let bot_id = ctx.cache.current_user().await.id.0;
    let err = CheckResult::new_user("I'm unable to run this command due to missing the `Manage Messages` permission.");
    if let Some(guild) = msg.channel(ctx)
        .await
        .unwrap()
        .guild()
    {
        if !guild.permissions_for_user(ctx, bot_id)
            .await
            .expect("what even")
            .manage_messages()
        { err }

        else {
            CheckResult::Success
        }
    }
    else {
        err
    }
}

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
async fn qr(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let words = args.message();

    let code = QrCode::new(words).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    msg.channel_id.say(ctx, format!(">>> ```{}```", image)).await?;
    Ok(())
}

/// Defines a term, using the urban dictionary.
/// Usage: `urban lmao`
#[command]
#[aliases(udic, udefine, define_urban, defineurban, udict, udictonary, urban_dictionary, u_dictionary, u_define, urban_define, define_urban)]
async fn urban(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
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
        msg.channel_id.say(ctx, format!("The term '{}' has no Urban Definitions", term)).await?;
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

        if let Err(why) = msg.channel_id.send_message(ctx, |m| {
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
                msg.channel_id.say(ctx, &choice.permalink).await?;
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
async fn translate(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
            msg.channel_id.say(ctx, "An invalid destination language was given").await?;
            return Ok(());
        };

        for translated_text in &resp.text.unwrap() {
            fields.push(("Translation", translated_text.to_string(), false));
        }


        msg.channel_id.send_message(ctx, |m| {
            m.content(format!("From **{}** to **{}**", resp_langs.next().unwrap(), resp_langs.next().unwrap()));
            m.embed(|e| {
                e.fields(fields)
            })
        }).await?;
    } else if resp.code == 404 {
        msg.channel_id.say(ctx, "The daily translation limit was exceeded.").await?;
    } else if resp.code == 413 {
        msg.channel_id.say(ctx, "The text length limit was exceeded.").await?;
    } else if resp.code == 422 {
        msg.channel_id.say(ctx, "The text could not be translated.").await?;
    } else if resp.code == 501 {
        msg.channel_id.say(ctx, "The specified target language is not supported.").await?;
    } else if resp.code == 502 {
        msg.channel_id.say(ctx, "The specified language doesn't exist.").await?;
    } else {
        msg.channel_id.say(ctx, "An unhandled error happened.").await?;
    }

    Ok(())
}

/// Searches a term on duckduckgo.com, for you.
///
/// Usage: `ddg hello world`
#[command]
#[min_args(1)]
#[aliases(ddg, duck, duckduckgo, search, better_than_google, betterthangoogle)]
async fn duck_duck_go(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let url = Url::parse_with_params("https://lmddgtfy.net/",
                                     &[("q", args.message())])?;
    msg.channel_id.say(ctx, url).await?;

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
async fn encrypt(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let message = args.message();

    let encrypted_data = encrypt_bytes(message.as_bytes()).ok().unwrap();
    let encrypted_data_text = hex::encode(encrypted_data.to_vec());

    msg.channel_id.say(ctx, format!("`{}`", encrypted_data_text)).await?;
    Ok(())
}

/// Decrypts and encrypted message.
///
/// Usage: `decrypt 36991e919634f4dc933787de47e9cb37`
#[command]
async fn decrypt(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let message = args.message();

    let encrypted_data = hex::decode(&message)?;
    let decrypted_data_bytes = match decrypt_bytes(&encrypted_data[..]) {
        Ok(ok) => ok,
        Err(why) => {
            error!("{:?}", why);
            msg.channel_id.say(ctx, format!("An invalid hash was provided. `{:?}`", why)).await?;
            return Ok(());
        },
    };

    let decrypted_data_text = String::from_utf8(decrypted_data_bytes)?;

    msg.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.title(format!("From `{}`", message));
        e.description(decrypted_data_text)
    })).await?;
    Ok(())
}

async fn place_piece<'a>(board: &mut Vec<Vec<&'a str>>, user: &User, piece: &'a str, ctx: &Context) -> Result<(), ()> {
    'outer: loop {
        let mut x: Option<usize> = None;
        let mut y: Option<usize> = None;
        loop {
            if x.is_none() || y.is_none() {
                if let Some(reaction) = user.await_reaction(ctx).timeout(Duration::from_secs(120)).await {
                    let _ = reaction.as_inner_ref().delete(ctx).await;
                    let emoji = &reaction.as_inner_ref().emoji;

                    match emoji.as_data().as_str() {
                        "1\u{fe0f}\u{20e3}" => y = Some(0),
                        "2\u{fe0f}\u{20e3}" => y = Some(1),
                        "3\u{fe0f}\u{20e3}" => y = Some(2),
                        "\u{01f1e6}" => x = Some(0),
                        "\u{01f1e7}" => x = Some(1),
                        "\u{01f1e8}" => x = Some(2),
                        _ => ()
                    }
                } else {
                    return Err(());
                }
            } else {
                if !x.is_none() && !y.is_none() {
                    if board[y.unwrap()][x.unwrap()] == " " {
                        board[y.unwrap()][x.unwrap()] = piece;
                        break 'outer;
                    } else {
                        x = None;
                        y = None;
                    }
                }
            }
        }
    }

    Ok(())
}

fn check_win<'a>(board: &Vec<Vec<&'a str>>) -> Option<&'a str> {
    // diagonal \
    if board[0][0] == "O" && board[1][1] == "O" && board[2][2] == "O" {
        return Some("O");
    } else if board[0][0] == "X" && board[1][1] == "X" && board[2][2] == "X" {
        return Some("X");

    // diagonal /
    } else if board[2][0] == "O" && board[1][1] == "O" && board[0][2] == "O" {
        return Some("O");
    } else if board[2][0] == "X" && board[1][1] == "X" && board[0][2] == "X" {
        return Some("X");

    // straight lines ---
    } else if board[0] == vec!["O", "O", "O"] {
        return Some("O");
    } else if board[1] == vec!["O", "O", "O"] {
        return Some("O");
    } else if board[2] == vec!["O", "O", "O"] {
        return Some("O");
    } else if board[0] == vec!["X", "X", "X"] {
        return Some("X");
    } else if board[1] == vec!["X", "X", "X"] {
        return Some("X");
    } else if board[2] == vec!["X", "X", "X"] {
        return Some("X");

    // straigt lines |
    } else if board[0][0] == "O" && board[1][0] == "O" && board[2][0] == "O" {
        return Some("O");
    } else if board[0][1] == "O" && board[1][1] == "O" && board[2][1] == "O" {
        return Some("O");
    } else if board[0][2] == "O" && board[1][2] == "O" && board[2][2] == "O" {
        return Some("O");
    } else if board[0][0] == "X" && board[1][0] == "X" && board[2][0] == "X" {
        return Some("X");
    } else if board[0][1] == "X" && board[1][1] == "X" && board[2][1] == "X" {
        return Some("X");
    } else if board[0][2] == "X" && board[1][2] == "X" && board[2][2] == "X" {
        return Some("X");
    }


    None 
}

fn format_board(board: &Vec<Vec<&str>>) -> String {
    let mut lines = "```X | A   B   C\n--------------\n".to_string();

    for (i, x) in board.iter().enumerate() {
        let line = format!("{} | {} | {} | {}", i+1, x[0], x[1], x[2]);
        lines += format!("{}\n", line).as_str();
    }
    lines += "\nY```";
    lines
}

/// 2 player game where you must compete with the other player to be the first to obtain 3 of your pieces in line.
/// 
/// X is --- / Horizontal
/// Y is ||| / Vertical
///
/// When it's your turn, react with a number and a letter, corresponding to the position of the board.
/// If the place is taken, you will need to repick the position.
///
/// Is there an AI to play by myself? No, you have to play with another player.
///
/// Usage:
/// `ttt @timmy`
#[command]
#[aliases(ttt, tictactoe)]
#[checks("bot_has_manage_messages")]
#[min_args(1)]
async fn tic_tac_toe(mut ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user1 = &msg.author;
    let user2 = parse_member(&mut ctx, &msg, args.single::<String>()?).await?;

    let mut confirmation = msg.channel_id.say(ctx, format!("{}: Do you accept this TicTacToe match?", user2.mention())).await?;
    confirmation.react(ctx, '✅').await?;
    confirmation.react(ctx, '❌').await?;

    loop {
        if let Some(reaction) = user2.user.await_reaction(ctx).timeout(Duration::from_secs(120)).await {
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "✅" => {
                    confirmation.delete(ctx).await?;
                    break;
                },
                "❌" => {
                    confirmation.edit(ctx, |m| m.content(format!("{}: {} didn't accept the match.", user1.mention(), user2.mention()))).await?;
                    return Ok(());
                },
                _ => ()
            }
        } else {
            confirmation.edit(ctx, |m| m.content(format!("{}: {} took to long to respond.", user1.mention(), user2.mention()))).await?;
            return Ok(());
        }
    }

    let users = {
        if msg.timestamp.timestamp() % 2 == 0 {
            (user1, &user2.user)
        } else {
            (&user2.user, user1)
        }
    };

    let mut iteration = 0u8;
    let mut board = vec![
        vec![" ", " ", " "],
        vec![" ", " ", " "],
        vec![" ", " ", " "],
    ];

    let b = format_board(&board);
    let mut m = msg.channel_id.send_message(ctx, |m| {
        m.content(format!("{} (X): Select the position for your piece.", users.0.id.mention()));
        m.embed(|e| {
            e.description(&b)
        })
    }).await?;

    for i in 1..4 {
        let num = ReactionType::Unicode(String::from(format!("{}\u{fe0f}\u{20e3}", i)));
        m.react(ctx, num).await?;
    }

    let _a = ReactionType::Unicode(String::from("\u{01f1e6}"));
    let _b = ReactionType::Unicode(String::from("\u{01f1e7}"));
    let _c = ReactionType::Unicode(String::from("\u{01f1e8}"));

    m.react(ctx, _a).await?;
    m.react(ctx, _b).await?;
    m.react(ctx, _c).await?;

    loop {
        if let Err(_) = place_piece(&mut board, &users.0, "X", ctx).await {
            m.edit(ctx, |m| {
                m.content("Timeout.");
                m.embed(|e| {
                    e.description(&b)
                })
            }).await?;
        };

        let b = format_board(&board);
        m.edit(ctx, |m| {
            m.content(format!("{} (O): Select the position for your piece.", users.1.id.mention()));
            m.embed(|e| {
                e.description(&b)
            })
        }).await?;

        let won = check_win(&board);

        if iteration == 4 {
            if let Some(win) = won {
                if win == "X" {
                    m.edit(ctx, |m| {
                        m.content(format!("{} (X) won!", users.0.id.mention()));
                        m.embed(|e| {
                            e.description(&b)
                        })
                    }).await?;
                } else {
                    m.edit(ctx, |m| {
                        m.content(format!("{} (O) won!", users.1.id.mention()));
                        m.embed(|e| {
                            e.description(&b)
                        })
                    }).await?;
                }
            } else {
                m.edit(ctx, |m| {
                    m.content(format!("{} and {} tied.", users.0.id.mention(), users.1.id.mention()));
                    m.embed(|e| {
                        e.description(&b)
                    })
                }).await?;
            }
            m.delete_reactions(ctx).await?;
            break;
        } else {
            if let Some(win) = won {
                if win == "X" {
                    m.edit(ctx, |m| {
                        m.content(format!("{} (X) won!", users.0.id.mention()));
                        m.embed(|e| {
                            e.description(&b)
                        })
                    }).await?;
                } else {
                    m.edit(ctx, |m| {
                        m.content(format!("{} (O) won!", users.1.id.mention()));
                        m.embed(|e| {
                            e.description(&b)
                        })
                    }).await?;
                }
                m.delete_reactions(ctx).await?;
                break;
            }
        }

        if let Err(_) = place_piece(&mut board, &users.1, "O", ctx).await {
            m.edit(ctx, |m| {
                m.content("Timeout.");
                m.embed(|e| {
                    e.description(b)
                })
            }).await?;
            m.delete_reactions(ctx).await?;
            break;
        };

        let b = format_board(&board);
        m.edit(ctx, |m| {
            m.content(format!("{} (X): Select the position for your piece.", users.0.id.mention()));
            m.embed(|e| {
                e.description(&b)
            })
        }).await?;

        let won = check_win(&board);
        if let Some(win) = won {
            if win == "X" {
                m.edit(ctx, |m| {
                    m.content(format!("{} (X) won!", users.0.id.mention()));
                    m.embed(|e| {
                        e.description(&b)
                    })
                }).await?;
            } else {
                m.edit(ctx, |m| {
                    m.content(format!("{} (O) won!", users.1.id.mention()));
                    m.embed(|e| {
                        e.description(&b)
                    })
                }).await?;
            }
            m.delete_reactions(ctx).await?;
            break;
        }
        
        iteration += 1;
    }

    Ok(())
}

/// Play some Higher or Lower.
/// You don't get anything in reward for playing this, gambling is bad.
#[command]
#[aliases(hol, higherorlower)]
async fn higher_or_lower(ctx: &Context, msg: &Message) -> CommandResult {
    let cards = fs::read_dir("poker_cards")?.map(|i| {
        let f = i.unwrap();
        f.file_name().into_string().unwrap()
    }).collect::<Vec<String>>();

    let mut rng = StdRng::from_entropy();
    let choice = &cards.choose(&mut rng).unwrap();

    let mut message = msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Higher or Lower");
            e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
        })
    }).await?;

    let up = ReactionType::Unicode("⬆️".to_string());
    let down = ReactionType::Unicode("⬇️".to_string());

    message.react(ctx, up).await?;
    message.react(ctx, down).await?;

    let mut iteration = 1u8;
    let mut current_value = choice.split('.').next().unwrap()[1..].parse::<u8>()?;

    loop {
        if let Some(reaction) = message.await_reaction(ctx).author_id(msg.author.id).timeout(Duration::from_secs(120)).await {
            let emoji = &reaction.as_inner_ref().emoji;
            let emoji_data = emoji.as_data();
            let emoji_str = emoji_data.as_str();

            match emoji_str {
                "⬆️" | "⬇️" => {
                    let higher = emoji_str == "⬆️";

                    let choice = &cards.choose(&mut rng).unwrap();
                    let new_value = choice.split('.').next().unwrap()[1..].parse::<u8>()?;


                    if higher {
                        if new_value < current_value {
                            message.edit(ctx, |m| {
                                m.embed(|e| {
                                    e.title(format!("{} lost.", msg.author.name));
                                    e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                                })
                            }).await?;

                            break
                        }
                    } else {
                        if new_value > current_value {
                            message.edit(ctx, |m| {
                                m.embed(|e| {
                                    e.title(format!("{} lost.", msg.author.name));
                                    e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                                })
                            }).await?;

                            break 
                        }
                    }

                    current_value = new_value;

                    iteration += 1;

                    if iteration > 3 {
                        message.edit(ctx, |m| {
                            m.embed(|e| {
                                e.title(format!("{} won!", msg.author.name));
                                e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                            })
                        }).await?;

                        break 
                    } else {
                        message.edit(ctx, |m| {
                            m.embed(|e| {
                                e.title("Higher or Lower");
                                e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                            })
                        }).await?;
                    }
                },
                _ => (),
            }
        } else {
            message.edit(ctx, |m| {
                m.embed(|e| {
                    e.title("Timeout!")
                })
            }).await?;
            break
        }
    }

    let _ = message.delete_reactions(ctx).await;

    Ok(())
}

/// Shows the information of a user.
/// (not bound to a guild)
#[command]
#[aliases(pfp, avatar, discord_profile, prof, user, u)]
async fn profile(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = if let Ok(user_id) = args.single_quoted::<UserId>() {
        user_id.to_user(ctx).await?
    } else {
        msg.author.clone()
    };

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            if user.bot {
                e.title(format!("[BOT] {}", user.tag(),));
            } else {
                e.title(user.tag());
            }

            e.field("ID:", user.id.0, false);
            e.field("Created at:", format!("{}UTC\n({} ago)", user.created_at().to_rfc2822().replace("+0000", ""), {
                let date = chrono::Utc::now();
                let time = date.timestamp() - user.created_at().timestamp();
                let duration = Duration::from_secs(time as u64);
                humantime::format_duration(duration)
            }), false);

            e.image(user.face())
        })
    }).await?;

    Ok(())
}

/// Calculates an expression.
///
/// Example: `calc 1+2*3/4^5%6 + log(100K) + log(e(),100) + [3*(3-3)/3] + (2<3) && 1.23`
///
/// The precise integer limit is the signed 32 bit integer (-2147483648 to 2147483647)
/// The the unprecise integer limit is almost the signed 1024 bit integer.
/// The floating point precision is 64 bit.
///
/// Supported operators:
/// ```
/// +               Addition
/// -               Subtraction
/// *               Multiplication
/// /               Division
/// %               Modulo
/// ^ **            Exponentiation
/// && (and)        Logical AND with short-circuit
/// || (or)         Logical OR with short-circuit
/// == != < <= >= > Comparisons (all have equal precedence)
///
/// ---------------
///
/// Integers: 1, 2, 10, 100, 1001
/// 
/// Decimals: 1.0, 1.23456, 0.000001
/// 
/// Exponents: 1e3, 1E3, 1e-3, 1E-3, 1.2345e100
/// 
/// Suffix:
/// 1.23p       = 0.00000000000123
/// 1.23n       = 0.00000000123
/// 1.23µ 1.23u = 0.00000123
/// 1.23m       = 0.00123
/// 1.23K 1.23k = 1230
/// 1.23M       = 1230000
/// 1.23G       = 1230000000
/// 1.23T       = 1230000000000
///
/// ---------------
///
/// e()  -- Euler's number (2.718281828459045)
/// pi() -- π (3.141592653589793)
///
/// log(base=10, val)
/// ---
/// Logarithm with optional 'base' as first argument.
/// If not provided, 'base' defaults to '10'.
/// Example: "log(100) + log(e(), 100)"
///
/// int(val)
/// ceil(val)
/// floor(val)
/// round(modulus=1, val)
/// ---
/// Round with optional 'modulus' as first argument.
/// Example: "round(1.23456) == 1  &&  round(0.001, 1.23456) == 1.235"
///
/// sqrt(val)
/// abs(val)
/// sign(val)
///
/// min(val, ...) -- Example: "min(1, -2, 3, -4) == -4"
/// max(val, ...) -- Example: "max(1, -2, 3, -4) == 3"
///
/// sin(radians)     asin(val)
/// cos(radians)     acos(val)
/// tan(radians)     atan(val)
/// sinh(val)        asinh(val)
/// cosh(val)        acosh(val)
/// tanh(val)        atanh(val)
/// ```
#[command]
#[aliases(calc, math, maths)]
#[min_args(1)]
async fn calculator(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut operation = args.message().to_string();
    operation = operation.replace("**", "^");
    operation = operation.replace("pi()", "pi");
    operation = operation.replace("pi", "pi()");
    operation = operation.replace("π", "pi()");
    operation = operation.replace("euler", "e()");

    let mut operation_without_markdown = operation.replace(r"\\", r"\\\\");
    // " my ide is bugged lol

    for i in &["*", "`", "_", "~", "|"] {
        operation_without_markdown = operation_without_markdown.replace(i, &format!(r"\{}", i));
    }

    let mut cb = |name:&str, args:Vec<f64>| -> Option<f64> {
        match name {
            "sqrt" => {
                let a = args.get(0);
                if let Some(x) = a {
                    let l = x.log10();
                    Some(10.0_f64.powf(l/2.0))
                } else {
                    None
                }
            },
            _ => None,
        }
    };

    let val = fasteval::ez_eval(&operation, &mut cb);

    match val {
        Err(why) => {
            let text = match &why {
                Error::SlabOverflow => "Too many Expressions/Values/Instructions were stored in the Slab.".to_string(),
                Error::EOF => "Reached an unexpected End Of Input during parsing.\nMake sure your operators are complete.".to_string(),
                Error::EofWhileParsing(x) => format!("Reached an unexpected End Of Input during parsing:\n{}", x),
                Error::Utf8ErrorWhileParsing(_) => "The operator could not be decoded with UTF-8".to_string(),
                Error::TooLong => "The expression is too long.".to_string(),
                Error::TooDeep => "The expression is too recursive.".to_string(),
                Error::UnparsedTokensRemaining(x) => format!("An expression was parsed, but there is still input data remaining.\nUnparsed data: {}", x),
                Error::InvalidValue => "A value was expected, but invalid input data was found.".to_string(),
                Error::ParseF64(x) => format!("Could not parse a 64 bit floating point number:\n{}", x),
                Error::Expected(x) => format!("The expected input data was not found:\n{}", x),
                Error::WrongArgs(x) => format!("A function was called with the wrong arguments:\n{}", x),
                Error::Undefined(x) => format!("The expression tried to use an undefined variable or function, or it didn't provide any required arguments.:\n{}", x),
                Error::Unreachable => "This error should never happen, if it did, contact nitsuga5124#2207 immediately!".to_string(),
                _ => format!("An unhandled error occurred:\n{:#?}", &why),
            };

            msg.channel_id.send_message(ctx, |m| m.embed(|e| {
                e.title("ERROR");
                e.description(text);
                e.field("Operation", &operation_without_markdown, true);
                e.footer(|f| f.text(format!("Submitted by: {}", msg.author.tag())))
            })).await?;
        },
        Ok(res) => {
            msg.channel_id.send_message(ctx, |m| m.embed(|e| {
                e.title("Result");
                e.description(res);
                e.field("Operation", &operation_without_markdown, true);
                e.footer(|f| f.text(format!("Submitted by: {}", msg.author.tag())))
            })).await?;
        }
    }
    Ok(())
}

/// Reminds you of a message after some time.
///
/// ```
/// s -> Second
/// m -> Minute
/// h -> Hour
/// D -> Day
/// W -> Week
/// M -> Month
/// Y -> Year
/// ```
///
/// Usage:
/// `remind_me 2h take the dog out for a walk.`
/// `remind_me "2h 30m" mess with the neighbours :P`
/// `remind_me "1Y 1M 1W 1D 1h 1m 1s" i bet you forgot about this!`
#[command]
#[aliases(remindme, reminder, remind, schedule)]
#[min_args(1)]
async fn remind_me(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    let unformatted_time = args.single_quoted::<String>()?;
    let text = args.rest();

    let message = if text.is_empty() {
        None
    } else {
        Some(text)
    };

    let seconds = string_to_seconds(unformatted_time);

    if seconds < 30 {
        msg.reply(ctx, "Duration is too short").await?;
        return Ok(());
    }

    sqlx::query!("INSERT INTO reminders (date, message_id, channel_id, guild_id, user_id, message) VALUES ($1, $2, $3, $4, $5, $6)",
        chrono::offset::Utc::now() + chrono::Duration::seconds(seconds as i64),
        msg.id.0 as i64,
        msg.channel_id.0 as i64,
        msg.guild_id.unwrap_or(GuildId(0)).0 as i64,
        msg.author.id.0 as i64,
        message,
    )
    .execute(pool)
    .await?;

    msg.react(ctx, '👍').await?;

    Ok(())
}
