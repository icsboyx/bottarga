
# Bottarga

Bottarga is a modular and extensible Rust-based project designed to handle asynchronous tasks, audio playback, bot commands, and more. It is built with a focus on efficiency, scalability, and ease of use.

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

Bottarga is a multi-functional project that integrates various components to provide a robust framework for managing tasks, processing audio, handling bot commands, and interacting with external services like Twitch. It leverages asynchronous programming with [Tokio](https://tokio.rs/) to ensure high performance and scalability.

---

## Capabilities

### 1. **Task Management**
   - Add and manage asynchronous tasks.
   - Execute tasks with retry mechanisms.
   - List and monitor running tasks.

### 2. **Audio Playback**
   - Play, stop, and manage audio streams.
   - Notify and handle audio playback events.

### 3. **Bot Commands**
   - Process and execute bot commands for automation.

### 4. **IRC Parsing**
   - Parse and process IRC messages for real-time communication.

### 5. **Text-to-Speech (TTS)**
   - Generate and play TTS audio streams.

### 6. **Twitch Client**
   - Interact with Twitch APIs and services for streaming and chat integration.

---

## Installation

To set up Bottarga, follow these steps:

1. Clone the repository:
   ```bash
   git clone https://github.com/icsboyx/bottarga.git
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
        .add("Task01", || Box::pin(twitch_client::start()), 3)
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
server = "irc.chat.twitch.tv"
nick = "justinfan69696942"
channel = "icsboyx"
auth_token = "1234567890"
irc_cap_req = [
    "twitch.tv/commands",
    "twitch.tv/commands",
    "twitch.tv/tags",
]
ping_interval = 180
```


The `BotSpeechConfig.toml` file contains default settings for the Bot Voice. 

```toml
[speech_config]
// Name of the voice to use for text-to-speech
// See the list in .config/VoiceDB.toml it static list of supported voices updated on each start
voice_name = "Microsoft Server Speech Text to Speech Voice (it-IT, GiuseppeMultilingualNeural)"

// Audio format for the generated TTS audio
audio_format = "audio-24khz-48kbitrate-mono-mp3"

// Pitch of the generated audio signed integer higher is higher lower is lower 0 is default
pitch = 0

// Rate of the generated audio signed integer higher is faster lower is slower 0 is default
rate = 0

// Volume of the generated audio signed integer higher is louder lower is quieter 0 is default
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



## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Tokio Documentation](https://tokio.rs/)
- [GitHub Repository](https://github.com/icsboyx/bottarga)
