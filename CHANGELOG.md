# 0.0.16-alpha

### Commands
- prefixes:
  New command.
  Sends the configured prefixes of the guild.
- about:
  New command.
  Sends information about the bot.

# 0.0.15-alpha

### Structure
- src/commands/moderation.rs
  New file.

### Commands
- urban:
  Fixed error caused by definitions without examples.
- kick, ban:
  New commands.
  They Kick or Ban the specified user respectively.

# 0.0.14-alpha

### Structure
- translate.py
  New file.

### Commands
- urban:
  New command.
  Searches a term in Urban Dictionary.
- translate:
  New command.
  Translates text into a specific language.

# 0.0.13-alpha

### Structure
- src/commands/fun.rs
  New file.

### Commands
- qr:
  New command.
  Transforms text into an ASCII qr code.

# 0.0.12-alpha

### Structure
- boorus.json
  New file.
  Contains the availabe boorus and api type.

### Commands
- configure_osu:
  Fixed help.
- e621, furrybooru, realbooru, r34, safebooru, gelbooru, konachan, yandere:
  New booru commands.
  Same parameters as idol.
- HypnoHub, DanBooru and Behoimi:
  Broken commands.

# 0.0.11-alpha

### Commands
- toggle_annoy:
  Added a command to toggle the annoying features of the bot on the invoked channel.
  Made the annoying features of the bot optional per channel.

### Database
- annoyed_channels table:
  New table.

# 0.0.10-alpha

### Commands
- idol:
  Added tag support.
  Added flag support.
  Added basic tag filter.

### Documentation
- README.md:
  Updated configuration needed.

# 0.0.9-alpha

### Commands
- idol:
  Added a basic api call to the IdolComplex api.
  TODO: Custom tags and flags.

# 0.0.8-alpha

### Commands
- pride:
  Added a basic grayscaling command, named pride for future use.

### Dependencies
- image

### Structure
- commands/image_manipulation.rs:
  New file.

# 0.0.7-alpha

### Structure
- commands/meta.rs:
  New file.

### Commands
- Ping / Test:
  Moved to new meta.rs file.
- Source / Invite:
  New commands. One to get the gitlab link to the bot, the other to get the bot invite link.

# 0.0.6-alpha

### Commands
- recent:
  Paginated recent command, took a while monkaS
  \
  Added a description for the help command.
- configure_osu
  Moved description from decorator to docstring.

# 0.0.5-alpha

### Commands
- recent:
  Added recent command for osu!
- test:
  Made owner only.
- help:
  Updated embed colors.

### Dependencies
- bitflags
- num-format

# 0.0.4-alpha

### Commands
- configure_osu:
  Added command && Implemented basic database and argument reading.

### Dependencies
- toml
- postgres
- updated serenity features to allow compilation on the Raspberry PI 3B.

### Data
- Moved tokens from env_vars to tokens.toml

# 0.0.3-alpha

### Commands
- safebooru:
  Moved from the json api to xml.

### Modules
- utils:
  Implemented basic_functions module to have basic utility functions.

- commands:
  Renamed Cogs to Commands.

# 0.0.2-alpha

### Commands
- safebooru:
  Modified to send an embed for the image.

### Groups
- Changed the NSFW group to only have `test`

# 0.0.1-alpha
##Initial Release

Just a basic bot with Ping and a Safebooru command.
