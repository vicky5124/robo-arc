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
fn basic_command(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.say(&ctx, "Hello World!");
    Ok(())
}
