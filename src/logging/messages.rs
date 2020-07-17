use crate::ConnectionPool;

use std::{
    collections::HashMap,
    sync::Arc,
};

use darkredis::Connection;
use futures::lock::MutexGuard;


use tracing::{
    //info,
    warn,
    error,
};

use serenity::{
    model::{
        id::{
            ChannelId,
            MessageId,
        },
        event::*,
    },
    prelude::Context,
};

pub async fn log_message(ctx: Arc<Context>, data: &MessageCreateEvent) {
    let message = &data.message;

    if message.guild_id.is_none() {
        return;
    }

    let message_id = message.id.0 as i64;
    let channel_id = message.channel_id.0 as i64;
    let guild_id = message.guild_id.unwrap().0 as i64;
    let author_id = message.author.id.0 as i64;
    
    let content = &message.content;
    let attachments = message.attachments.iter().map(|i| i.proxy_url.to_string()).collect::<Vec<String>>();
    let embeds = message.embeds.iter().map(|i| serde_json::to_string(i).unwrap()).collect::<Vec<String>>();
    
    let pinned = message.pinned;
    //let kind = format!("{:?}", message.kind);
    //let kind = message.kind;
    
    let timestamp = message.timestamp;
    
    let tts = message.tts;
    let webhook_id = if let Some(x) = message.webhook_id { Some(x.0 as i64) } else { None };

    let rdata = ctx.data.read().await;
    let pool = rdata.get::<ConnectionPool>().unwrap();

    if let Err(why) = sqlx::query_unchecked!("INSERT INTO log_messages (id, channel_id, guild_id, author_id, content, attachments, embeds, pinned, creation_timestamp, tts, webhook_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            message_id, channel_id, guild_id, author_id,
            content, &attachments, &embeds,
            pinned, timestamp, tts, webhook_id, //kind,
        )
        .execute(pool)
        .await
    {
        error!("Error inserting message to database: {}", why);
    };
}

pub async fn anti_spam_message(ctx: Arc<Context>, data: &MessageCreateEvent, redis: &mut MutexGuard<'_, Connection>) {
    let message = &data.message;

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

                            let _ = message.reply(&*ctx, "No spamming.").await;
                            let _ = redis.del(message.author.id.0.to_string()).await;
                        }
                    } else {
                        warn!("This should never happen! Redis didn't obtain the message data that just got sent.");
                    }
                }
            }
        }
    }
}
