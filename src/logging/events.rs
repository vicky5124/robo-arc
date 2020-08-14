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
                }
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
                Event::MessageDeleteBulk(data) => {
                    if data.guild_id.is_none() {
                        return;
                    }

                    senders::send_message_delete_bulk(&ctx, &data).await;
                }
                Event::GuildMemberAdd(data) => {
                    senders::send_guild_member_add(&ctx, &data).await;
                }
                Event::GuildMemberRemove(data) => {
                    senders::send_guild_member_remove(&ctx, &data).await;
                }
                Event::GuildMemberUpdate(data) => {
                    senders::send_guild_member_update(&ctx, &data).await;
                }
                Event::GuildRoleCreate(data) => {
                    senders::send_guild_role_create(&ctx, &data).await;
                }
                Event::GuildRoleDelete(data) => {
                    senders::send_guild_role_delete(&ctx, &data).await;
                }
                Event::GuildRoleUpdate(data) => {
                    senders::send_guild_role_update(&ctx, &data).await;
                }
                Event::ReactionAdd(data) => {
                    if data.reaction.guild_id.is_none() {
                        return;
                    }

                    senders::send_reaction_add(&ctx, &data).await;
                }
                Event::ReactionRemove(data) => {
                    if data.reaction.guild_id.is_none() {
                        return;
                    }

                    senders::send_reaction_remove(&ctx, &data).await;
                }
                Event::ReactionRemoveAll(data) => {
                    if data.guild_id.is_none() {
                        return;
                    }

                    senders::send_reaction_remove_all(&ctx, &data).await;
                }
                Event::ChannelCreate(data) => {
                    if data.channel.clone().private().is_some() {
                        return;
                    }

                    senders::send_channel_create(&ctx, &data).await;
                }
                Event::ChannelDelete(data) => {
                    if data.channel.clone().private().is_some() {
                        return;
                    }

                    senders::send_channel_delete(&ctx, &data).await;
                }
                Event::ChannelUpdate(data) => {
                    if data.channel.clone().private().is_some() {
                        return;
                    }

                    senders::send_channel_update(&ctx, &data).await;
                }
                Event::ChannelPinsUpdate(data) => {
                    if data.guild_id.is_none() {
                        return;
                    }

                    senders::send_channel_pins_update(&ctx, &data).await;
                }
                Event::GuildBanAdd(data) => {
                    senders::send_guild_ban_add(&ctx, &data).await;
                }
                Event::GuildBanRemove(data) => {
                    senders::send_guild_ban_remove(&ctx, &data).await;
                }
                Event::GuildEmojisUpdate(data) => {
                    senders::send_guild_emojis_update(&ctx, &data).await;
                }
                Event::GuildIntegrationsUpdate(data) => {
                    senders::send_guild_integrations_update(&ctx, &data).await;
                }

                _ => ()
            }
        });
    }
}
