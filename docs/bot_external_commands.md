# External Bot Commands

This module provides functionality for managing and executing external bot commands in the Twitch bot. It allows users to define custom commands with specific activation patterns, optional arguments, and custom audio responses.

## Key Structures

### `ExternalBotCommand`

Represents a single external bot command with the following fields:

- `activation_pattern` (String): The command trigger word.
- `need_arg` (bool): Indicates if the command requires an argument.
- `custom_audio_url` (String): URL for custom audio to play when the command is triggered.
- `replay_text` (String): Text response template. Supports placeholders like `{SENDER}` and `{ARG}`.

### `ExternalBotCommands`

A collection of `ExternalBotCommand` objects stored in a `HashMap`. Implements:

- `PersistentConfig`: For loading and saving configuration.
- `Default`: Provides default commands (`test`, `meow`, `for_president`).

## Key Functions

### `ExternalBotCommands::init()`

Initializes the external bot commands by loading them from the configuration directory.

### `ExternalBotCommands::reg_ext_bot_cmd()`

Registers all external bot commands with the bot's command handler.

### `handle_command(irc_message: IrcMessage, command: ExternalBotCommand)`

Handles the execution of a command when triggered. It:

1. Replaces placeholders in the `replay_text` with actual values.
2. Plays custom audio if `custom_audio_url` is provided.
3. Sends the response text to the Twitch chat.

### `get_audio_data(url: impl AsRef<str>) -> Vec<u8>`

Fetches audio data from a given URL for playback.

## Default Commands

1. **`test`**: Replies with "Hi there {SENDER} this is the reply to your test command".
2. **`meow`**: Plays a "meow" sound from a predefined URL.
3. **`for_president`**: Requires an argument and replies with "{ARG} for President!".

## Usage

1. Define commands in the `ExternalBotCommands` structure.
2. Use `ExternalBotCommands::init()` to load commands from the configuration file.
3. Call `ExternalBotCommands::reg_ext_bot_cmd()` to register commands with the bot.

## Example

To add a new command, update the `ExternalBotCommands.toml` file in the configuration directory as follows:

```toml
# filepath: .config/ExternalBotCommands.toml

[commands.hello]
activation_pattern = "hello"
need_arg = false
custom_audio_url = ""
replay_text = "Hello, {SENDER}!"

[commands.greet]
activation_pattern = "greet"
need_arg = true
custom_audio_url = "https://example.com/greet.mp3"
replay_text = "Greetings, {ARG}!"
```

This will create two commands:

1. `hello`: Responds with "Hello, {SENDER}!" when triggered.
2. `greet`: Requires an argument and responds with "Greetings, {ARG}!" while playing the specified audio.
