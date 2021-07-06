use crate::utils::osu_model::*;
//use crate::utils::osu::*;
use crate::commands::osu::progress_math;
use crate::global_data::OsuHttpClient;
use crate::utils::basic_functions::capitalize_first;

use std::time::Duration;

use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use osu_perf::{Accuracy, Difficulty, Map, MapStatistics, Mods, PpV2};
use rand::{thread_rng, Rng};
use uuid::Uuid;

use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::*,
    model::interactions::message_component::*,
    model::interactions::InteractionResponseType,
    prelude::Context,
};

/// The new recent command, currently WIP.
///
/// Note: if you are not getting your recent scores, it's because your nick or username in discord doesn't match the osu! username, if that's the case, call the command with the username afterwards.
///
/// `new_recent vicky5124`
#[command]
#[aliases(nrc, newrc, newrececnt, new_rc, n_rc, nrs, newrs, new_rs, n_rs)]
async fn new_recent(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut message = msg.reply(ctx, "Loading recent scores...").await?;
    let mut content = String::new();
    let mut content_swapped = false;

    let uuid_prev = Uuid::new_v4().to_string();
    let uuid_next = Uuid::new_v4().to_string();
    let uuid_keep = Uuid::new_v4().to_string();
    let uuid_done = Uuid::new_v4().to_string();

    let mut to_keep = 0;

    let raw_user = if let Ok(x) = args.single_quoted() {
        x
    } else {
        msg.member(ctx).await?.display_name().into_owned()
    };

    let client_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<OsuHttpClient>().unwrap().clone()
    };

    let user = {
        let user_data = client_lock
            .read()
            .await
            .get(&format!("https://osu.ppy.sh/api/v2/users/{}", &raw_user))
            .send()
            .await?
            .json::<OsuUser>()
            .await;
        if let Ok(u) = user_data {
            u.id
        } else {
            message
                .edit(ctx, |m| {
                    m.content(format!("Invalid osu! user: `{}`", raw_user))
                })
                .await?;
            return Ok(());
        }
    };

    let res_recent_data = client_lock
        .read()
        .await
        .get(&format!(
            "https://osu.ppy.sh/api/v2/users/{}/scores/recent?mode=osu&include_fails=1&limit=50",
            &user
        ))
        .send()
        .await?
        .json::<Recent>()
        .await;

    let recent_data = if let Ok(x) = res_recent_data {
        x
    } else {
        message
            .edit(ctx, |m| {
                m.content(format!("No recent plays for user {}", user))
            })
            .await?;
        return Ok(());
    };

    trace!("{:#?}", &recent_data);

    if recent_data.is_empty() {
        message
            .edit(ctx, |m| {
                m.content(format!("No recent plays for user {}", user))
            })
            .await?;
        return Ok(());
    }

    let chunks = recent_data
        .iter()
        .chunks(3)
        .into_iter()
        .map(|i| i.map(|i| i.to_owned()).collect::<Vec<&RecentElement>>())
        .collect::<Vec<Vec<&RecentElement>>>();

    let max = chunks.len() - 1;
    let mut index = 0;

    let mut current_embeds = vec![];
    let mut current_urls = vec![];

    loop {
        for (idx, data) in chunks[index].iter().enumerate() {
            let beatmap_file = client_lock
                .read()
                .await
                .get(&format!("https://osu.ppy.sh/web/maps/{}", &data.beatmap.id))
                .send()
                .await?
                .text()
                .await?;

            let map = if let Ok(x) = Map::parse(beatmap_file.as_bytes()) { x } else { continue }; // TODO: Self::from_str()
            let mods = Mods::from_strs(&data.mods);
            let difficulty = Difficulty::calc(&map, mods);

            let map_statistics = MapStatistics::new(
                data.beatmap.ar,
                data.beatmap.accuracy,
                data.beatmap.cs,
                data.beatmap.drain,
            )
            .with_mods(mods);

            let accuracy = Accuracy {
                n300: data.statistics.count_300 as i32,
                n100: data.statistics.count_100 as i32,
                n50: data.statistics.count_50 as i32,
                misses: data.statistics.count_miss as i32,
            }; // TODO: Self::new()

            let accuracy_fc = Accuracy {
                n300: (data.statistics.count_300 + data.statistics.count_miss) as i32,
                n100: data.statistics.count_100 as i32,
                n50: data.statistics.count_50 as i32,
                misses: 0,
            }; // TODO: Self::new()

            let progress: f32 = progress_math(
                data.beatmap.count_circles as f32,
                data.beatmap.count_sliders as f32,
                data.beatmap.count_spinners as f32,
                data.statistics.count_300 as f32,
                data.statistics.count_100 as f32,
                data.statistics.count_50 as f32,
                data.statistics.count_miss as f32,
            );

            trace!("Calculating v1 for {}", data.id);

            let pp_v1 = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                Some(data.max_combo as u32),
                mods,
                accuracy,
                1,
                None,
            );

            trace!("Calculating v2 for {}", data.id);

            let pp_v2 = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                Some(data.max_combo as u32),
                mods,
                accuracy,
                2,
                None,
            );

            trace!("Calculating v1 fc for {}", data.id);

            let pp_v1_fc = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                None,
                mods,
                accuracy_fc,
                1,
                None,
            );

            trace!("Calculating v2 fc for {}", data.id);

            let pp_v2_fc = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                None,
                mods,
                accuracy_fc,
                2,
                None,
            );

            debug!(
                "{}: {} {} {} {}",
                data.id, pp_v1.total, pp_v2.total, pp_v1_fc.total, pp_v2_fc.total
            );

            let mut embed = CreateEmbed::default();

            embed.title({
                if data.beatmapset.artist != data.beatmapset.artist_unicode
                    && data.beatmapset.title != data.beatmapset.title_unicode
                {
                    format!(
                        "{} => {} ({}) - {} ({})",
                        idx + 1,
                        data.beatmapset.artist,
                        data.beatmapset.artist_unicode,
                        data.beatmapset.title,
                        data.beatmapset.title_unicode
                    )
                } else if data.beatmapset.artist != data.beatmapset.artist_unicode {
                    format!(
                        "{} => {} ({}) - {}",
                        idx + 1,
                        data.beatmapset.artist,
                        data.beatmapset.artist_unicode,
                        data.beatmapset.title
                    )
                } else if data.beatmapset.title != data.beatmapset.title_unicode {
                    format!(
                        "{} => {} - {} ({})",
                        idx + 1,
                        data.beatmapset.artist,
                        data.beatmapset.title,
                        data.beatmapset.title_unicode
                    )
                } else {
                    format!(
                        "{} => {} - {}",
                        idx + 1,
                        data.beatmapset.artist,
                        data.beatmapset.title
                    )
                }
            });

            if !current_urls.contains(&data.beatmap.url) {
                embed.url(&data.beatmap.url);
                current_urls.push(data.beatmap.url.to_string());
            }

            // thread_rng and async issues
            {
                let mut rng = thread_rng();
                embed.colour(rng.gen_range(0..0xFFFFFF));
            }

            embed.image(&data.beatmapset.covers.cover_2x);
            embed.footer(|f| {
                f.icon_url({
                    if data.rank == "F" {
                        "https://5124.mywire.org/HDD/Downloads/BoneF.png".to_string()
                    } else if data.rank == "SS" {
                        "https://s.ppy.sh/images/XS.png".to_string()
                    } else if data.rank == "SSH" || data.rank == "XSH" {
                        "https://s.ppy.sh/images/XH.png".to_string()
                    } else {
                        format!("https://s.ppy.sh/images/{}.png", data.rank.to_uppercase())
                    }
                });
                f.text({
                    format!(
                        "{:.2}pp SV1 | {:.2}pp SV2 | {:.2}pp SV1 FC | {:.2}pp SV2 FC\nProgress: {:.2}% | {} | Played",
                        pp_v1.total,
                        pp_v2.total,
                        pp_v1_fc.total,
                        pp_v2_fc.total,
                        progress,
                        capitalize_first(&data.beatmapset.status),
                    )
                })
            });
            embed.timestamp(&data.created_at);
            embed.description(
                format!(
                    "__Mapped by **[{}](https://osu.ppy.sh/users/{})**__ | Difficulty **{}** {}\n**{:.2}\\*** ({:.2}\\* Aim | {:.2}\\* Speed)** {}**\nAR {:.1} | OD {:.1} | CS {:.1} | HP {:.1}\n**{}** ┇ **x{}** / {} {}\n**{:.2}%** ┇ {} - {} - {} - ~~{}~~",
                    data.beatmapset.creator,
                    data.beatmapset.user_id,
                    data.beatmap.version,
                    if data.beatmapset.nsfw { "*Explicit*" } else { "" },
                    difficulty.total,
                    difficulty.aim,
                    difficulty.speed,
                    data.mods.join(", "),
                    difficulty.stats.ar,
                    difficulty.stats.od,
                    difficulty.stats.cs,
                    difficulty.stats.hp,
                    data.score.to_formatted_string(&Locale::en),
                    data.max_combo,
                    map.max_combo,
                    if data.perfect { "**FC**" } else { "" },
                    accuracy.value() * 100.0,
                    data.statistics.count_300,
                    data.statistics.count_100,
                    data.statistics.count_50,
                    data.statistics.count_miss,
                )
            );

            embed.author(|a| {
                if data.user.is_supporter {
                    a.name(format!("{} 💜", data.user.username));
                } else {
                    a.name(&data.user.username);
                }
                a.icon_url(&data.user.avatar_url);
                a.url(format!("https://osu.ppy.sh/u/{}", data.user.id));

                a
            });

            if !content_swapped {
                content = format!("Displaying recent scores for {}", data.user.username);
                content_swapped = true;
            }

            current_embeds.push(embed);
        }

        // -----

        message.edit(ctx, |m| {
            m.content(&content);
            m.set_embeds(current_embeds.clone());
            if recent_data.len() > 1 {
                m.components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Secondary);
                            b.label("Past Scores");
                            b.emoji(ReactionType::Unicode("⬅️".to_string()));
                            b.custom_id(&uuid_prev)
                        });
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Secondary);
                            b.label("Future Scores");
                            b.emoji(ReactionType::Unicode("➡️".to_string()));
                            b.custom_id(&uuid_next)
                        });
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Success);
                            b.label("Done!");
                            b.emoji(ReactionType::Unicode("✅".to_string()));
                            b.custom_id(&uuid_done)
                        });
                        ar
                    });
                    c.create_action_row(|ar| {
                        ar.create_select_menu(|sm| {
                            sm.placeholder("Keep Score...");
                            sm.min_values(1);
                            sm.max_values(1);
                            sm.custom_id(&uuid_keep);

                            sm.options(|o| {
                                for (idx, _) in chunks[index].iter().enumerate() {
                                    o.create_option(|o| {
                                        match idx {
                                            0 => {
                                                if to_keep == 0 {
                                                    o.default_selection(true);
                                                }
                                                o.label("Keep First Score");
                                                o.value("0");
                                                o.description("Remove the rest of scores and keep the 1st score.");
                                            }
                                            1 => {
                                                if to_keep == 1 {
                                                    o.default_selection(true);
                                                }
                                                o.label("Keep Second Score");
                                                o.value("1");
                                                o.description("Remove the rest of scores and keep the 2nd score.");
                                            }
                                            2 => {
                                                if to_keep == 2 {
                                                    o.default_selection(true);
                                                }
                                                o.label("Keep Third Score");
                                                o.value("2");
                                                o.description("Remove the rest of scores and keep the 3rd score.");
                                            }
                                            3 => {
                                                if to_keep == 3 {
                                                    o.default_selection(true);
                                                }
                                                o.label("Keep Forth Score");
                                                o.value("3");
                                                o.description("Remove the rest of scores and keep the 4th score.");
                                            }
                                            4 => {
                                                if to_keep == 4 {
                                                    o.default_selection(true);
                                                }
                                                o.label("Keep Fifth Score");
                                                o.value("4");
                                                o.description("Remove the rest of scores and keep the 5th score.");
                                            }
                                            _ => ()
                                        }
                                        o
                                    });
                                }
                                o
                            });
                            sm
                        });
                        ar
                    });
                    c
                });
            }
            m
        }).await?;

        if recent_data.len() == 1 {
            break;
        }

        let mov_uuid_prev = uuid_prev.clone();
        let mov_uuid_next = uuid_next.clone();
        let mov_uuid_keep = uuid_keep.clone();
        let mov_uuid_done = uuid_done.clone();

        let mci = message
            .await_component_interaction(ctx)
            .author_id(msg.author.id.0)
            .timeout(Duration::from_secs(60))
            .filter(move |mci| match mci.data.component_type {
                ComponentType::SelectMenu => {
                    if mci.data.custom_id == mov_uuid_keep {
                        true
                    } else {
                        false
                    }
                }
                ComponentType::Button => {
                    if mci.data.custom_id == mov_uuid_prev
                        || mci.data.custom_id == mov_uuid_next
                        || mci.data.custom_id == mov_uuid_done
                    {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            })
            .await;

        if let Some(mci) = mci {
            current_embeds = vec![];
            current_urls = vec![];

            if mci.data.values.is_empty() {
                if mci.data.custom_id == uuid_prev {
                    if index == max {
                        index = 0;
                    } else {
                        index += 1;
                    }
                } else if mci.data.custom_id == uuid_next {
                    if index == 0 {
                        index = max;
                    } else {
                        index -= 1;
                    }
                } else if mci.data.custom_id == uuid_done {
                    mci.create_interaction_response(ctx, |ir| {
                        ir.kind(InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await?;

                    break;
                }
            } else {
                to_keep = mci.data.values[0].parse().unwrap();
            }

            mci.create_interaction_response(ctx, |ir| {
                ir.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await?;
        } else {
            break;
        }
    }

    let data = &chunks[index][to_keep];

    let beatmap_file = client_lock
        .read()
        .await
        .get(&format!("https://osu.ppy.sh/web/maps/{}", &data.beatmap.id))
        .send()
        .await?
        .text()
        .await?;

    message.edit(ctx, |m| {
        m.content(format!("`{}`", data.beatmap.id));
        m.components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.style(ButtonStyle::Link);
                    b.url(format!("https:{}", data.beatmapset.preview_url));
                    b.emoji(ReactionType::Unicode("🔊".to_string()));
                    b.label("Audio Preview")
                });
                ar.create_button(|b| {
                    b.style(ButtonStyle::Link);
                    b.url(format!("https://osu.ppy.sh/beatmapsets/{}/download", data.beatmapset.id));
                    b.emoji(ReactionType::Unicode("📁".to_string()));
                    b.label("Download OSZ")
                });
                if let Some(id) = data.best_id {
                    ar.create_button(|b| {
                        b.style(ButtonStyle::Link);
                        b.url(format!("https://osu.ppy.sh/scores/osu/{}", id));
                        b.label("Share Score!")
                    });
                }

                ar
            })
        });
        m.embed(|embed| {
            let map = Map::parse(beatmap_file.as_bytes()).unwrap(); // TODO: Self::from_str()
            let mods = Mods::from_strs(&data.mods);
            let difficulty = Difficulty::calc(&map, mods);

            let map_statistics = MapStatistics::new(
                data.beatmap.ar,
                data.beatmap.accuracy,
                data.beatmap.cs,
                data.beatmap.drain,
            )
            .with_mods(mods);

            let accuracy = Accuracy {
                n300: data.statistics.count_300 as i32,
                n100: data.statistics.count_100 as i32,
                n50: data.statistics.count_50 as i32,
                misses: data.statistics.count_miss as i32,
            }; // TODO: Self::new()

            let accuracy_fc = Accuracy {
                n300: (data.statistics.count_300 + data.statistics.count_miss) as i32,
                n100: data.statistics.count_100 as i32,
                n50: data.statistics.count_50 as i32,
                misses: 0,
            }; // TODO: Self::new()

            let progress: f32 = progress_math(
                data.beatmap.count_circles as f32,
                data.beatmap.count_sliders as f32,
                data.beatmap.count_spinners as f32,
                data.statistics.count_300 as f32,
                data.statistics.count_100 as f32,
                data.statistics.count_50 as f32,
                data.statistics.count_miss as f32,
            );

            let pp_v1 = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                Some(data.max_combo as u32),
                mods,
                accuracy,
                1,
                None,
            );

            let pp_v2 = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                Some(data.max_combo as u32),
                mods,
                accuracy,
                2,
                None,
            );

            let pp_v1_fc = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                None,
                mods,
                accuracy_fc,
                1,
                None,
            );

            let pp_v2_fc = PpV2::pp(
                &map,
                &map_statistics,
                difficulty.aim,
                difficulty.speed,
                None,
                mods,
                accuracy_fc,
                2,
                None,
            );

            embed.title({
                if data.beatmapset.artist != data.beatmapset.artist_unicode
                    && data.beatmapset.title != data.beatmapset.title_unicode
                {
                    format!(
                        "{} ({}) - {} ({})",
                        data.beatmapset.artist,
                        data.beatmapset.artist_unicode,
                        data.beatmapset.title,
                        data.beatmapset.title_unicode
                    )
                } else if data.beatmapset.artist != data.beatmapset.artist_unicode {
                    format!(
                        "{} ({}) - {}",
                        data.beatmapset.artist,
                        data.beatmapset.artist_unicode,
                        data.beatmapset.title
                    )
                } else if data.beatmapset.title != data.beatmapset.title_unicode {
                    format!(
                        "{} - {} ({})",
                        data.beatmapset.artist,
                        data.beatmapset.title,
                        data.beatmapset.title_unicode
                    )
                } else {
                    format!("{} - {}", data.beatmapset.artist, data.beatmapset.title)
                }
            });

            embed.url(&data.beatmap.url);

            // thread_rng and async issues
            {
                let mut rng = thread_rng();
                embed.colour(rng.gen_range(0..0xFFFFFF));
            }

            embed.image(&data.beatmapset.covers.cover_2x);
            embed.footer(|f| {
                f.icon_url({
                    if data.rank == "F" {
                        "https://5124.mywire.org/HDD/Downloads/BoneF.png".to_string()
                    } else if data.rank == "SS" {
                        "https://s.ppy.sh/images/XS.png".to_string()
                    } else if data.rank == "SSH" || data.rank == "XSH" {
                        "https://s.ppy.sh/images/XH.png".to_string()
                    } else {
                        format!("https://s.ppy.sh/images/{}.png", data.rank.to_uppercase())
                    }
                });
                f.text({
                    format!(
                        "{:.2}pp SV1 | {:.2}pp SV2 | {:.2}pp SV1 FC | {:.2}pp SV2 FC\nProgress: {:.2}% | {} | Played",
                        pp_v1.total,
                        pp_v2.total,
                        pp_v1_fc.total,
                        pp_v2_fc.total,
                        progress,
                        capitalize_first(&data.beatmapset.status),
                    )
                })
            });
            embed.timestamp(&data.created_at);
            embed.description(
                format!(
                    "__Mapped by **[{}](https://osu.ppy.sh/users/{})**__ | Difficulty **{}** {}\n**{:.2}\\*** ({:.2}\\* Aim | {:.2}\\* Speed)** {}**\nAR {:.1} | OD {:.1} | CS {:.1} | HP {:.1}\n**{}** ┇ **x{}** / {} {}\n**{:.2}%** ┇ {} - {} - {} - ~~{}~~\nScoreable: {} ┇ {} ❤️\nLast updated: <t:{}:F>",
                    data.beatmapset.creator,
                    data.beatmapset.user_id,
                    data.beatmap.version,
                    if data.beatmapset.nsfw { "*Explicit*" } else { "" },
                    difficulty.total,
                    difficulty.aim,
                    difficulty.speed,
                    data.mods.join(", "),
                    difficulty.stats.ar,
                    difficulty.stats.od,
                    difficulty.stats.cs,
                    difficulty.stats.hp,
                    data.score.to_formatted_string(&Locale::en),
                    data.max_combo,
                    map.max_combo,
                    if data.perfect { "**FC**" } else { "" },
                    accuracy.value() * 100.0,
                    data.statistics.count_300,
                    data.statistics.count_100,
                    data.statistics.count_50,
                    data.statistics.count_miss,
                    if data.beatmap.is_scoreable { "✅" } else { "❌" },
                    data.beatmapset.favourite_count,
                    data.beatmap.last_updated.timestamp(),
                )
            );

            embed.author(|a| {
                if data.user.is_supporter {
                    a.name(format!("{} 💜", data.user.username));
                } else {
                    a.name(&data.user.username);
                }
                a.icon_url(&data.user.avatar_url);
                a.url(format!("https://osu.ppy.sh/u/{}", data.user.id));

                a
            });

            embed
        })
    }).await?;

    Ok(())
}
