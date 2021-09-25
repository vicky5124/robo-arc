use crate::global_data::{Lavalink, SongbirdCalls};
use crate::notifications::notification_loop;
use crate::AnnoyedChannels;
use crate::DatabasePool;
use crate::Tokens;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use lavalink_rs::gateway::LavalinkEventHandler;
use tokio::{time::{sleep, Duration}, sync::Mutex};
use warp::{reply::json, reply::Json, Filter};

use serenity::{
    async_trait,
    model::{
        channel::{GuildChannel, Message, Reaction, ReactionType},
        event::VoiceServerUpdateEvent,
        gateway::{Activity, Ready},
        guild::Member,
        id::{ChannelId, GuildId},
        user::OnlineStatus,
    },
    prelude::{Context, EventHandler},
};

#[derive(Serialize)]
pub struct Allow {
    allowed: bool,
}

pub struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {}

// Defines the handler to be used for events.
#[derive(Debug)]
pub struct Handler {
    pub run_loops: Mutex<bool>,
}

pub async fn is_on_guild(guild_id: u64, ctx: Arc<Context>) -> Result<Json, warp::Rejection> {
    let cache = &ctx.cache;

    let data = Allow {
        allowed: cache.guilds().iter().map(|i| i.0).any(|x| x == guild_id),
    };

    Ok(json(&data))
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache is READY");

        if *self.run_loops.lock().await {
            *self.run_loops.lock().await = false;

            let ctx = Arc::new(ctx);

            let web_server_info = {
                let read_data = ctx.data.read().await;
                let config = read_data.get::<Tokens>().unwrap();
                config.web_server.clone()
            };

            let ctx_clone = Arc::clone(&ctx);
            let ctx_clone2 = Arc::clone(&ctx);

            let notification_loop = tokio::spawn(async move { notification_loop(ctx_clone).await });

            tokio::spawn(async move {
                let routes = warp::path::param()
                    .and(warp::any().map(move || ctx_clone2.clone()))
                    .and_then(is_on_guild);

                let ip = web_server_info.server_ip;
                let port = web_server_info.server_port;

                warp::serve(routes)
                    .run(SocketAddr::from_str(format!("{}:{}", ip, port).as_str()).unwrap())
                    .await;
            });

            let _ = notification_loop.await;
            *self.run_loops.lock().await = false;
        }
    }

    // on_ready event on d.py
    // This function triggers when the client is ready.
    async fn ready(&self, ctx: Context, ready: Ready) {
        let info = {
            let read_data = ctx.data.read().await;
            let config = read_data.get::<Tokens>().unwrap();
            config.presence.clone()
        };

        if info.play_or_listen == "playing" {
            ctx.set_presence(Some(Activity::playing(&info.status)), OnlineStatus::Online)
                .await;
        } else if info.play_or_listen == "listening" {
            ctx.set_presence(
                Some(Activity::listening(&info.status)),
                OnlineStatus::Online,
            )
            .await;
        } else if info.play_or_listen == "competing" {
            ctx.set_presence(
                Some(Activity::competing(&info.status)),
                OnlineStatus::Online,
            )
            .await;
        }

        info!("Bot is READY");
        println!("{} is ready to rock!", ready.user.name);
    }

    // on_message event on d.py
    // This function triggers every time a message is sent.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let annoyed_channels = {
            // Read the global data lock
            let data_read = ctx.data.read().await;
            // Get the list of channels where the bot is allowed to be annoying
            data_read.get::<AnnoyedChannels>().unwrap().clone()
        };

        // if the channel the message was sent on is on the list
        if (annoyed_channels.read().await).contains(&msg.channel_id.0) {
            // NO U
            if msg.content == "no u" {
                let _ = msg.reply(&ctx, "no u").await; // reply pings the user
                                                       // AYY LMAO
            } else if msg.content == "ayy" {
                let _ = msg.channel_id.say(&ctx, "lmao").await; // say just send the message
            }
        }

        if msg.content.contains("discordapp.com/channels/")
            || msg.content.contains("discord.com/channels/")
        {
            let mut splits = msg.content.split('/');
            if splits.clone().count() == 7 {
                let channel_id = splits.nth(5).unwrap_or("0").parse::<u64>().expect("NaN");
                if let Ok(chan) = ChannelId(channel_id).to_channel(&ctx).await {
                    if chan.is_nsfw() {
                        let _ = msg.react(&ctx, '🇳').await;
                        let _ = msg.react(&ctx, '🇸').await;
                        let _ = msg.react(&ctx, '🇫').await;
                        let _ = msg.react(&ctx, '🇼').await;
                    }
                }
            }
        }

        if msg.content.starts_with("3.14") || msg.content.starts_with("3,14") {
            let content = msg.content.replace(",", ".");
            let pif = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679821480865132823066470938446095505822317253594081284811174502841027019385211055596446229489549303819644288109756659334461284756482337867831652712019091456485669234603486104543266482133936072602491412737245870066063155881748815209209628292540917153643678925903600113305305488204665213841469519415116094330572703657595919530921861173819326117931051185480744623799627495673518857527248912279381830119491298336733624406566430860213949463952247371907021798609437027705392171762931767523846748184676694051320005681271452635608277857713427577896091736371787214684409012249534301465495853710507922796892589235420199561121290219608640344181598136297747713099605187072113499999983729780499510597317328160963185950244594553469083026425223082533446850352619311881710100031378387528865875332083814206171776691473035982534904287554687311595628638823537875937519577818577805321712268066130019278766111959092164201989";

            let l = if pif.len() > content.len() {
                content.len()
            } else {
                pif.len()
            };

            let mut correct = true;

            for i in 0..l {
                if pif.chars().into_iter().nth(i) != content.chars().into_iter().nth(i) {
                    correct = false;
                    break;
                }
            }

            if correct {
                let _ = msg.react(&ctx, '✅').await;
            } else {
                let _ = msg.react(&ctx, '❌').await;
            }
        }

        if msg.guild_id.unwrap_or_default().0 == 159686161219059712 {
            if msg.content.to_lowercase().contains("ping me on nsfw!") {
                let _ = ChannelId(354294536198946817)
                    .say(&ctx, format!("<@{}>", msg.author.id))
                    .await;
            } else if msg.content.to_lowercase().contains("ping him on nsfw!") {
                let _ = ChannelId(354294536198946817)
                    .say(&ctx, "<@299624139633721345>")
                    .await;
            }
        }
    }

    /// on_raw_reaction_add event on d.py
    /// This function triggers every time a reaction gets added to a message.
    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // Ignores all reactions that come from the bot itself.
        if &add_reaction.user_id.unwrap().0 == ctx.cache.current_user_id().as_u64() {
            return;
        }

        // gets the message the reaction happened on
        let msg = if let Ok(x) = ctx
            .http
            .as_ref()
            .get_message(add_reaction.channel_id.0, add_reaction.message_id.0)
            .await
        {
            x
        } else {
            return;
        };

        // Obtain the "global" data in read mode
        let annoyed_channels = {
            let data_read = &ctx.data.read().await;
            data_read.get::<AnnoyedChannels>().unwrap().clone()
        };

        let annoy = (annoyed_channels.read().await).contains(&msg.channel_id.0);

        match add_reaction.emoji {
            // Matches custom emojis.
            ReactionType::Custom { id, .. } => {
                // If the emote is the GW version of slof, React back.
                // This also shows a couple ways to do error handling.
                if id.0 == 375_459_870_524_047_361 {
                    if let Err(why) = msg.react(&ctx, add_reaction.emoji).await {
                        error!("There was an error adding a reaction: {}", why);
                    }

                    if annoy {
                        let _ = msg
                            .channel_id
                            .say(&ctx, format!("<@{}>: qt", add_reaction.user_id.unwrap().0))
                            .await;
                    }
                }
            }
            // Matches unicode emojis.
            ReactionType::Unicode(s) => {
                if annoy {
                    // This will not be kept here for long, as i see it being very annoying eventually.
                    if s == "🤔" {
                        let _ = msg
                            .channel_id
                            .say(
                                &ctx,
                                format!(
                                    "<@{}>: What ya thinking so much about",
                                    add_reaction.user_id.unwrap().0
                                ),
                            )
                            .await;
                    }
                }

                // This makes every message sent by the bot get deleted if 🚫 is on the reactions.
                // aka If you react with 🚫 on any message sent by the bot, it will get deleted.
                // This is helpful for antispam and anti illegal content measures.
                if s == "🚫" && msg.author.id == ctx.cache.current_user_id() {
                    let _ = msg.delete(&ctx).await;
                }
            }
            // Ignore the rest of the cases.
            _ => (), // complete code
                     //_ => {}, // incomplete code / may be longer in the future
        }
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, member: Member) {
        let pool = {
            let data_read = &ctx.data.read().await;
            data_read.get::<DatabasePool>().unwrap().clone()
        };

        let data = sqlx::query!(
            "SELECT banner_user_id FROM permanent_bans WHERE guild_id = $1 AND user_id = $2",
            guild_id.0 as i64,
            member.user.id.0 as i64
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        if let Some(row) = data {
            if member
                .ban_with_reason(
                    &ctx,
                    0,
                    &format!(
                        "User ID {} has been banned PERMANENTLY by {}",
                        member.user.id.0, row.banner_user_id
                    ),
                )
                .await
                .is_err()
            {
                if let Some(channel) = guild_id.to_guild_cached(&ctx).unwrap().system_channel_id {
                    let _ = channel.say(&ctx, format!("I was unable to reban the permanently banned user <@{}>, originally banned by <@{}>", member.user.id.0, row.banner_user_id)).await;
                }
            };
        }
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        if let Err(e) = thread.id.join_thread(ctx).await {
            println!("Error in thread join! (ID {}): {}", thread.id, e);
        }
    }

    async fn voice_server_update(&self, ctx: Context, vsu: VoiceServerUpdateEvent) {
        let guild_id = vsu.guild_id.unwrap();
        let call = ctx
            .data
            .read()
            .await
            .get::<SongbirdCalls>()
            .unwrap()
            .clone()
            .read()
            .await
            .get(&guild_id)
            .cloned();

        if let Some(call) = call {
            let connection_info = call.lock().await.current_connection().unwrap().clone();
            let lavalink = ctx.data.read().await.get::<Lavalink>().unwrap().clone();

            tokio::spawn(async move {
                trace!("(Voice Server Update) Call pause");
                if let Err(why) = lavalink.pause(guild_id).await {
                    error!(
                        "Error when pausing on voice_server_update: {}",
                        why
                    );
                }

                sleep(Duration::from_millis(100)).await;

                trace!("(Voice Server Update) Call create_session");
                if let Err(why) = lavalink.create_session_with_songbird(&connection_info).await {
                    error!(
                        "Error when creating a session on voice_server_update: {}",
                        why
                    );
                }

                sleep(Duration::from_millis(1000)).await;

                trace!("(Voice Server Update) Call resume");
                if let Err(why) = lavalink.resume(guild_id).await {
                    error!(
                        "Error when resuming on voice_server_update: {}",
                        why
                    );
                }
            });
        }
    }
}
