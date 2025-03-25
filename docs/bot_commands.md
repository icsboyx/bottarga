# Bot Commands Documentation

This document provides an overview of the bot commands available in the application.

## Command Prefix

All bot commands must be prefixed with `!`. This prefix is defined in the code as a global variable: `BOT_COMMAND_PREFIX`.

## Available Commands

### `!help`

- **Description**: Lists all available bot commands.
- **Usage**: `!help`
- **Behavior**: The bot will respond with a list of all registered commands by dynamically retrieving them from the internal command registry.

### `!die`

- **Description**: Shuts down the bot with a farewell message.
- **Usage**: `!die`
- **Behavior**: Sends a farewell message to the chat and terminates the bot's process. _(Note: This command is commented out in the code and may not be active.)_

## Adding New Commands

To add a new command, use the `add_command` method in the `BotCommands` struct. Example:

```rust
BOT_COMMANDS
    .add_command("new_command", Arc::new(|irc_message| Box::pin(new_command_function(irc_message))))
    .await;
```

Replace `new_command` with the desired trigger and `new_command_function` with the function to execute.

## Command Execution Flow

1. **Trigger Detection**:  
   The bot listens for messages starting with the prefix `!`. If a message starts with the prefix, the bot extracts the command trigger.

2. **Command Lookup**:  
   The extracted trigger is matched against the registered commands in the `BotCommands` registry.

3. **Command Execution**:  
   If a matching command is found, the associated function is executed asynchronously.

## Key Components

### `BotCommands`

The `BotCommands` struct manages the registration and execution of bot commands.

#### Methods

- **`add_command(&self, trigger: impl Into<String>, command: BotCommandType)`**  
  Registers a new command with the given trigger and function.

- **`run_command(&self, command: &str, message: IrcMessage) -> Result<()>`**  
  Executes the command associated with the given trigger.

### `list_all_commands`

- **Description**: A function that lists all registered commands.
- **Details**: It retrieves all command triggers from the `BotCommands` registry and formats them for display.

### `die`

- **Description**: A function that shuts down the bot with a farewell message.
- **Details**: Sends a farewell message to the chat and terminates the bot's process. _(Note: This function is commented out in the code.)_

## Notes

- Commands are case-sensitive.
- Ensure the bot is running and connected to the chat.
- The bot uses asynchronous programming to handle commands efficiently.
- Developers can dynamically add or modify commands at runtime using the `add_command` method.
