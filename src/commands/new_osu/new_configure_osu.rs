use crate::global_data::{DatabasePool, OsuHttpClient};
use crate::utils::osu_model::*;

use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::*,
    model::interactions::message_component::*,
    model::interactions::InteractionResponseType,
    prelude::Context,
};

#[derive(Debug, Default)]
struct UserConfiguration {
    osu_id: i32,
    instant_recent: bool,
}

async fn parse_username(ctx: &Context, content: &str) -> CommandResult<i32> {
    let mut osu_id = 0;

    if let Ok(user_id) = content.parse::<i32>() {
        osu_id = user_id;
    } else {
        if content.starts_with("http") {
            let mut initial_split = content.split("ppy.sh/");

            if let Some(second) = initial_split.nth(1) {
                let mut user_split = second.split('/');

                if let Some(user) = user_split.nth(1) {
                    if let Ok(user_id) = user.parse::<i32>() {
                        osu_id = user_id;
                    }
                }
            }
        } else {
            let client_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<OsuHttpClient>().unwrap().clone()
            };

            let user_data = client_lock
                .read()
                .await
                .get(&format!("https://osu.ppy.sh/api/v2/users/{}", &content))
                .send()
                .await?
                .json::<OsuUser>()
                .await;

            if let Ok(u) = user_data {
                osu_id = u.id as i32;
            }
        }
    }

    Ok(osu_id)
}

