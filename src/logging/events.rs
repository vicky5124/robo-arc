use crate::RedisPool;
use crate::ConnectionPool;
use crate::logging::*;
use crate::utils::logging::LoggingEvents;

use std::sync::Arc;

//use tracing::{
//    warn,
//    error,
//};

use serenity::{
    async_trait,
    model::{
        event::Event,
        id::ChannelId,
    },
    prelude::{
        RawEventHandler,
        Context,
    },
};

pub struct RawHandler; // Defines the raw handler to be used for logging.

#[async_trait]
impl RawEventHandler for RawHandler {
    async fn raw_event(&self, ctx: Context, event: Event) {
        let ctx = Arc::new(ctx);

        tokio::spawn(async move {
            match event {
                Event::MessageCreate(data) => {
                    let message = &data.message;
                    if message.guild_id.is_none() {
                        return;
                    }

                    let data_read = ctx.data.read().await;
                    let redis_pool = data_read.get::<RedisPool>().unwrap();
                    let mut redis = redis_pool.get().await;

                    messages::anti_spam_message(Arc::clone(&ctx), &data, &mut redis).await;

                    drop(redis_pool);

                    messages::log_message(Arc::clone(&ctx), &data).await;
                },
                Event::MessageUpdate(data) => {
                    if data.guild_id.is_none() {
                        return;
                    }

                    messages::log_edit(Arc::clone(&ctx), &data).await;

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
                                                    e.field("New Content", &data.content.unwrap_or("- Empty Message".to_string()), false);
                                                    if let Some(author) = data.author {
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

                _ => ()
            }
        });
    }
}
