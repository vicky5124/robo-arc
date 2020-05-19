//use std::path::Path;
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
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/01_basic_ping_bot>").await?,

        "2" | "02" | "sharding" | "shards" | "shard" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/02_transparent_guild_sharding>").await?,

        "3" | "03" | "utils" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/03_struct_utilities>").await?,

        "4" | "04" | "builder" | "message builder" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/04_message_builder>").await?,

        "5" | "05" | "commands" | "framework" | "command" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/05_command_framework>").await?,

        "6" | "06" | "voice" | "music" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/06_voice>").await?,

        "7" | "07" | "basic bot" | "bot structure" | "structure" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/07_sample_bot_structure>").await?,

        "8" | "08" | "logging" | "logs" | "log" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/08_env_logging>").await?,

        "9" | "09" | "shard manager" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/09_shard_manager>").await?,

        "10" | "record" | "record voice" | "recieve voice" | "recieve" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/10_voice_receive>").await?,

        "11" | "embeds" | "file" | "files" | "send file" | "send files" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/11_create_message_builder>").await?,

        "12" | "collectors" | "await for" | "reactions" | "reply" | "wait for" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/12_collectors>").await?,

        "13" | "intets" | "intent" =>
            msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples/13_gateway_intents>").await?,

        _ => msg.channel_id.say(ctx, "<https://github.com/Lakelezz/serenity/tree/await/examples>").await?,
    };
    Ok(())
}

#[command]
#[aliases(rtfd, rtfw, rtm, rtd, rtw)]
async fn rtfm(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    //let query = args.message();
    msg.channel_id.say(ctx, "<https://5124.mywire.org/tmp/serenity-await/serenity/>").await?;

    Ok(())
}

