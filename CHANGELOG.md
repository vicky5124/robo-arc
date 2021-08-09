# Changelog

## 0.3.3-alpha

### Features

- Switch to api v9.
- Added new recent command.
- Added new osu configurator.
- Made the invite command send a link with slash command support

### Bugs

- Fix serenity examples due to move to songbird

### Dependencies

- Removed failure.
- Switched loc with tokei.

### TRAAAAAA

- Transitioned mtf

## 0.3.2-alpha

### Features

- Switch to api v8.
- Auto-delete spammy messages in music commands.
- Added custom merge algorythms to pride.
- Added songbird to rtfm.
- Added webp support for profile pictures in image manipulation.
- Added remove command.
- Added public eval command.
- Addedd rust playground command.
- Added support for stage channels.
- Added music equalization.
- Alias tr tl to translate.
- Add automated build options to the rust command.
- Added file support for eval and rust.

### Bugs

- Remove the new serenity default prefix.
- Wrong field name on message_delete log.
- Fixed recent scores not sending if user account is too new.
- Replaced block_in_place with spawn_blocking.
- Missing minimum arguments on urban.
- Fixed broken auto-unmute on fail, and fixed message notifications.

### Logs

- Removed support for builtin emojis on logs.
- Log message structure on command invoke.

### Dependencies

- Updated to tokio 1.0.
- Removed racer.
- Updated everything.
- Switched serenity to next branch.
- Switched from OpenSSL to Rustls.
- Photon and Songbird are no longer git.

### Codebase

- Split the main file.
- Fixed all clippy warnings.
- Format with nightly.

## 0.3.1-alpha

### Dependencies

- Switched music to Songbird Gateway.
- Updated lavalink-rs to use Songbird.
- Updated serenity to the last v6 commit of the next branch.
- Updated photon-rs.
- Updated twilight on eval.

### Bugs

- Missing webhooks no longer loop yandere posts.
- Fixed empty messages failing to send webhooks on logs.
- Typo on translate.
- Fixed broken skip after the bot leaves and comes back.
- All events now check if the guild wants that event logged.
- Removed (empty) spam from channel logs.
- Fixed deadlock on ping and about when being rate limited.
- nHentai no longer sends an inaccurate error on 404.

### Additions

- Added missing examples to the serenity commands.
- Switched logging to use webhooks!
- Added submitter and time left to now_playing.

## 0.3.0-alpha

### Bugs

- Removed A LOT of DeadLocks!
- Empty queue no longer sends an empty codeblock.
- Eval no longer deserializes to custom struct.
- Improved code quality some.

## 0.2.7-alpha

### Bugs

- Fixed everyone mention bugs.
- Fixed owners_only macro on teams.
- HTTP Server now doesn't use a clone of cache.
- Fixed BeatmapData to serialize good types.
- Fixed Error handling on twitch notifications.
- Updated RTFM to use the main repo.
- Properly handle messages that are too long for embed fields on logging.
- Logging messages cannot be deleted with reactions.

### Dependencies

- Updated Dependencies.

### Features
- Added Competing status.
- Added Rust eval command.
- Swaped Translate from Yandex to IBM
- Added cargo-make support.
- Play can now search with youtube-dl
- Moved TODO to Gitlab Boards.
- Added clear_queue command.
- Added basic logging for almost every event.

## 0.2.6-alpha

### Bugs
- Fixed notification loop retriggering on ready
- Replaced sqlx iterators with vectors to fix logging bug on sqlx #599.

### Commands
- about    Now it supports teams.
- chan / idol    Fixed incorrect error reported when a tag is missing.
- help    Added more information.
    Restructured the fields.
- play    Play time is no longer are twice as long.
- skip    Now it stops the current playing song if the queue is empty.
- recent    Increased limit from 10 to 50 on pagination.
- eval    Switched from discord.py to hikari.
    Added STDOUT, Return Value and Syntax Error handling.

### Logging
- Restructured file structure.
- Added message create logging.
- Added message update logging.

### Repository
- Added CI/CD

## 0.2.5-alpha

### Dependencies
- serenity    Updated to PR 905
- sqlx-cli    Updated to latest version.
- sqlx    Updated to latest version with new pool builder.
- serenity-lavalink    Updated to the new Serenity PR

### Commands
- ping    Added REST GET latency.
- help    Fixed order so it's more cozy.
- uwufy    New command.
    Translates messages to the UWU language.
