use crate::global_data::DatabasePool;

use std::{
    collections::HashMap,
    sync::Arc,
};

use darkredis::Connection;

use tracing::{
    //info,
    warn,
    error,
    instrument,
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

#[instrument(skip(ctx))]
pub async fn log_message(ctx: Arc<Context>, data: &MessageCreateEvent) {
    let message = &data.message;

    if message.guild_id.is_none() {
        return;
    }

    let message_id = message.id.0 as i64;
    let channel_id = message.channel_id.0 as i64;
    let guild_id = message.guild_id.unwrap().0 as i64;
    let author_id = message.author.id.0 as i64;

    let webhook_id = if let Some(x) = message.webhook_id {
        Some(x.0 as i64)
    } else {
        None
    };
    
    let attachments = message.attachments
        .iter()
        .map(|i| i.proxy_url.to_string())
        .collect::<Vec<String>>();

    let embeds = message.embeds
        .iter()
        .map(|i| serde_json::to_string(i).unwrap())
        .collect::<Vec<String>>();

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    if let Err(why) = sqlx::query!("
        INSERT INTO log_messages
        (id, channel_id, guild_id, author_id, content, attachments, embeds, pinned, creation_timestamp, tts, webhook_id)
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ",
            message_id, channel_id, guild_id, author_id,
            &message.content, &attachments, &embeds,
            message.pinned, message.timestamp, message.tts,
            webhook_id, //message.kind,
        )
        .execute(&pool)
        .await
    {
        error!("Error inserting message to database: {}", why);
    };
}

pub async fn anti_spam_message(ctx: Arc<Context>, data: &MessageCreateEvent, redis: &mut Connection) {
    let message = &data.message;

    if message.author.bot || message.guild_id.is_none() {
        return;
    }

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let data = match sqlx::query!("SELECT enabled FROM anti_spam WHERE guild_id = $1", message.guild_id.unwrap().0 as i64)
        .fetch_optional(&pool)
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

pub async fn log_edit(ctx: Arc<Context>, data: &MessageUpdateEvent) {
    if data.guild_id.is_none() {
        return;
    }

    let message_id = data.id.0 as i64;

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let mut old_message = match sqlx::query!("SELECT content, content_history, attachments, attachments_history, embeds, embeds_history, pinned, was_pinned FROM log_messages WHERE id = $1", message_id)
        .fetch_optional(&pool)
        .await {
            Err(why) => {
                error!("Error getting existing message message of an edit from database: {}", why);
                return;
            },
            Ok(x) => if let Some(y) = x { y } else { return },
    };
    
    let content = if let Some(x) = &data.content {
        if let Some(x) = old_message.content.clone() {
            if let Some(ref mut old_contents) = old_message.content_history {
                old_contents.push(x);
                old_contents.dedup();
            } else {
                old_message.content_history = Some(vec![x])
            }
        }

        Some(x.to_string())
    } else {
        old_message.content
    };

    let attachments = if let Some(x) = &data.attachments {
        if let Some(x) = old_message.attachments.clone() {
            if let Some(ref mut old_attachments) = old_message.attachments_history {
                // Serialize the data, because sqlx does not support 2D arrays.
                old_attachments.push(x.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("|"));
                old_attachments.dedup();
            } else {
                old_message.attachments_history = Some(vec![x.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("|")])
            }
        }

        Some(x.iter().map(|i| i.proxy_url.to_string()).collect::<Vec<String>>())
    } else {
        old_message.attachments
    };

    let embeds = if let Some(x) = &data.embeds {
        if let Some(x) = old_message.embeds.clone() {
            if let Some(ref mut old_embeds) = old_message.embeds_history {
                old_embeds.push(x.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("|"));
                old_embeds.dedup();
            } else {
                old_message.embeds_history = Some(vec![x.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("|")])
            }
        }

        Some(x.iter().map(|i| serde_json::to_string(i).unwrap()).collect::<Vec<String>>())
    } else {
        old_message.embeds
    };
    
    let (pinned, was_pinned) = if let Some(x) = data.pinned {
        if x {
            (true, Some(true))
        } else {
            (false, old_message.was_pinned)
        }
    } else {
        (old_message.pinned, old_message.was_pinned)
    };

    let timestamp = data.edited_timestamp;
    
    if let Err(why) = sqlx::query!("UPDATE log_messages SET content=$2, content_history=$3, attachments=$4, attachments_history=$5, embeds=$6, embeds_history=$7, pinned=$8, was_pinned=$9, edited_timestamp=$10 WHERE id = $1",
            message_id,
            content.as_deref(), old_message.content_history.as_deref(),
            attachments.as_deref(), old_message.attachments_history.as_deref(),
            embeds.as_deref(), old_message.embeds_history.as_deref(),
            pinned, was_pinned, timestamp,
        )
        .execute(&pool)
        .await
    {
        error!("Error updating message from edit to database: {}", why);
    };
}
