# Bot Commands Documentation

This document provides an overview of the bot commands available in the application.

## Command Prefix

All bot commands must be prefixed with `!`.

## Available Commands

### `!help`

- **Description**: Lists all available bot commands.
- **Usage**: `!help`
- **Behavior**: The bot will respond with a list of all available commands.

### `!die`

- **Description**: Terminates the bot with a farewell message.
- **Usage**: `!die`
- **Behavior**: The bot will announce its termination and stop functioning.

## Adding New Commands

To add a new command, use the `add_command` method in the `BotCommands` struct. Example:

```rust
BOT_COMMANDS
    .add_command("new_command", Arc::new(|irc_message| Box::pin(new_command_function(irc_message))))
    .await;
```

Replace `new_command` with the desired trigger and `new_command_function` with the function to execute.

## Command Execution Flow

1. Commands are triggered by messages starting with the prefix `!`.
2. The bot extracts the command and executes the corresponding function if it exists.

## Notes

- The selection feature is experimental and may be removed in the future.

For more details, refer to the source code in `src/bot_commands.rs`.
