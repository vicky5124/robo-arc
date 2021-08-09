use crate::{
    global_data::{DatabasePool, ShardManagerContainer},
    utils::basic_functions::*,
    Tokens, Uptime,
};
use std::{
    fs::{read_to_string, File, OpenOptions},
    io::prelude::*,
    lazy::Lazy,
    process::id,
    time::Instant,
};

use tokei::{Config, Languages, LanguageType};
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use serde_json::json;
use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::Message,
        oauth2::OAuth2Scope,
        Permissions,
        //channel::ReactionType,
    },
    prelude::Context,
};
use tokio::process::Command;
use toml::Value;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
struct Code {
    language: String,
    source: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RustCode {
    backtrace: bool,
    channel: String,
    code: String,
    #[serde(rename = "crateType")]
    crate_type: String,
    edition: String,
    mode: String,
    tests: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RanCode {
    ran: bool,
    language: String,
    version: String,
    output: String,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRanCode {
    success: bool,
    stdout: String,
    stderr: String,
}

#[command] // Sets up a command
#[aliases("pong", "latency")] // Sets up aliases to that command.
#[description = "Sends the latency of the bot to the shards."] // Sets a description to be used for the help command. You can also use docstrings.

// All command functions must take a Context and Message type parameters.
// Optionally they may also take an Args type parameter for command arguments.
// They must also return CommandResult.
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Instant::now();
    reqwest::get("https://discordapp.com/api/v6/gateway").await?;
    let get_latency = now.elapsed().as_millis();

    let shard_latency = {
        let shard_manager = {
            let data_read = ctx.data.read().await;
            data_read.get::<ShardManagerContainer>().unwrap().clone()
        };

        let manager = shard_manager.lock().await;
        let runners = manager.runners.lock().await;

        let runner_raw = runners.get(&ShardId(ctx.shard_id));
        if let Some(runner) = runner_raw {
            match runner.latency {
                Some(ms) => format!("{}ms", ms.as_millis()),
                _ => "?ms".to_string(),
            }
        } else {
            "?ms".to_string()
        }
    };

    let map = json!({"content" : "Calculating latency..."});

    let now = Instant::now();
    let mut message = ctx.http.send_message(msg.channel_id.0, &map).await?;
    let post_latency = now.elapsed().as_millis();

    message
        .edit(ctx, |m| {
            m.content("");
            m.embed(|e| {
                e.title("Latency");
                e.description(format!(
                    "Gateway: {}\nREST GET: {}ms\nREST POST: {}ms",
                    shard_latency, get_latency, post_latency
                ))
            })
        })
        .await?;

    Ok(())
}

/// This command just sends an invite of the bot with the required permissions.
#[command]
async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    // Sets up the permissions
    let mut permissions = Permissions::empty();
    permissions.set(Permissions::KICK_MEMBERS, true);
    permissions.set(Permissions::BAN_MEMBERS, true);
    permissions.set(Permissions::MANAGE_CHANNELS, true);
    permissions.set(Permissions::ADD_REACTIONS, true);
    permissions.set(Permissions::VIEW_AUDIT_LOG, true);
    permissions.set(Permissions::READ_MESSAGES, true);
    permissions.set(Permissions::SEND_MESSAGES, true);
    permissions.set(Permissions::MANAGE_MESSAGES, true);
    permissions.set(Permissions::EMBED_LINKS, true);
    permissions.set(Permissions::ATTACH_FILES, true);
    permissions.set(Permissions::READ_MESSAGE_HISTORY, true);
    permissions.set(Permissions::USE_EXTERNAL_EMOJIS, true);
    permissions.set(Permissions::CONNECT, true);
    permissions.set(Permissions::SPEAK, true);
    permissions.set(Permissions::MANAGE_ROLES, true);
    permissions.set(Permissions::MANAGE_WEBHOOKS, true);
    permissions.set(Permissions::MENTION_EVERYONE, true);

    let scopes = vec![OAuth2Scope::Bot, OAuth2Scope::ApplicationsCommands];

    // Creates the invite link for the bot with the permissions specified earlier.
    // Error handling in rust i so nice.
    let url = match ctx
        .cache
        .current_user()
        .await
        .invite_url_with_oauth2_scopes(ctx, permissions, &scopes)
        .await
    {
        Ok(v) => v,
        Err(why) => {
            println!("Error creating invite url: {:?}", why);

            return Ok(()); // Prematurely finish the command function.
        }
    };

    msg.channel_id.send_message(ctx, |m| {
        m.embed( |e| {
            e.title("Invite Link");
            e.url(url);
            e.description("Keep in mind, this bot is still in pure developement, so not all of this mentioned features are implemented.\n\n__**Reason for each permission**__");
            e.fields(vec![
                ("Attach Files", "For some of the booru commands.\nFor an automatic text file to be sent when a message is too long.", true),
                ("Read Messages", "So the bot can read the messages to know when a command was invoked and such.", true),
                ("Manage Messages", "Be able to clear reactions of timed out paginations.\nClear moderation command.", true),
                ("Manage Channels", "Be able to mute members on the channel without having to create a role for it.", true),
                ("Manage Webhooks", "For all the commands that can be ran on a schedule, so it's more efficient.", true),
                ("Manage Roles", "Be able to give a stream notification role.\nMute moderation command.", true),
                ("Read Message History", "This is a required permission for every paginated command.", true),
                ("Use External Emojis", "For all the commands that use emojis for better emphasis.", true),
                ("View Audit Log", "To be able to have a more feature rich logging to a channel.", true),
                ("Add Reactions", "To be able to add reactions for all the paginated commands.", true),
                ("Mention Everyone", "To be able to mention the livestream notification role.", true),
                ("Send Messages", "So the bot can send the messages it needs to send.", true),
                ("Speak", "To be able to play music on that voice channel.", true),
                ("Embed Links", "For the tags to be able to embed images.", true),
                ("Connect", "To be able to connect to a voice channel.", true),
                ("Kick Members", "Kick/GhostBan moderation command.", true),
                ("Ban Members", "Ban moderation command.", true),
            ]);
            e
        });

        m
    }).await?;
    Ok(())
}

#[command]
#[help_available(false)] // makes it not show up on the help menu
#[owners_only] // to only allow the owner of the bot to use this command
               //#[min_args(3)] // Sets the minimum ammount of arguments the command requires to be ran. This is used to trigger the `NotEnoughArguments` error.
               // Testing command, please ignore.
async fn test(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    //if let Ok(channels) = _msg.guild_id.unwrap().channels(_ctx).await {
    //    let channels_stream = stream::iter(channels.iter());

    //    let log_channels_future = channels_stream.filter_map(|(&c, _)| async move {
    //        if let Some(name) = c.name(_ctx).await {
    //            if name == "log" {
    //                Some(c.clone())
    //            } else {
    //                None
    //            }
    //        } else {
    //            None
    //        }
    //    }).collect::<Vec<_>>().await;

    //    dbg!(&log_channels_future);
    //}

    //_msg.channel_id.say(_ctx, "test").await?;
    //crate::utils::osu::PpCalculation::test();
    //
    //msg.reply(ctx, futures::stream::iter(ctx.cache.guilds().await.iter()).map(|i| async { i.members(ctx, None, None).await.unwrap() }).filter(|i| async { !i.await.user.bot }).collect::<Vec<_>>().await.len()).await?;

    Ok(())
}

/// Sends the source code url to the bot.
#[command]
async fn source(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "<https://gitlab.com/vicky5124/robo-arc/>")
        .await?;
    Ok(())
}

