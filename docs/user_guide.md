# Bottarga User Guide

This guide is for people who want to run and use Bottarga, not change its code.

Bottarga listens to Twitch chat, reads messages aloud, and lets chat users run simple commands.

## What Bottarga Does

- Reads Twitch chat messages aloud.
- Lets users change their own TTS voice.
- Lets moderators or chat users stop current audio playback with a command.
- Sends command replies back to Twitch chat.
- Supports custom commands from a configuration file.
- Can play custom audio clips for external commands.

## Before You Start

You need:

- Rust installed.
- A Twitch account for the bot token.
- Permission to authorize the Twitch account with the scopes Bottarga requests.
- Working audio output on the machine running Bottarga.
- Internet access to Twitch and Microsoft Edge TTS services.

On Linux, you can optionally configure a PulseAudio sink if you want Bottarga audio routed to a specific output device.

## First Run

Build and run Bottarga:

```bash
cargo run --release
```

The first run creates the `.config` directory and asks for Twitch setup information.

### 1. Authorize Twitch

Bottarga prints a Twitch authorization URL.

Open that URL in a browser, authorize the app, and copy the full redirected URL. Paste it back into the terminal when Bottarga asks for it.

Bottarga stores the validated token in:

```text
.config/TwitchToken.toml
```

Bottarga also creates:

```text
.config/TwitchScopesConfig.toml
```

This file controls which OAuth scopes are requested during Twitch authorization. It is generated automatically like the other `.config` files, and its defaults match the bot's built-in features.

Example:

```toml
scopes = [
  "channel:moderate",
  "channel:bot",
  "user:bot",
  "user:edit:broadcast",
  "user:read:email",
  "user:read:emotes",
  "user:read:follows",
  "user:read:subscriptions",
  "user:read:chat",
  "user:write:chat",
  "user:manage:whispers",
]
```

### 2. Choose the Twitch Channel

Bottarga asks for the Twitch channel name to join.

Enter the channel login, for example:

```text
my_channel_name
```

Bottarga resolves and stores the channel in:

```text
.config/StreamerChannel.toml
```

## Daily Use

After the first setup, start Bottarga with:

```bash
cargo run --release
```

When the bot is running:

- normal chat messages are read aloud
- commands starting with `!` are handled as bot commands
- command replies are sent back to Twitch chat

## Chat Commands

### `!help`

Shows the list of available commands.

### `!list_locales`

Shows available TTS locales.

Use this when users want to discover voice language codes.

### `!reset_voice <filter>`

Changes the sender's TTS voice.

Examples:

```text
!reset_voice it-IT
!reset_voice female
!reset_voice multilingual
!reset_voice en-US neural
```

Bottarga searches the voice database using all words after `!reset_voice`. If multiple voices match, it picks one randomly.

### `!stop`

Stops current audio playback.

## Custom Commands

Custom commands are configured in:

```text
.config/ExternalBotCommands.toml
```

Example:

```toml
[commands.hello]
activation_pattern = "hello"
aliases = ["hi"]
need_arg = false
custom_audio_url = ""
replay_text = "Hello, {SENDER}!"

[commands.shout]
activation_pattern = "shout"
aliases = []
need_arg = true
custom_audio_url = ""
replay_text = "{ARG}!"
```

This creates:

```text
!hello
!hi
!shout something here
```

### Custom Command Fields

- `activation_pattern`: the command name without `!`.
- `aliases`: optional alternative command names.
- `need_arg`: set to `true` when the command needs text after the trigger.
- `custom_audio_url`: optional audio file URL to play.
- `replay_text`: text Bottarga sends back to chat and reads aloud.

### Placeholders

- `{SENDER}` is replaced with the Twitch user who ran the command.
- `{ARG}` is replaced with the text after the command.

## Voice Configuration

Bottarga stores user voices in:

```text
.config/UsersDB.toml
```

New users get a default voice selected with:

```text
.config/UserDefaultVoiceConfig.toml
```

The generated voice catalog is:

```text
.config/VoiceDB.toml
```

You can inspect `VoiceDB.toml` to find voice names, locales, and metadata that work well with `!reset_voice`.

## Audio Configuration

Audio settings are stored in:

```text
.config/AudioControl.toml
```

The main fields are:

- `volume`: playback volume.
- `linux_sink_name`: optional PulseAudio sink name on Linux.

If `linux_sink_name` is empty, Bottarga uses the default Kira playback path.

## Files You Usually Edit

Most users only need these files:

- `.config/ExternalBotCommands.toml`
- `.config/UserDefaultVoiceConfig.toml`
- `.config/AudioControl.toml`

These files are generated or managed by Bottarga and usually should not be edited manually unless you know what you are changing:

- `.config/TwitchToken.toml`
- `.config/TwitchScopesConfig.toml`
- `.config/StreamerChannel.toml`
- `.config/UsersDB.toml`
- `.config/VoiceDB.toml`

Edit `TwitchScopesConfig.toml` only when you intentionally want to change the Twitch OAuth permissions requested by the bot. After changing scopes, delete or move `TwitchToken.toml` and authorize again.

## Troubleshooting

### The Bot Asks For Twitch Authorization Again

The stored token is missing, expired, invalid, or does not contain all scopes listed in `TwitchScopesConfig.toml`.

Authorize again using the URL printed by Bottarga.

### The Bot Does Not Reply In Chat

Check that the Twitch token has the required chat write scope. If in doubt, delete or move `.config/TwitchToken.toml` and authorize again.

### Chat Is Not Being Read Aloud

Check that:

- Bottarga is connected to the right channel.
- Audio output works on the machine.
- Microsoft Edge TTS can connect to the network.
- The audio queue is not stopped by a previous `!stop` command.

### Custom Audio Does Not Play

Check that:

- `custom_audio_url` is not empty.
- The URL is reachable from the machine running Bottarga.
- The URL points to audio data that the playback path can decode.

### The Wrong Channel Is Used

Edit or delete:

```text
.config/StreamerChannel.toml
```

Then restart Bottarga and enter the correct Twitch channel name.

### The Bot Uses A Bad Voice

Use:

```text
!reset_voice <filter>
```

or adjust the default filter in:

```text
.config/UserDefaultVoiceConfig.toml
```
