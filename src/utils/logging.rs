#![allow(non_upper_case_globals)]
use bitflags::bitflags;

use sqlx::PgPool;
use serenity::model::id::GuildId;
use tracing::error;

bitflags! {
    pub struct LoggingEvents: u64 {
        const ChannelCreate              = 0b_000000000000000000000000001;
        const ChannelDelete              = 0b_000000000000000000000000010;
        const ChannelPinsUpdate          = 0b_000000000000000000000000100;
        const ChannelUpdate              = 0b_000000000000000000000001000;
        const GuildBanAdd                = 0b_000000000000000000000010000;
        const GuildBanRemove             = 0b_000000000000000000000100000;
        const GuildEmojisUpdate          = 0b_000000000000000000001000000;
        const GuildIntegrationsUpdate    = 0b_000000000000000000010000000;
        const GuildMemberAdd             = 0b_000000000000000000100000000;
        const GuildMemberRemove          = 0b_000000000000000001000000000;
        const GuildMemberUpdate          = 0b_000000000000000010000000000;
        const GuildRoleCreate            = 0b_000000000000000100000000000;
        const GuildRoleDelete            = 0b_000000000000001000000000000;
        const GuildRoleUpdate            = 0b_000000000000010000000000000;
        const GuildUpdate                = 0b_000000000000100000000000000;
        //const MessageCreate            = 0b_000000000001000000000000000;
        const MessageDelete              = 0b_000000000010000000000000000;
        const MessageDeleteBulk          = 0b_000000000100000000000000000;
        const MessageUpdate              = 0b_000000001000000000000000000;
        //const PresenceUpdate           = 0b_000000010000000000000000000;
        const ReactionAdd                = 0b_000000100000000000000000000;
        const ReactionRemove             = 0b_000001000000000000000000000;
        const ReactionRemoveAll          = 0b_000010000000000000000000000;
        //const UserUpdate               = 0b_000100000000000000000000000;
        const VoiceStateUpdate           = 0b_001000000000000000000000000;
        const VoiceServerUpdate          = 0b_010000000000000000000000000;
        const WebhookUpdate              = 0b_100000000000000000000000000;
    }
}

pub struct LoggingChannels {
    pub guild_id: i64,
    pub bitwise: i64,
    pub webhook_url: String,
}



pub async fn guild_has_logging(pool: &PgPool, event: LoggingEvents, guild_id: impl Into<GuildId>) -> Option<LoggingChannels> {
    let query = match sqlx::query_as!(LoggingChannels, "SELECT * FROM logging_channels WHERE guild_id = $1", guild_id.into().0 as i64)
        .fetch_optional(pool)
        .await {
            Ok(x) => x,
            Err(why) => {
                error!("Error quering Database: {}", why);
                return None;
            }
        }?;

    let log_events = LoggingEvents::from_bits_truncate(query.bitwise as u64);
    if log_events.contains(event) {
        Some(query)
    } else {
        None
    }
}
