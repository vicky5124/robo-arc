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
fn basic_command(ctx: &mut Contextm, msg: Messagem, args: Args) -> CommandResult {
    msg.channel_id.say(&ctx, args.0);
    Ok(())
}
