# Bottarga Project Documentation

## Overview

The `bottarga` project is a Rust-based application designed to manage tasks and integrate with various services such as Twitch, TTS (Text-to-Speech), and audio playback. This document provides an overview of the main components and their functionality.

---

## File Structure

### Main File: `main.rs`

The `main.rs` file serves as the entry point for the application. It initializes and starts various components of the project.

### Modules

The project is organized into the following modules:

- **`audio_player`**: Handles audio playback functionality.
- **`bot_commands`**: Manages bot commands.
- **`bot_external_commands`**: Provides external bot command integration.
- **`irc_parser`**: Parses IRC messages.
- **`task_manager`**: Manages tasks and their execution.
- **`task_stats`**: Tracks task statistics.
- **`tts`**: Handles Text-to-Speech functionality.
- **`twitch_client`**: Manages Twitch client interactions.
- **`users`**: Handles user-related functionality.

---

## Key Features

1. **Task Management**  
   The `TASKS_MANAGER` is responsible for managing and running tasks. Tasks are added with a name, a function, and a priority level.

2. **Twitch Integration**  
   The `twitch_client` module provides functionality to interact with Twitch services.

3. **Text-to-Speech (TTS)**  
   The `tts` module enables TTS capabilities.

4. **Audio Playback**  
   The `audio_player` module manages audio playback.

5. **Bot Commands**  
   The `bot_commands` module handles bot command execution.

6. **External Commands**  
   The `bot_external_commands` module allows integration with external commands.

---

## Initialization Flow

1. **Task Initialization**  
   Tasks such as `Twitch Client`, `TTS`, `Audio Player`, and `Bot Commands` are added to the `TASKS_MANAGER`.

2. **External Commands**  
   The `ExternalBotCommands::init()` function initializes external bot commands.

3. **Task Execution**  
   The `TASKS_MANAGER.run_tasks()` function starts all tasks.

---

## Configuration

The `CONFIG_DIR` variable specifies the configuration directory for the application. By default, it is set to `.config`.

---

## Logging

The application uses logging for debugging and tracing. For example:

```rust
log_trace!("Test External Bot Command {:?}", ret_val);
```

---

## Running the Application

To run the application, use the following command:

```bash
cargo run
```

Ensure all dependencies are installed and configured correctly.

---

## Future Enhancements

- Add more robust error handling.
- Improve modularity and scalability.
- Enhance logging and monitoring capabilities.

---

For more details, refer to the source code in `main.rs` and the respective modules.
