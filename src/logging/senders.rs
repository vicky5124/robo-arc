use crate::ConnectionPool;
use crate::utils::logging::LoggingEvents;

//use tracing::{
//    warn,
//    error,
//};

use serenity::{
    model::{
        event::*,
        id::ChannelId,
    },
    prelude::Context,
};

pub async fn send_message_update(ctx: &Context, data: &MessageUpdateEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();
    let old_message = sqlx::query!("SELECT content_history FROM log_messages WHERE id = $1", data.id.0 as i64)
        .fetch_optional(pool)
        .await;
    if let Ok(old_message) = old_message {
        if let Some(old_message) = old_message {
            if let Some(old_message) = old_message.content_history {
                let old_message_content = old_message.get(old_message.len().checked_sub(1).unwrap_or(0));
                if old_message_content.clone().unwrap_or(&String::new()) != &data.content.clone().unwrap_or_default() {
                    let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.unwrap().0 as i64)
                        .fetch_optional(pool)
                        .await;

                    if let Ok(query) = query {
                        if let Some(query) = query {
                            let log_events = LoggingEvents::from_bits_truncate(query.bitwise as u64);
                            if log_events.contains(LoggingEvents::MessageUpdate) {
                                let _ = ChannelId(query.channel_id as u64).send_message(&ctx, |m| m.embed(|e| {
                                    e.title("Message Updated");
                                    e.description(format!("[Jump](https://discord.com/{}/{}/{})", data.guild_id.unwrap().0, data.channel_id.0, data.id.0));
                                    e.field("Original Content", old_message_content.unwrap_or(&String::new()).to_owned() + "\u{200b}", false);
                                    e.field("New Content", &data.content.as_ref().unwrap_or(&"- Empty Message".to_string()), false);
                                    if let Some(author) = &data.author {
                                        e.author(|a| {
                                            a.icon_url(author.face());
                                            a.name(author.tag())
                                        });
                                        //e.footer(|f| {
                                        //    f.icon_url(author.face());
                                        //    f.text(author.tag())
                                        //});
                                    }

                                    e
                                })).await;
                            }
                        }
                    }
                }
            }
        }
    }
}