- dictionary    Now it paginates definitions
- nhentai    Now it falls back to a previous page in case of failing to fetch.
- skip    Added more help to it.
- booru_commands    Now they ignore capitalization.
- rtfm / example    Updated to the new PR

### Utils
- parse_member() now doesn't have a mention exploit

### Database
- Added ON UPDATE CASCADE to streamer constraints.


## 0.2.4-alpha

### Commands
- recent    Now shows pp.
- permanent_ban    New command.
    Permanently bans a user froma guild.
- picture    Fixed help.
- invite    Updated the required permissions.
- play/skip    Added some aliases.
- configure guild    Can now toggle anti-spam

### Logging
- New system
- Added a simple anti-spam

### Dependencies
- Updated dependencies.
- Updated rust nightly to remove depricated code.

### Structure
- new utils: osu.rs
- new source file: logging.rs
- moved osu! mods module to crate::utils::osu

## 0.2.3-alpha

### Structure
- restructured checks.
- added Games group.
- re-enabled presence intent.
- mute commands now check for manage_roles permission.

### Commands
- TicTacToe    Rewritten to be more stable.
- Playlist    Now accepts playlists without video.
- queue    Now only shows a maximum of 10 items.
    And the total amount of items.
- todo    Updated items on the list.
- sauce    New command.
    Attempts to find the source of images.
- behoimi    Now it displays images correctly, rather than the big NO sausage.
- temporal_mute    New command.
    Temporarily mutes a member.
- self_temporal_mute    New command.
    Calls temporal_mute on yourself.

## 0.2.2-alpha

### Commands
- mute    New command.
    Gives the configured mute role to a member.
- selfmute    New command.
    Gives the configured mute role to the invoker.
- configure guild mute_role    New command.
    Configures the muted role on the guild.
- score    Now it works with loved maps.
- reminder    New command.
    Reminds you of a message after some specified time.

## 0.2.1-alpha

### Commands
- osu!top    New command.
    Shows the osu! top plays of a user.
- play    Fixed failed search.
- help    Now it works again in DMs.
- configure channel notifications    Twitch streams can now be removed.
    YandeRe can now be configured with channel.
    Twitch can now be configured with webhook.
    Modified the design of the command.

## 0.2.0-alpha

### Commands
- calculator    Now has a sqrt() function.

### Dependencies
- sqlx    Switched to master branch.
    NO MORE 4 HOUR COMPILE TIMES POGCHAMP!!!
- the rest    Updated.

### Database
- Migrations    Implemented migrations.

### Fixes
- web server    No longer runs every ready event.

## 0.1.19-alpha

### Commands
- profile    New command.
    Sends the profile of a user. (not member)
- higher_or_lower    Forgot to push the playing cards.
- calculator    New command.
    Just a calculator that takes a query.
- play    Now it retries in case of not finding a result.

### Codebase
- removed accidental debug code.

### Notifications
- twitch no longer fails when a streamer doesn't exist.

## 0.1.18-alpha

### Commands
- higher_or_lower    New command.
    Plays higher or lower.
- rtfm    Tried to do something, but failed hard at it lol.

### Web Server
- Added a web server.
- Added a system to know if the bot is in a guild.

## 0.1.17-alpha

### Commands
- pause/unapuse    New command.
    Pauses or unpauses the player, depending on status.
- configure channel notifications    Can now configure new streams.
- stop    No longer panics if there's no player.
- urban    Added several aliases.
- configure user streamrole    Fixed panic due to not having members in cache, due to missing pressence intents.
- example    Added keyword event to example 11.

### Loops
- disabled automatic disconnection from voice chat while i investigate how to fix it.

## 0.1.16-alpha

### Commands
- disable_command / enable_command    New commands.
    Disables or Enables a command on the guild respectively.
- shuffle    New command.
    Shuffles current queue.
- play_playlist    New command.
    Adds a whole playlist to the queue.
- play / play_playlist    Now they automatically join a voice channel when called if they are not in one already.
- define    Moved alias from urban to dictionary.
- example    New command.
    Sends you a link to the serenity examples.

### Dependencies
- Updated dependencies.

## 0.1.15-alpha

### Commands
- Dictionary    New command.
    Defines a word.
- ping    Added rest latency.
- about    Added additional information.
- help    Updated the order so it looks better and smaller.
- pride_pre_grayscale    New command.
    Just like pride, but grayscales beforehand.
    (Useful for dark images)

### Dependencies
- Updated serenity, no more cache deadlocks.
- Updated photon-rs, pride now works!

