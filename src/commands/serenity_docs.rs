//use std::path::Path;
//use std::fs::File;
//use std::io::prelude::*;
//use walkdir::WalkDir;

use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};

/// Sends you a link to the serenity example on the specific topic.
/// It will default to a link to all the examples if the search was not found.
///
/// Usage:
/// `example 5`
/// `example collectors`
/// `example record voice`
#[command]
#[aliases(examples)]
async fn example(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.message().to_lowercase().as_str() {
        "1" | "01" | "basic" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e01_basic_ping_bot>").await?,

        "2" | "02" | "sharding" | "shards" | "shard" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e02_transparent_guild_sharding>").await?,

        "3" | "03" | "utils" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e03_struct_utilities>").await?,

        "4" | "04" | "builder" | "message builder" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e04_message_builder>").await?,

        "5" | "05" | "commands" | "framework" | "command" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e05_command_framework>").await?,

        "6" | "06" | "voice" | "music" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e06_voice>").await?,

        "7" | "07" | "basic bot" | "bot structure" | "structure" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e07_sample_bot_structure>").await?,

        "8" | "08" | "logging" | "logs" | "log" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e08_env_logging>").await?,

        "9" | "09" | "shard manager" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e09_shard_manager>").await?,

        "10" | "record" | "record voice" | "recieve voice" | "recieve" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e10_voice_receive>").await?,

        "11" | "embeds" | "file" | "files" | "send file" | "send files" | "embed" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e11_create_message_builder>").await?,

        "12" | "collectors" | "await_next for" | "reactions" | "reply" | "wait for" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e12_collectors>").await?,

        "13" | "intets" | "intent" =>
            msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples/e13_gateway_intents>").await?,

        _ => msg.channel_id.say(ctx, "<https://github.com/serenity-rs/serenity/tree/current/examples>").await?,
    };
    Ok(())
}

#[command]
#[aliases(rtfd, rtfw, rtm, rtd, rtw)]
async fn rtfm(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    //let query = args.message();
    
    //let path = std::path::Path::new(".");
    //let cache = racer::FileCache::default();
    //let session = racer::Session::new(&cache, Some(path));

    //for entry in WalkDir::new("/mnt/storage/Projects/Rust/serenity-await/src") {
    //    let entry = entry.unwrap();
    //    let path = entry.path();
    //    if path.is_file() {
    //        let file_path = path.to_str().unwrap();
    //        if file_path.ends_with(".rs") {
    //            let mut f = File::open(path.to_str().unwrap())?;
    //            let mut src = String::new();
    //            //dbg!(&f);
    //            f.read_to_string(&mut src)?;
    //            session.cache_file_contents(path.to_str().unwrap(), src);
    //        }
    //    }
    //}

    //
    //for m in racer::complete_fully_qualified_name("src::http::client::Htt", &path, &session) {
    //    dbg!(&m);
    //};
    //for m in racer::complete_fully_qualified_name("std::tim", &path, &session) {
    //    dbg!(&m);
    //};

    //println!("done");

    msg.channel_id.say(ctx, "Serenity Tokio: <https://docs.rs/serenity/0.9.0-rc.0/serenity/>
Serenity ThreadPool: <https://docs.rs/serenity/0.8.7/serenity/>").await?;

    Ok(())
}