#[command]
#[aliases(nosuc, new_osuc, newosuc, n_osuc, noc)]
async fn new_configure_osu(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let db_data = sqlx::query!(
        "SELECT * FROM osu WHERE discord_id = $1",
        msg.author.id.0 as i64
    )
    .fetch_optional(&pool)
    .await?;

    let mut user_config = UserConfiguration::default();

    if let Some(data) = db_data {
        user_config.osu_id = data.osu_id;
        user_config.instant_recent = data.instant_recent;
    }

    if user_config.osu_id == 0 {
        let mut bot_msg = msg.reply_ping(ctx, "It appears that you do not have an osu! account configured for discord.\nPlease, send a message with one of the following:\n- osu! id: `9410384`\n- osu! username: `vicky5124` (If your username is only numbers, send your ID instead)\n- Profile Link: `https://osu.ppy.sh/users/9410384`\n\nYou have 2 minutes to provide any of the valid options.").await?;

        let resp_msg = msg
            .author
            .await_reply(ctx)
            .channel_id(msg.channel_id)
            .timeout(Duration::from_secs(120))
            .await;

        if let Some(message) = resp_msg {
            user_config.osu_id = parse_username(ctx, &message.content).await?;
        }

        let button_confim = "b_confirm";
        let button_deny = "b_dny";

        if user_config.osu_id == 0 {
            bot_msg.edit(ctx, |m| {
                m.content("The information you provided lead to an invalid user, please, re-run the command if you want to try again.")
            }).await?;
        } else {
            bot_msg
                .edit(ctx, |m| {
                    m.content(format!(
                        "The user https://osu.ppy.sh/users/{} was found, is it correct?",
                        user_config.osu_id
                    ));
                    m.components(|c| {
                        c.create_action_row(|ar| {
                            ar.create_button(|b| {
                                b.style(ButtonStyle::Success);
                                b.label("Yes!!!");
                                b.emoji(ReactionType::Unicode("✔️".to_string()));
                                b.custom_id(button_confim)
                            });
                            ar.create_button(|b| {
                                b.style(ButtonStyle::Danger);
                                b.label("What? noooo");
                                b.emoji(ReactionType::Unicode("✖️".to_string()));
                                b.custom_id(button_deny)
                            });
                            ar
                        })
                    })
                })
                .await?;

            let interaction_response = bot_msg
                .await_component_interaction(ctx)
                .author_id(msg.author.id.0)
                .timeout(Duration::from_secs(60))
                .await;

            if let Some(interaction_data) = interaction_response {
                if interaction_data.data.custom_id == button_confim {
                    sqlx::query!("INSERT INTO osu (discord_id, osu_id, instant_recent) VALUES ($1, $2, $3) ON CONFLICT (discord_id) DO UPDATE SET osu_id = $2, instant_recent = $3", msg.author.id.0 as i64, user_config.osu_id, user_config.instant_recent)
                        .execute(&pool)
                        .await?;

                    bot_msg
                        .edit(ctx, |m| {
                            m.content(format!("Current configuration for you: <https://osu.ppy.sh/users/{0}>```md\n- User ID: {0}\n- Instant Recent: {1}```", user_config.osu_id, user_config.instant_recent));
                            m.components(|c| c.set_action_rows(vec![]));
                            m.suppress_embeds(true);
                            m
                        })
                        .await?;

                    return Ok(());
                }

                interaction_data
                    .create_interaction_response(ctx, |ir| {
                        ir.kind(InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await?;
            }

            bot_msg
                .edit(ctx, |m| {
                    m.content("User configuration cancelled, please, re-run the command if you want to try again.");
                    m.components(|c| c.set_action_rows(vec![]));
                    m.suppress_embeds(true);
                    m
                })
                .await?;
        }
    } else {
        let button_username = "b_username";
        let button_instant_recent = "b_ir";

        let mut bot_msg = msg
            .channel_id
            .send_message(ctx, |m| {
                m.reference_message(msg);
                m.content("What do you wish to configure?");
                m.components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Primary);
                            b.label("Username");
                            b.emoji(ReactionType::Unicode("🗒️".to_string()));
                            b.custom_id(button_username)
                        });
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Primary);
                            b.label("Toggle instant recent");
                            b.emoji(ReactionType::Unicode("🔄".to_string()));
                            b.custom_id(button_instant_recent)
                        });
                        ar
                    })
                })
            })
            .await?;

        let interaction_response = bot_msg
            .await_component_interaction(ctx)
            .author_id(msg.author.id.0)
            .timeout(Duration::from_secs(60))
            .await;

        if let Some(interaction_data) = interaction_response {
            interaction_data
                .create_interaction_response(ctx, |ir| {
                    ir.kind(InteractionResponseType::DeferredUpdateMessage)
                })
                .await?;

            if interaction_data.data.custom_id == button_username {
                bot_msg.edit(ctx, |m| {
                    m.content("Please, send a message with one of the following:\n- osu! id: `9410384`\n- osu! username: `vicky5124` (If your username is only numbers, send your ID instead)\n- Profile Link: `https://osu.ppy.sh/users/9410384`\n\nYou have 2 minutes to provide any of the valid options.");
                    m.components(|c| c.set_action_rows(vec![]));
                    m
                }).await?;

                let resp_msg = msg
                    .author
                    .await_reply(ctx)
                    .channel_id(msg.channel_id)
                    .timeout(Duration::from_secs(120))
                    .await;

                if let Some(message) = resp_msg {
                    let osu_id = parse_username(ctx, &message.content).await?;

                    if osu_id != 0 {
                        user_config.osu_id = osu_id;
                    } else {
                        bot_msg
                            .edit(ctx, |m| {
                                m.content("The user provided was invalid, please, re-run the command if you want to try again.");
                                m.components(|c| c.set_action_rows(vec![]));
                                m.suppress_embeds(true);
                                m
                            })
                            .await?;
                    }
                }
            } else if interaction_data.data.custom_id == button_instant_recent {
                user_config.instant_recent = !user_config.instant_recent;
            }

            sqlx::query!("INSERT INTO osu (discord_id, osu_id, instant_recent) VALUES ($1, $2, $3) ON CONFLICT (discord_id) DO UPDATE SET osu_id = $2, instant_recent = $3", msg.author.id.0 as i64, user_config.osu_id, user_config.instant_recent)
                .execute(&pool)
                .await?;

            bot_msg
                .edit(ctx, |m| {
                    m.content(format!("Current configuration for you: <https://osu.ppy.sh/users/{0}>```md\n- User ID: {0}\n- Instant Recent: {1}```", user_config.osu_id, user_config.instant_recent));
                    m.components(|c| c.set_action_rows(vec![]));
                    m.suppress_embeds(true);
                    m
                })
                .await?;
        } else {
            bot_msg
                .edit(ctx, |m| {
                    m.content("User configuration timed out.");
                    m.components(|c| c.set_action_rows(vec![]));
                    m.suppress_embeds(true);
                    m
                })
                .await?;
        }
    }

    Ok(())
}
