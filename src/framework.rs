// Import this 2 commands in specific with a different name
// as they interfere with the configuration commands that are also being imported.
use crate::commands::booru::{BEST_BOY_COMMAND as BG_COMMAND, BEST_GIRL_COMMAND as BB_COMMAND};
use crate::commands::meta::PREFIX_COMMAND as PREFIXES_COMMAND;

use crate::commands::booru::*; // Import everything from the booru module.
use crate::commands::configuration::*; // Import everything from the configuration module.
use crate::commands::dictionary::*; // Import everything from the dictionary module.
use crate::commands::fun::*; // Import everything from the fun module.
use crate::commands::games::*; // Import everything from the games module.
use crate::commands::image_manipulation::*; // Import everything from the image manipulation module.
use crate::commands::meta::*; // Import everything from the meta module.
use crate::commands::moderation::*; // Import everything from the moderation module.
use crate::commands::music::*; // Import everything from the configuration module.
use crate::commands::new_osu::*; // Import everything from the new osu module.
use crate::commands::osu::*; // Import everything from the osu module.
use crate::commands::sankaku::*; // Import everything from the sankaku booru module.
use crate::commands::serenity_docs::*; // Import everything from the serenity_docs module.

use std::collections::HashSet;

use serenity::{
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::prelude::*,
    prelude::*,
    utils::Colour, // To change the embed help color
};
#[group("Master")]
#[sub_groups(
    Meta,
    Sankaku,
    Osu,
    Fun,
    Music,
    AllBoorus,
    ImageManipulation,
    Mod,
    SerenityDocs,
    Games
)]
pub struct Master;

// The basic commands group is being defined here.
// this group includes the commands that basically every bot has, nothing really special.
#[group("Meta")]
#[description = "All the basic commands that basically every bot has."]
#[commands(
    ping,
    test,
    invite,
    source,
    todo,
    prefixes,
    about,
    changelog,
    terms_of_service,
    issues,
    eval,
    rust,
    admin_eval,
    force_cache_ready,
)]
pub struct Meta;

// The SankakuComplex command group.
// This group contains commands for the variants Chan and Idol of the sankaku boorus.
#[group("Sankaku")]
#[help_available(false)] // So the group doesn't show up on the help command.
#[description = "All the NSFW/BSFW related commands."]
#[commands(idol, chan)]
pub struct Sankaku;

// The osu! command group.
// This group contains all the osu! related commands.
#[group("osu!")]
#[description = "All the osu! related commands"]
#[commands(configure_osu, recent, score, osu_profile, osu_top, beatmap_pp)]
pub struct Osu;

#[group("new osu!")]
#[description = "All the osu! related commands"]
#[commands(new_recent, new_configure_osu)]
pub struct NewOsu;

// The Booru command group.
// This group will contain every single command from every booru that gets implemented.
// As you can see on the last line, the description also supports url markdown.
#[group("Image Boards")]
#[description = "All the booru related commands.\n\
Available parameters:
`-x` Explicit
`-q` Questionable
`-s` Safe
`-n` Non Safe (Random between E or Q)

Inspired by -GN's WaifuBot ([source](https://github.com/isakvik/waifubot/))"]
#[commands(booru_command, BB, BG, n_hentai, sauce)] // We imported BB_COMMAND and BG_COMMAND, but this macro automatically adds _COMMAND, so we don't put that.
pub struct AllBoorus;

// The Image Manipulation command group.
// This group contains all the commands that manipulate images.
#[group("Image Manipulation")]
#[description = "All the image manipulaiton based commands."]
#[commands(gray, pride, pride_pre_grayscaled)]
pub struct ImageManipulation;

// The FUN command group.
// Where all the random commands go into lol
#[group("Fun")]
#[description = "All the random and fun commands."]
#[commands(
    profile,
    qr,
    urban,
    dictionary,
    translate,
    duck_duck_go,
    encrypt,
    decrypt,
    calculator,
    remind_me,
    uwufy
)]
pub struct Fun;

// The FUN command group.
// Where all the random commands go into lol
#[group("Games")]
#[description = "All the games the bot has. (Just for fun)"]
#[commands(tic_tac_toe, higher_or_lower)]
pub struct Games;

// The moderation command group.
#[group("Moderation")]
#[description = "All the moderation related commands."]
#[commands(
    kick,
    clear,
    ban,
    permanent_ban,
    permanent_mute,
    temporal_mute,
    permanent_self_mute,
    temporal_self_mute
)]
pub struct Mod;

// The music command group.
#[group("Music")]
#[description = "All the voice and music related commands."]
#[only_in("guilds")]
#[commands(
    join,
    leave,
    play,
    play_playlist,
    pause,
    resume,
    stop,
    skip,
    remove,
    seek,
    shuffle,
    queue,
    clear_queue,
    now_playing,
    equalize,
    equalize_band
)]
pub struct Music;

#[group("Serenity Documentation")]
#[description = "All the commands related to serenity's documentation."]
#[commands(example, rtfm)]
pub struct SerenityDocs;

// The configuration command.
// Technically a group, but it only has a single command.
#[group("Configuration")]
#[description = "All the configuration related commands.
Basic usage:
`config user VALUE DATA`
`config guild VALUE DATA`
`config channel VALUE DATA`"]
#[prefixes("config", "configure", "conf")]
#[commands(guild, channel, user)]
pub struct Configuration;

// This is a custom help command.
// Each line has the explaination that is required.
#[help]
// This is the basic help message
// We use \ at the end of the line to easily allow for newlines visually on the code.
#[individual_command_tip = "Hello!

If you would like to get more information about a specific command or group, you can just pass it as a command argument; like so: `help configuration`

NOTE: All the command examples through out the help will be shown without prefix, add whatever command prefix is configured on the server.
By default it's a mention or `.`, but it can be configured using `configure guild prefix n!` replacing `n!` with the prefix of choice.

You can react with 🚫 on *any* message sent by the bot to delete it.
Exceptions to this rule include logging messages, some notifications and webhook messages.\n"]
// This is the text that gets displayed when a given parameter was not found for information.
#[command_not_found_text = "Could not find: `{}`."]
// This is the ~~strikethrough~~ text.
#[strikethrough_commands_tip_in_dm = "~~`Strikethrough commands`~~ are unavailabe because the bot is unable to run them."]
#[strikethrough_commands_tip_in_guild = "~~`Strikethrough commands`~~ are unavailabe because the bot is unable to run them."]
// This is the level of similarities between the given argument and possible other arguments.
// This is used to give suggestions in case of a typo.
#[max_levenshtein_distance(3)]
// This makes it so specific sections don't get showed to the user if they don't have the
// permission to use them.
#[lacking_permissions = "Hide"]
// In the case of just lacking a role to use whatever is necessary, nothing will happen when
// setting it to "Nothing", rn it just strikes the option.
#[lacking_role = "Hide"]
// In the case of being on the wrong channel type (either DM for Guild only commands or vicecersa)
// the command will be ~~striked~~
#[wrong_channel = "Strike"]
// This will change the text that appears on groups that have a custom prefix
#[group_prefix = "Prefix commands"]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let mut ho = help_options.clone();
    // Changing the color of the embed sidebar, because the default one is ugly :P
    ho.embed_error_colour = Colour::from_rgb(255, 30, 30);
    ho.embed_success_colour = Colour::from_rgb(141, 91, 255);

    let _ = help_commands::with_embeds(ctx, msg, args, &ho, groups, owners).await;
    Ok(())
}