## 0.1.14-alpha

### Commands
- pride    Mostly fixed, it fails in some situations.
- recent    Fixed unknown message error when the message gets deleted.
- configure_osu    Fixed improper username management.
- osu_profile    Fixed pacman not doing well with not long enough decimal points.

### Notifications
- Twitch    No longer fails in nonexisting games.
    Updated to use the new authentication system.

### Boorus
- Heavily improved content safety.
- Blacklisted tagme.
- Notify about nonexisting tags.
- Added a link to the original post.


## 0.1.13-alpha

### Events
- Added custom intents.

### Commands
- play    Now it uses serenity-lavalink!
- help, tic_tac_toe, configure channel notifications    Fixed deadlock.

### Dependencies
- serenity    Updated, no longer has `&mut ctx`
- the rest    Also updated.


## 0.1.12-alpha

### Commands
- TicTacToe    New command.
    To play tic tac toe with other users.

### Bugs
- python bot no longer spams command not found errors.
- antispam not triggering on the correct messages if the bot can't manage messages.
- fixed 3 panics

## 0.1.11-alpha

### Commands
- nhentai    No longer runs in sfw channels.
- ban / kick    Now they can use reasons.
- recent / configure_osu    Removed possible mention exploits.
- encrypt / decrypt
    Changed to an aes algorythm.
    No longer broken.

## 0.1.10-alpha

### Commands
- all_boorus    Improved tag filter.
- e621    No longer broken on missing url fields.
- configure channel notifications    webhook yandere: fixed issue #832 on serenity.
- nHentai    New command.
    Allows you to read nhentai mangas from within discord.

## 0.1.9-alpha

### Commands

- configure user    streamrole: New command, gives you the bound notification role to a streamer.
- sankaku / idol    Imrpoved tag filtering.

### Notifications

- yandere    Improved tag filtering.

### Dependencies

- Updated all dependencies.

## 0.1.8-alpha

### Commands

- e621    Updated to the new api
- recent    Reactions are now exclusive to the command author.

### Adaptive commands

- pi    Added pi verification.
- nsfw    Added NSFW reaction to message jumps going to nsfw channels.
- Ping X in nsfw!
    Added easter egg.

### Voice

- Automatic disconnection in case of being alone.

## 0.1.7-alpha

### Commands

- all    Removed the default prefix from the examples.
- play    Fixed some url's obtaining the wrong video.
- translate    Fixed invalid language error being unhandled.
- recent    Reversed arrow directions.
- gay/pride    Prevented decompression bombs.

### Notifications

- Webhooks are optional.

## 0.1.6-alpha

### Notifications

- Added twitch notifications

### Design

- Status are now configurable.

## 0.1.5-alpha

### Commands

- play    Now it uses lavalink!
- translate    Now it uses yandex!
- eval    New command.
    Evaluates python code.
    Using discord.py

## 0.1.4-alpha

### Commands

- osu_profile    New command.
    Sends the osu! profile of a user.
- score    New command.
    Sends your score on a beatmap.
- play    New command.
    Plays the audio from a url.
- join/leave    New commands.
    Joins or Leaves voicechat respectively.

## 0.1.3-alpha

### Design

- Replaced tokio-postgres with SQLx

### Commands

- configure channel notifications    New command.
    Configures the notifications on the channel.

## 0.1.2-alpha

### Notifications

- yande.re    Added basic yande.re new post notifications.

### Commands


- booru_commands    Implemented the picture command, that sends a picture from the perffered booru from the user.
- all xml boorus    Now they are case insensitive.
- SafeBooru, FurryBooru, RealBooru    Reixed file extension...
- grayscale    Renamed pride to grayscale.

## 0.1.1-alpha

### Commands

- ban    Fixed api error.
- recent    Added pagination back.
- SafeBooru, FurryBooru, RealBooru    Fixed file extension.
- Danbooru    Fixed gold content.

## 0.1.0-alpha

# **ASYNC!**

## 0.0.21-alpha

### Commands

- encrypt/decrypt    New command, but broken.
- SafeBooru / RealBooru    Fixed recent api change.

### Debug

- Added basic logging.

### Optimizations

- Fixed linting

### Bugs

- Fixed command prefixes on booru commands.


## 0.0.20-alpha

### Commands

- duck_duck_go    New command.
    Searches a term on duckduckgo for you.
- reload_db    New command.
    Reloads the database connection. Owner only.
