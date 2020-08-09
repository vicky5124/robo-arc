use crate::RedisPool;
use crate::logging::*;

use std::sync::Arc;

//use tracing::{
//    warn,
//    error,
//};

use serenity::{
    async_trait,
    model::event::Event,
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

                    senders::send_message_update(&ctx, &data).await;
                }
                Event::MessageDelete(data) => {
                    if data.guild_id.is_none() {
                        return;
                    }

                    senders::send_message_delete(&ctx, &data).await;
                }

                _ => ()
            }
        });
    }
}
