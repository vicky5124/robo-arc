use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};

#[command]
async fn basic_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.say(ctx, "Hello World!").await?;
    Ok(())
}