/// Sends the current TO-DO list of the bot
#[command]
#[aliases(todo_list, issues, bugs, bug)]
async fn todo(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(ctx, "The TODO List and all the open Issues can be found here:\n<https://gitlab.com/vicky5124/robo-arc/-/boards>").await?;
    Ok(())
}

/// Sends the current prefixes set to the server.
#[command]
#[aliases(prefixes)]
async fn prefix(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let prefix;
    if let Some(id) = &msg.guild_id {
        // obtain the id of the guild as an i64, because the id is stored as a u64, which is
        // not compatible with the postgre datbase types.
        let gid = id.0 as i64;

        // Read the configured prefix of the guild from the database.
        let db_prefix = sqlx::query!("SELECT prefix FROM prefixes WHERE guild_id = $1", gid)
            .fetch_optional(&pool)
            .await?;

        // Just read the value that was stored on the database and return it.
        if let Some(x) = db_prefix {
            prefix = x.prefix.unwrap();
            // Else, the guild doesn't have a configured prefix, return the default prefix.
        } else {
            prefix = ".".to_string();
        }
    } else {
        prefix = ".".to_string();
    }

    msg.channel_id
        .say(ctx, format!("Current prefix:\n`{}`", &prefix))
        .await?;

    Ok(())
}

/// Sends information about the bot.
#[command]
#[aliases(info)]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let shard_latency = {
        let shard_manager = {
            let data_read = ctx.data.read().await;
            data_read.get::<ShardManagerContainer>().unwrap().clone()
        };

        let manager = shard_manager.lock().await;
        let runners = manager.runners.lock().await;

        let runner_raw = runners.get(&ShardId(ctx.shard_id));
        if let Some(runner) = runner_raw {
            match runner.latency {
                Some(ms) => format!("{}ms", ms.as_millis()),
                _ => "?ms".to_string(),
            }
        } else {
            "?ms".to_string()
        }
    };

    let uptime = {
        let instant = {
            let data_read = ctx.data.read().await;
            data_read.get::<Uptime>().unwrap().clone()
        };

        let duration = instant.elapsed();
        seconds_to_days(duration.as_secs())
    };

    let map = json!({"content" : "Calculating latency..."});

    let now = Instant::now();
    let mut message = ctx.http.send_message(msg.channel_id.0, &map).await?;
    let rest_latency = now.elapsed().as_millis();

    let pid = id().to_string();

    let full_stdout = Command::new("sh")
        .arg("-c")
        .arg(format!("./full_memory.sh {}", &pid).as_str())
        .output()
        .await
        .expect("failed to execute process");
    let reasonable_stdout = Command::new("sh")
        .arg("-c")
        .arg(format!("./reasonable_memory.sh {}", &pid).as_str())
        .output()
        .await
        .expect("failed to execute process");

    let mut full_mem = String::from_utf8(full_stdout.stdout).unwrap();
    let mut reasonable_mem = String::from_utf8(reasonable_stdout.stdout).unwrap();

    full_mem.pop();
    full_mem.pop();
    reasonable_mem.pop();
    reasonable_mem.pop();

    let version = {
        let mut file = File::open("Cargo.toml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data = contents.parse::<Value>().unwrap();
        let version = data["package"]["version"].as_str().unwrap();
        version.to_string()
    };

    let (hoster_team, hoster_tag, hoster_id) = {
        let app_info = ctx.http.get_current_application_info().await?;

        if let Some(t) = app_info.team {
            (t.id.to_string(), t.members[0].user.tag(), t.owner_user_id)
        } else {
            ("None".to_string(), app_info.owner.tag(), app_info.owner.id)
        }
    };

    let current_user = ctx.cache.current_user().await;

    let bot_name = &current_user.name;
    let bot_icon = &current_user.avatar_url();

    let num_guilds = ctx.cache.guilds().await.len();
    let num_shards = ctx.cache.shard_count().await;
    let num_channels = ctx.cache.guild_channel_count().await;
    let num_users = ctx.cache.user_count().await;

    let mut c_blank = 0;
    let mut c_comment = 0;
    let mut c_code = 0;
    let mut c_lines = 0;
    let mut command_count = 0;

    for entry in WalkDir::new("src") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let text = read_to_string(&path)?;
            command_count += text.match_indices("#[command]").count();
        }
    }

    let paths = &["src", "migrations", "."];
    let excluded = &["target", "osu_data", "eval", "opus", ".git", "*.png", "*.jpg", "*.lock", "*.example"];
    let config = Config {
        treat_doc_strings_as_comments: Some(true),
        ..Config::default()
    };
    
    let mut languages = Languages::new();
    languages.get_statistics(paths, excluded, &config);
    let count = [&languages[&LanguageType::Rust], &languages[&LanguageType::Python], &languages[&LanguageType::Sql]];

    for i in count {
        c_blank += i.blanks;
        c_comment += i.comments;
        c_code += i.code;
        c_lines += i.lines();
    }

    message.edit(ctx, |m| {
        m.content("");
        m.embed(|e| {
            e.title(format!("**{}** - v{}", bot_name, version));
            e.url("https://gitlab.com/vicky5124/robo-arc");
            e.description("General Purpose Discord Bot made in [Rust](https://www.rust-lang.org/) using [serenity.rs](https://github.com/serenity-rs/serenity)\n\nHaving any issues? join the [Support Server](https://discord.gg/kH7z85n)\nBe sure to `invite` me to your server if you like what i can do!");

            //e.field("Creator", "Tag: vicky5124#2207\nID: 182891574139682816", true);
            e.field("Statistics:", format!("Shards: {}\nGuilds: {}\nChannels: {}\nUsers: {}", num_shards, num_guilds, num_channels, num_users), true);
            e.field("Lines of code:", format!("Blank: {}\nComment: {}\nCode: {}\nTotal Lines: {}", c_blank, c_comment, c_code, c_lines), true);
            e.field("Currently owned by:", format!("Team: {}\nTag: {}\nID: {}", hoster_team, hoster_tag, hoster_id), true);
            e.field("Latency:", format!("Gateway:\n`{}`\nREST:\n`{}ms`", shard_latency, rest_latency), true);
            e.field("Memory usage:", format!("Complete:\n`{} KB`\nBase:\n`{} KB`",
                    &full_mem.parse::<u32>().expect("NaN").to_formatted_string(&Locale::en),
                    &reasonable_mem.parse::<u32>().expect("NaN").to_formatted_string(&Locale::en)
            ), true);
            e.field("Somewhat Static Stats:", format!("Command Count:\n`{}`\nUptime:\n`{}`", command_count, uptime), true);

            if let Some(x) = bot_icon {
                e.thumbnail(x);
            }
            e
        });
        m
    }).await?;

    Ok(())
}

