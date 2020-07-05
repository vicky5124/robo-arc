use crate::RedisPool;
use crate::ConnectionPool;

use std::collections::HashMap;

use tracing::{
    warn,
    error,
};

use serenity::{
    async_trait,
    model::{
        id::{
            ChannelId,
            MessageId,
        },
        event::Event,
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
        let data_read = ctx.data.read().await;
        let redis_pool = data_read.get::<RedisPool>().unwrap();
        let mut redis = redis_pool.get().await;

        match event {
            Event::MessageCreate(data) => {
                let message = data.message;

                if message.author.bot || message.guild_id.is_none() {
                    return;
                }

                let rdata = ctx.data.read().await;
                let pool = rdata.get::<ConnectionPool>().unwrap();

                let data = match sqlx::query!("SELECT enabled FROM anti_spam WHERE guild_id = $1", message.guild_id.unwrap().0 as i64)
                    .fetch_optional(pool)
                    .await {
                        Err(why) => {
                            error!("Error quering database for anti_spam: {}", why);
                            return;
                        },
                        Ok(x) => x
                };

                if let Some(row) = data {
                    if row.enabled {
                        if let Err(why) = redis.append(message.author.id.0.to_string(), format!("{}|{},", message.id.0, message.channel_id.0)).await {
                            error!("Error sending data to redis: {}", why);
                        }

                        if let Err(why) = redis.expire_seconds(message.author.id.0.to_string(), 5).await {
                            error!("Error setting expire date to redis: {}", why);
                        }

                        match redis.get(message.author.id.0.to_string()).await {
                            Err(why) => error!("Error getting message data from redis: {}", why),
                            Ok(x) => {
                                if let Some(messages) = x {
                                    let mut messages_channels = String::from_utf8(messages).unwrap();
                                    messages_channels.pop();
                                    let messages = messages_channels.split(',');

                                    if messages.clone().count() > 5 {
                                        let mut bad_messages: HashMap<u64, Vec<MessageId>> = HashMap::new();

                                        for msg_chan in messages {
                                            let mut split = msg_chan.split('|');
                                            let message_id = split.nth(0).unwrap().parse::<u64>().unwrap();
                                            let channel_id = split.nth(0).unwrap().parse::<u64>().unwrap();

                                            if let Some(x) = bad_messages.get_mut(&channel_id) {
                                                x.push(MessageId(message_id));
                                            } else {
                                                bad_messages.insert(channel_id, vec![MessageId(message_id)]);
                                            }
                                        }

                                        for (channel, message_ids) in bad_messages.iter() {
                                            let _ = ChannelId(*channel).delete_messages(&ctx, message_ids).await;
                                        }

                                        let _ = message.reply(&ctx, "No spamming.").await;
                                        let _ = redis.del(message.author.id.0.to_string()).await;
                                    }
                                } else {
                                    warn!("This should never happen! Redis didn't obtain the message data that just got sent.");
                                }
                            }
                        }
                    }
                }
            },

            _ => (),
        }
    }
}
