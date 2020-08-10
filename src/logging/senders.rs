use crate::ConnectionPool;
use crate::utils::logging::LoggingEvents;

//use tracing::{
//    warn,
//    error,
//};

use serenity::{
    model::{
        event::*,
        id::{
            ChannelId,
            UserId,
        },
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
                                    e.description(format!("[Jump](https://discord.com/channels/{}/{}/{})", data.guild_id.unwrap().0, data.channel_id.0, data.id.0));
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
pub async fn send_message_delete(ctx: &Context, data: &MessageDeleteEvent) {
    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();
    let result = sqlx::query!("SELECT content, author_id, attachments, pinned, edited_timestamp, tts, webhook_id FROM log_messages WHERE id = $1", data.message_id.0 as i64)
        .fetch_optional(pool)
        .await;

    if let Ok(message) = result {
        let query = sqlx::query!("SELECT * FROM logging_channels WHERE guild_id = $1", data.guild_id.unwrap().0 as i64)
            .fetch_optional(pool)
            .await;

        if let Ok(query) = query {
            if let Some(query) = query {
                if let Some(msg) = message {
                    let author = if let Ok(x) = UserId(msg.author_id as u64).to_user(ctx).await { x } else { return };

                    let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                        e.title("Message Deleted");
                        e.description(format!("Message from <#{}> with the id `{}`", data.channel_id.0, data.message_id.0));
                        e.author(|a| {
                            a.icon_url(author.face());
                            a.name(author.tag())
                        });

                        if let Some(content) = &msg.content {
                            e.field("Content", content, false);
                        }

                        if let Some(id) = &msg.webhook_id {
                            e.field("Webhook ID", id, false);
                        }

                        if let Some(attachments) = &msg.attachments {
                            for attachment in attachments {
                                e.field("Attachment", attachment, false);
                            }
                        }

                        if let Some(x) = &msg.edited_timestamp {
                            e.timestamp(x);
                        }

                        e.footer(|f| {
                            f.text({
                                if msg.pinned && msg.tts.unwrap() {
                                    "Message was pinned and it was sent with TTS".to_string()
                                } else if msg.pinned && !msg.tts.unwrap() {
                                    "Message was pinned".to_string()
                                } else if !msg.pinned && msg.tts.unwrap() {
                                    "Message was sent with TTS".to_string()
                                } else {
                                    "Normal Message".to_string()
                                }
                            })
                        });

                        e
                    })).await;
                } else {
                    let _ = ChannelId(query.channel_id as u64).send_message(ctx, |m| m.embed(|e| {
                        e.title("Message Deleted");
                        e.description(format!("No information about the message was available.\nMessage from <#{}> with the id `{}`", data.channel_id.0, data.message_id.0))
                    })).await;
                }
            }
        }
    }
}
