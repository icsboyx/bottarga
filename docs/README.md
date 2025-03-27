## Table of Contents

1. [Overview](#overview)
2. [Software Components](#software-components)

## Overview

Bottarga is a multi-functional project that integrates various components to provide a robust framework for managing tasks, processing audio, handling bot commands, and interacting with external services like Twitch. It leverages asynchronous programming with [Tokio](https:#tokio.rs/) to ensure high performance and scalability.

---

## Software Components

### Source Files Documentation

#### `src/main.rs`

The main entry point of the application. It initializes the `TASKS_MANAGER` and adds tasks for various components like Twitch client, TTS, audio player, and bot commands. The tasks are then executed asynchronously.

- [Documentation](main.md)

#### `src/task_manager.rs`

Manages the lifecycle of tasks, including adding, retrying, and monitoring them. Provides an interface for scheduling asynchronous tasks with retry mechanisms.

- [Documentation](task_manager.md)

#### `src/audio_player.rs`

Handles audio playback functionality. Includes methods to play, stop, and manage audio streams. It also integrates with the TTS module to play generated audio.

- [Documentation](audio_player.md)

#### `src/bot_commands.rs`

Processes bot commands received from Twitch chat or external sources. Includes predefined commands and supports extending functionality through external configuration.

- [Documentation](bot_commands.md)

#### `src/irc_parser.rs`

Parses and processes IRC messages for real-time communication. It extracts relevant information from Twitch chat messages and passes them to the appropriate handlers.

- [Documentation](irc_parser.md)

#### `src/tts.rs`

Implements text-to-speech functionality. Converts text messages into audio streams using the configured voice settings. Supports customization of pitch, rate, and volume.

- [Documentation](tts.md)

#### `src/twitch_client.rs`

Interacts with Twitch APIs and services. Manages the connection to Twitch chat, handles incoming messages, and sends responses when required.

- [Documentation](twitch_client.md)

#### `src/common.rs`

Contains shared definitions, constants, and utility functions used across the project. Acts as a central location for reusable components and configurations.

- [Documentation](common.md)

---

<div style="text-align:center"><img src="assets/img/bottarga.png" /></div>

# Bottarga

Bottarga is a simle Text to Speech bot for Twitch chat.
It can read chat messages and convert them to the voice.
Can interact with Twitch Chat. And execute commands from chat.
Command are predefined and can be extended.

---

## Table of Contents

1. [Overview](#overview)
2. [Capabilities](#capabilities)
3. [Installation](#installation)
4. [Usage](#usage)
5. [Configuration](#configuration)
6. [Project Structure](#project-structure)
7. [Contributing](#contributing)
8. [License](#license)

---

## Overview

Bottarga is a multi-functional project that integrates various components to provide a robust framework for managing tasks, processing audio, handling bot commands, and interacting with external services like Twitch. It leverages asynchronous programming with [Tokio](https:#tokio.rs/) to ensure high performance and scalability.

---

## Capabilities

### 1. **Task Management**

- Add and manage asynchronous tasks.
- Execute tasks with retry mechanisms.
- List and monitor running tasks.

### 2. **Audio Playback**

- Play, stop, and manage audio streams.
- Notify and handle audio playback events.

### 3. **Twitch Client**

- Interact with Twitch APIs and services for streaming and chat integration.

### 4. **IRC Parsing**

- Parse and process IRC messages for real-time communication.

### 5. **Text-to-Speech (TTS)**

- Generate and play TTS audio streams.

### 6. **Bot Commands**

- Process and execute bot commands for automation.
- Available commands:
  - **`!help`**: Show help message and list all commands.
  - **`!list_locales`**: Show list of supported locales for TTS.
  - **`!reset_voice`**:
    > Reset voice to random if no arguments are provided.
    > If arguments are provided, a text search is applied on VoiceDB to find a voice.
    > In case of multiple results, a random one is selected from the search results.
  - **`!stop`**: Stop Audio playing.
  - **`!die`**: temporary command to test command service restart (in this case only on bot commands task).

### 7. **External Bot Commands**

- Process and execute external bot commands for automation, loaded from external config file.

---

## Installation

To set up Bottarga, follow these steps:

1. Clone the repository:
   ```bash
   git clone https:#github.com/icsboyx/bottarga.git
   ```


2. Navigate to the project directory:
   ```bash
   cd bottarga
   ```
3. Build the project using Cargo:
   ```bash
   cargo build --release
   ```

---

## Usage

### Running the Application

To run the application, use the following command:

```bash
cargo run
```

### Example Workflow

The main entry point of the application is defined in `src/main.rs`. The `TASKS_MANAGER` is used to add and manage tasks:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TASKS_MANAGER
        .add("TWITCH_CLIENT", || Box::pin(twitch_client::start()), 3)
        .await;

    TASKS_MANAGER.add("TTS", || Box::pin(tts::start()), 3).await;

    TASKS_MANAGER
        .add("AUDIO_PLAYER", || Box::pin(audio_player::start()), 3)
        .await;

    TASKS_MANAGER
        .add("BOT_COMMANDS", || Box::pin(bot_commands::start()), 3)
        .await;

    TASKS_MANAGER.run_tasks().await;

    Ok(())
}
```

### Adding a Task

To add a new task, use the `TASKS_MANAGER.add` method:

```rust
TASKS_MANAGER.add("TaskName", || Box::pin(async_task_function()), retries).await;
```

---

## Configuration

The project uses a configuration directory to store persistent data. The default configuration directory is `.config`, as defined in `src/main.rs`:

```rust
pub static CONFIG_DIR: Option<&'static str> = Some(".config");
```

### Configuration Files

When the application is run for the first time, configuration files will be automatically generated in the `.config` directory if they do not already exist. These files include default settings for various modules, such as task management, audio playback, and Twitch integration.

To customize the configuration, edit the files in the `.config` directory. For example:

### Default Configuration:

`TwitchConfig.toml`\
`BotSpeechConfig.toml`\
`UsersDB.toml`

The `TwitchConfig.toml` file contains default settings for the Twitch integration module. Below is an example of the default configuration:
The default config is for anonymous login to twitch chat. Bot can only read chat messages and convert to the voice. With Anonymous login, bot can't send messages to chat.

```toml
# The IRC server for Twitch chat.
server = "irc.chat.twitch.tv"
# Anonymous login nickname for Twitch chat.
nick = "justinfan69696942" 
# The Twitch channel to connect to.
channel = "icsboyx" 
# Authentication token for Twitch chat (use anonymous login token for read-only access, random string).
auth_token = "1234567890"
# Request Twitch-specific IRC tags capabilities. 
# Request Twitch-specific IRC membership capabilities.
# Request Twitch-specific IRC command capabilities.
irc_cap_req = [
    "twitch.tv/commands", 
    "twitch.tv/membership",
    "twitch.tv/tags" 
]
# Interval in seconds to send PING messages to keep the connection alive.
ping_interval = 180 
```

The `BotSpeechConfig.toml` file contains default settings for the Bot Voice.

```toml
[speech_config]
# Name of the voice to use for text-to-speech
# See the list in .config/VoiceDB.toml it static list of supported voices updated on each start
voice_name = "Microsoft Server Speech Text to Speech Voice (it-IT, GiuseppeMultilingualNeural)"

# Audio format for the generated TTS audio
audio_format = "audio-24khz-48kbitrate-mono-mp3"

# Pitch of the generated audio signed integer higher is higher lower is lower 0 is default
pitch = 0

# Rate of the generated audio signed integer higher is faster lower is slower 0 is default
rate = 0

# Volume of the generated audio signed integer higher is louder lower is quieter 0 is default
volume = 0
```

The `UsersDB.toml` file contains Users Database.

```toml
[users.icsboyx]
nick = "icsboyx"

[users.icsboyx.speech_config]
voice_name = "Microsoft Server Speech Text to Speech Voice (it-IT, DiegoNeural)"
audio_format = "audio-24khz-48kbitrate-mono-mp3"
pitch = 0
rate = 0
volume = 0
```

The `ExternalBotCommands.toml` file contains External Bot Commands.\

- {ARGS} and {SENDER} variables can be used in the replay_text field.
  - {ARGS} => This is the argument passed to the command.
  - {SENDER} => This is the sender/requester of the command.
- custom_audio_url => if valid audio file, will always be reproduced before text reply.

```toml
[commands.test]
# Command name and activation pattern
activation_pattern = "test"
# Indicates if the command requires an argument
need_arg = false
# Custom audio URL or local file to play before the text reply
custom_audio_url = ""
# Text reply
replay_text = "Hi there {SENDER} this is the reply to your test command"

[commands.for_president]
activation_pattern = "for_president"
need_arg = true
custom_audio_url = ""
replay_text = "{ARG} for President!"

[commands.meow]
activation_pattern = "meow"
need_arg = false
custom_audio_url = "https://www.myinstants.com/media/sounds/m-e-o-w.mp3"
replay_text = ""
```

Ensure that the `.config` directory is writable by the application to allow for proper initialization and updates. In case of error loading the configuration, the application will fall back to default settings and run in memory mode. No persistent data will be saved.

## Project Structure

The project is organized into the following modules:

- **`src/main.rs`**: Entry point of the application.
- **`src/task_manager.rs`**: Manages tasks and their execution.
- **`src/audio_player.rs`**: Implements audio playback and control.
- **`src/bot_commands.rs`**: Processes bot commands for automation.
- **`src/irc_parser.rs`**: Parses and processes IRC messages.
- **`src/tts.rs`**: Handles text-to-speech functionality.
- **`src/twitch_client.rs`**: Interacts with Twitch APIs and services.
- **`src/defs.rs`**: Contains shared definitions and utilities.

---

## Contributing

We welcome contributions to Bottarga!

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.

## Additional Resources

- [Rust Documentation](https:#doc.rust-lang.org/)
- [Tokio Documentation](https:#tokio.rs/)
- [GitHub Repository](https:#github.com/icsboyx/bottarga)
````
