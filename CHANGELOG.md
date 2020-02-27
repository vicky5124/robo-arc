# 0.0.10

### Commands
- idol:
  Added tag support.
  Added flag support.
  Added basic tag filter.

### Documentation
- README.md:
  Updated configuration needed.

# 0.0.9

### Commands
- idol:
  Added a basic api call to the IdolComplex api.
  TODO: Custom tags and flags.

# 0.0.8

### Commands
- pride:
  Added a basic grayscaling command, named pride for future use.

### Dependencies
- image

### Structure
- commands/image_manipulation.rs:
  New file.

# 0.0.7

### Structure
- commands/meta.rs:
  New file.

### Commands
- Ping / Test:
  Moved to new meta.rs file.
- Source / Invite:
  New commands. One to get the gitlab link to the bot, the other to get the bot invite link.

# 0.0.6

### Commands
- recent:
  Paginated recent command, took a while monkaS
  \
  Added a description for the help command.
- configure_osu
  Moved description from decorator to docstring.

# 0.0.5

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

# 0.0.4

### Commands
- configure_osu:
  Added command && Implemented basic database and argument reading.

### Dependencies
- toml
- postgres
- updated serenity features to allow compilation on the Raspberry PI 3B.

### Data
- Moved tokens from env_vars to tokens.toml

# 0.0.3

### Commands
- safebooru:
  Moved from the json api to xml.

### Modules
- utils:
  Implemented basic_functions module to have basic utility functions.

- commands:
  Renamed Cogs to Commands.

# 0.0.2

### Commands
- safebooru:
  Modified to send an embed for the image.

### Groups
- Changed the NSFW group to only have `test`

# 0.0.1 Initial Release

Just a basic bot with Ping and a Safebooru command.