/// Sends the bot changelog.
#[command]
async fn changelog(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            "<https://gitlab.com/vicky5124/robo-arc/-/blob/master/CHANGELOG.md>",
        )
        .await?;
    Ok(())
}

#[command]
#[aliases(tos, terms)]
async fn terms_of_service(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(ctx, "
        I know you likely don't care much about this, so i'll keep them short.

        By agreeing with this terms of service you agree that the application should be able to store all your messages and discord user data; This user data includes your account ID, Username, Discriminator and Avatar, along with a history of each; No personal information is ever stored.
        The application is completely open source, so you always are able to see what data is exactly being stored.

        All of this data is completely encrypted and will NEVER be used for any other purpose than logging inside discord itself.

        If you still don't want to have this data stored, contact vicky5124#2207, and all your data will be deleted and stopped from being logged.
        ").await?;

    Ok(())
}

#[command]
#[aliases(features, bugs, report, reports, suggest, suggestions)]
async fn issues(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            "
            You are free to submit issues, bug reports and new features to the issues page:
            <https://gitlab.com/vicky5124/robo-arc/-/issues>
            ",
        )
        .await?;
    Ok(())
}

/// Executes the provided code.
///
/// - A file can be attached rather than using a codeblock.
/// - There's a 10 second timeout.
/// - For rust, it is better to use the `rust` command!
///
/// usage:
/// eval \`\`\`py
/// print("Hello, world!")
/// \`\`\`
#[command]
#[aliases(exec, compile, run)]
async fn eval(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // curl -X POST https://emkc.org/api/v1/piston/execute -H 'content-type: application/json' --data '{"language": "py", "source": "import this; print(\"test\")"}'
    //"```([a-zA-Z].+?(?=\n))"
    //"([`]+)$|```([a-zA-Z].+?(?=\n))"
    //"(^```)(.*?(?=\n))([\s\S]*)(```$)"
    //"^```(?P<syntax>.*)\n(?P<code>(?:.+|\n)*)```$"

    let code = if let Some(attachment) = msg.attachments.get(0) {
        let filename_split = attachment.filename.split('.');

        if filename_split.clone().count() < 2 {
            msg.reply(
                ctx,
                "Please, provide a file with a valid file extension with the language used.",
            )
            .await?;
            return Ok(());
        }

        let language = match filename_split.last() {
            Some(x) => x.to_string(),
            None => {
                msg.reply(
                    ctx,
                    "Please, provide a file with a valid file extension with the language used.",
                )
                .await?;
                return Ok(());
            }
        };

        let raw_code = attachment.download().await?;
        let source = if raw_code.len() > 8000000 {
            msg.reply(ctx, "Please, don't upload a file over 8MB.")
                .await?;
            return Ok(());
        } else if raw_code.len() > 1 {
            String::from_utf8(raw_code)?
        } else {
            msg.reply(ctx, "Please, don't upload an empty file.")
                .await?;
            return Ok(());
        };

        Code { language, source }
    } else {
        let re = Lazy::new(|| Regex::new("^```(?P<lang>.*)\n(?P<src>(?:.+|\n)*)```$").unwrap());
        let captures = re.captures(args.message());

        let caps = if let Some(caps) = captures {
            caps
        } else {
            msg.reply(ctx, "No codeblock was provided, please put your code inside a codeblock:\n\n\\`\\`\\`lang\n<your code here>\n\\`\\`\\`").await?;
            return Ok(());
        };

        let language = caps.name("lang").unwrap().as_str().to_string();
        let source = caps.name("src").unwrap().as_str().trim().to_string();

        if language.is_empty() {
            msg.reply(ctx, "No codeblock with language was provided, please put the programming language inside the codeblock:\n\n\\`\\`\\`lang\n<your code here>\n\\`\\`\\`").await?;
            return Ok(());
        }

        if source.is_empty() {
            msg.reply(ctx, "No code was provided, please put some code inside the codeblock:\n\n\\`\\`\\`lang\n<your code here>\n\\`\\`\\`").await?;
            return Ok(());
        }

        Code { language, source }
    };

    let client = reqwest::Client::new();

    let response = match client
        .post("https://emkc.org/api/v1/piston/execute")
        .json(&code)
        .send()
        .await?
        .json::<RanCode>()
        .await
    {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "The programming language specified is not supported.")
                .await?;
            return Ok(());
        }
    };

    // println!("{:#?}", &response);

    let mut did_run = true;

    if !response.ran {
        did_run = false;
        warn!("Code didn't run:\n{:#?}", response);
    }

    let description = if response.output.len() > 1950 {
        match create_paste(&response.output).await {
            Err(why) => {
                error!("Error creating paste: {}", why);
                let parsed = response.output.replace("```", "\\`\\`\\`");
                format!("```\n{}```", &parsed[..1950])
            }
            Ok(x) => {
                if x.is_empty() {
                    let parsed = response.output.replace("```", "\\`\\`\\`");
                    format!(
                        "Output was too long to upload.\n```\n{}```",
                        &parsed[..1950]
                    )
                } else {
                    format!("Output was too long, so it was uploaded here: {}", x)
                }
            }
        }
    } else {
        format!("```\n{}```", &response.output.replace("```", "\\`\\`\\`"))
    };

    msg.channel_id
        .send_message(ctx, |m| {
            if response.language == "rust" {
                m.content("For rust, it's better to use the `rust` command instead.");
            }
            m.reference_message(msg);
            m.allowed_mentions(|f| f.replied_user(false));
            m.embed(|e| {
                e.title(format!(
                    "Evaluated \"{}\" v{}",
                    &response.language, &response.version
                ));
                e.description(description);
                e.footer(|f| {
                    if did_run {
                        f.text("Code ran successfully")
                    } else {
                        f.text("Code didn't run successfully")
                    }
                })
            })
        })
        .await?;

    Ok(())
}