- invite    Updated for recent role mention changes.
- configure    New subcommand: Guild
    New subsubcommand of guild: prefix
    > Configures the prefix for the guild.

### Framework

- Implemented custom prefix.

## 0.0.19-alpha

### Commands

- configure    User subcommand    > Can now configure a preffered booru
- best_girl, best_boy    New command    Sends a picture of your best girl or best boy respectively.
    Configured with .configure
- chan    Fixed error handling for invalid requests.

## 0.0.18-alpha

### Commands

- configure    Moved parameters to subcommands.
    New subcommand: user.
    User subcommand    > Can now configure bestgirl and bestboy
- chan    New command.
- DanBooru, HypnoBooru    Fixed commands.
- e621    Moved to the list of broken boorus due to an api change.

### Optimizations

- idol    Url parameters are now parsed properly.
- boorus    Changed the source detection system.

## 0.0.17-alpha

### Commands

- about    Added server invite link.
- gray    Fixed image quality.
- changelog    New command.
    Sends this file.
- todo    Added new items to the list.
- clear    New command.
    Clears X messages.
- toggle_annoy    Moved command to config.
- configure    New command.
    Configures different aspects about the bot.
    Moved toggle_annoy to `.config channel annoy`

### Structure

- src/commands/configuration.rs
    New file.

## 0.0.16-alpha

### Commands

- prefixes    New command.
    Sends the configured prefixes of the guild.
- about    New command.
    Sends information about the bot.

## 0.0.15-alpha

### Structure

- src/commands/moderation.rs
    New file.

### Commands

- urban    Fixed error caused by definitions without examples.
- kick, ban    New commands.
    They Kick or Ban the specified user respectively.

## 0.0.14-alpha

### Structure

- translate.py
    New file.

### Commands

- urban    New command.
    Searches a term in Urban Dictionary.
- translate    New command.
    Translates text into a specific language.

## 0.0.13-alpha

### Structure

- src/commands/fun.rs
    New file.

### Commands

- qr    New command.
    Transforms text into an ASCII qr code.

## 0.0.12-alpha

### Structure

- boorus.json
    New file.
    Contains the availabe boorus and api type.

### Commands

- configure_osu    Fixed help.
- e621, furrybooru, realbooru, r34, safebooru, gelbooru, konachan, yandere    New booru commands.
    Same parameters as idol.
- HypnoHub, DanBooru and Behoimi    Broken commands.

## 0.0.11-alpha

### Commands

- toggle_annoy    Added a command to toggle the annoying features of the bot on the invoked channel.
    Made the annoying features of the bot optional per channel.

### Database

- annoyed_channels table    New table.

## 0.0.10-alpha

### Commands

- idol    Added tag support.
    Added flag support.
    Added basic tag filter.

### Documentation

- README.md    Updated configuration needed.

## 0.0.9-alpha

### Commands

- idol    Added a basic api call to the IdolComplex api.
    TODO: Custom tags and flags.

## 0.0.8-alpha

### Commands

- pride    Added a basic grayscaling command, named pride for future use.

### Dependencies

- image

### Structure

- commands/image_manipulation.rs    New file.

## 0.0.7-alpha

### Structure

- commands/meta.rs    New file.

### Commands

- Ping / Test    Moved to new meta.rs file.
- Source / Invite    New commands. One to get the gitlab link to the bot, the other to get the bot invite link.

## 0.0.6-alpha

### Commands

- recent    Paginated recent command, took a while monkaS
    \
    Added a description for the help command.
- configure_osu
    Moved description from decorator to docstring.

## 0.0.5-alpha

### Commands

- recent    Added recent command for osu!
- test    Made owner only.
- help    Updated embed colors.

### Dependencies

- bitflags
- num-format

## 0.0.4-alpha

### Commands

- configure_osu    Added command && Implemented basic database and argument reading.

### Dependencies

- toml
- postgres
- updated serenity features to allow compilation on the Raspberry PI 3B.

### Data

- Moved tokens from env_vars to tokens.toml

## 0.0.3-alpha

### Commands

- safebooru    Moved from the json api to xml.

### Modules

- utils    Implemented basic_functions module to have basic utility functions.

- commands    Renamed Cogs to Commands.

## 0.0.2-alpha

### Commands

- safebooru    Modified to send an embed for the image.

### Groups

- Changed the NSFW group to only have `test`

## 0.0.1-alpha

## Initial Release

Just a basic bot with Ping and a Safebooru command.