/// Executes the provided rust code using https://play.rust-lang.org/
///
/// - A file can be attached rather than using a codeblock.
/// - There's a 10 second timeout.
/// - Default build is debug unless `// release` is used somewhere in the code.
/// - If `#![feature(...)]` is used, the build will be nightly rather than stable.
/// - If `fn main` is used, the code will be ran.
///
/// usage:
/// rust \`\`\`rs
/// fn main() {
///     println!("Hello, World!");
/// }
/// \`\`\`
#[command]
async fn rust(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let source = if let Some(attachment) = msg.attachments.get(0) {
        let raw_code = attachment.download().await?;

        let source = if raw_code.len() > 8000000 {
            msg.reply(ctx, "Please, don't upload a file over 8MB.")
                .await?;
            return Ok(());
        } else if raw_code.len() > 1 {
            String::from_utf8(raw_code)?
        } else {
            msg.reply(ctx, "Please, don't upload an empty file.")
                .await?;
            return Ok(());
        };

        source
    } else {
        let re = Lazy::new(|| Regex::new("^```(?P<lang>.*)\n(?P<src>(?:.+|\n)*)```$").unwrap());
        let captures = re.captures(args.message());

        let caps = if let Some(caps) = captures {
            caps
        } else {
            msg.reply(ctx, "No codeblock was provided, please put your code inside a codeblock:\n\n\\`\\`\\`rust\n<your code here>\n\\`\\`\\`").await?;
            return Ok(());
        };

        let language = caps.name("lang").unwrap().as_str();
        let source = caps.name("src").unwrap().as_str().trim().to_string();

        if language != "rs" && language != "rust" && !language.is_empty() {
            msg.reply(ctx, "The codeblock provided is for a different language, please put `rs` or `rust` in the language field:\n\n\\`\\`\\`rust\n<your code here>\n\\`\\`\\`").await?;
            return Ok(());
        }

        if source.is_empty() {
            msg.reply(ctx, "No code was provided, please put some code inside the codeblock:\n\n\\`\\`\\`rust\n<your code here>\n\\`\\`\\`").await?;
            return Ok(());
        }

        source
    };

    let code = RustCode {
        channel: {
            if source.contains("#![feature(") {
                "nightly".to_string()
            } else {
                "stable".to_string()
            }
        },
        crate_type: {
            if source.contains("fn main") {
                "bin".to_string()
            } else {
                "lib".to_string()
            }
        },
        mode: {
            if source.contains("// release") {
                "release".to_string()
            } else {
                "debug".to_string()
            }
        },
        code: source,
        edition: "2018".to_string(),
        backtrace: false,
        tests: false,
    };

    let client = reqwest::Client::new();

    let response = match client
        .post("https://play.rust-lang.org/execute")
        .json(&code)
        .send()
        .await?
        .json::<RustRanCode>()
        .await
    {
        Ok(x) => x,
        Err(_) => {
            msg.reply(
                ctx,
                "Timeout error: The code like took more than 12 seconds to compile.",
            )
            .await?;
            return Ok(());
        }
    };

    // println!("{:#?}", &response);

    let stderr = if response.stderr.len() > 950 {
        match create_paste(&response.stderr).await {
            Err(why) => {
                error!("Error creating paste: {}", why);
                let parsed = response.stderr.replace("```", "\\`\\`\\`");
                format!("```\n{}```", &parsed[..950])
            }
            Ok(x) => {
                if x.is_empty() {
                    let parsed = response.stderr.replace("```", "\\`\\`\\`");
                    format!("Output was too long to upload.\n```\n{}```", &parsed[..950])
                } else {
                    format!("Output was too long, so it was uploaded here: {}", x)
                }
            }
        }
    } else {
        format!("```\n{}```", &response.stderr.replace("```", "\\`\\`\\`"))
    };

    let stdout = if response.stdout.len() > 950 {
        match create_paste(&response.stdout).await {
            Err(why) => {
                error!("Error creating paste: {}", why);
                let parsed = response.stdout.replace("```", "\\`\\`\\`");
                format!("```\n{}```", &parsed[..950])
            }
            Ok(x) => {
                if x.is_empty() {
                    let parsed = response.stdout.replace("```", "\\`\\`\\`");
                    format!("Output was too long to upload.\n```\n{}```", &parsed[..950])
                } else {
                    format!("Output was too long, so it was uploaded here: {}", x)
                }
            }
        }
    } else {
        format!("```\n{}```", &response.stdout.replace("```", "\\`\\`\\`"))
    };

    msg.channel_id
        .send_message(ctx, |m| {
            m.reference_message(msg);
            m.allowed_mentions(|f| f.replied_user(false));
            m.embed(|e| {
                e.title("Evaluated Rust Playground!");
                e.field("stderr", stderr, false);
                e.field("stdout", stdout, false);
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
#[owners_only]
async fn admin_eval(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let token = {
        let read_data = ctx.data.read().await;
        let config = read_data.get::<Tokens>().unwrap();
        config.discord.to_string()
    };

    let message = serde_json::to_string(&msg)?;

    let raw_eval_code = args.message();

    let eval_code = if raw_eval_code.starts_with("```rs") && raw_eval_code.ends_with("```") {
        raw_eval_code[5..raw_eval_code.len() - 3].to_string() + ";"
    } else if raw_eval_code.starts_with("```") && raw_eval_code.ends_with("```") {
        raw_eval_code[3..raw_eval_code.len() - 3].to_string() + ";"
    } else {
        raw_eval_code.to_string() + ";"
    };

    let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("eval/src/main.rs")?;

    let code = format!(
        r#####"
#![allow(unused_variables)]
#![allow(redundant_semicolons)]
        use std::error::Error;

        use twilight_http::Client;
        use twilight_model::channel::message::Message;

#[tokio::main]
        async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {{
            let client = Client::new("{}");
            let ctx = &client;

            let msg: Message = serde_json::from_str(r####"{}"####)?;

            {}

            Ok(())
        }}
        "#####,
        token, message, eval_code
    );

    file.set_len(0)?;
    write!(file, "{}", code)?;

    let output = Command::new("cargo")
        .arg("make")
        .arg("-l")
        .arg("error")
        .arg("eval")
        .output()
        .await?;

    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    if !stdout.is_empty() {
        println!("{}", stdout);
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Rust Eval (stdout)");
                    e.description(format!("```rs\n{}\n```", stdout))
                })
            })
            .await?;
    }
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Rust Eval (stderr)");
                    e.description(format!("```rs\n{}\n```", stderr))
                })
            })
            .await?;
    }

    Ok(())
}
