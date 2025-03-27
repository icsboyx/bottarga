# Twitch Client Module Documentation

This module implements a Twitch client for interacting with Twitch's IRC-based chat system. It provides functionality for connecting to Twitch, handling messages, and broadcasting messages to other parts of the application.

## Key Components

### 1. **Static Variables**

- `TWITCH_BOT_INFO`: Stores bot-related information such as nickname, channel, and speech configuration.
- `TWITCH_BROADCAST`: A broadcast channel for sending IRC messages to other parts of the application.
- `TWITCH_RECEIVER`: A message queue for sending messages to Twitch.

### 2. **Traits**

- `IntoIrcPRIVMSG`: Converts a message into an IRC `PRIVMSG` format.
- `WsMessageHandler`: Converts a message into a WebSocket text message.

### 3. **Structures**

- `BotSpeechConfig`: Manages speech configuration for text-to-speech (TTS) functionality.
- `TwitchBotInfo`: Stores and manages bot-related information such as nickname and channel.
- `TwitchConfig`: Stores configuration details for connecting to Twitch, such as server, nickname, and authentication token.

### 4. **Functions**

- `start()`: The main entry point for the Twitch client. Handles WebSocket communication, message processing, and periodic pinging.
- `twitch_auth()`: Authenticates the bot with Twitch using the provided configuration.
- `split_lines()`: Splits a long message into multiple lines, ensuring each line adheres to the maximum length.
- `handle_twitch_msg()`: Processes incoming Twitch messages and handles commands like `PING`, `PRIVMSG`, and `JOIN`.

## Workflow

1. **Initialization**:

   - The bot initializes its configuration (`TwitchConfig`) and speech settings (`BotSpeechConfig`).
   - Static variables like `TWITCH_BOT_INFO` and `TWITCH_BROADCAST` are set up.

2. **Connection**:

   - The `start()` function establishes a WebSocket connection to Twitch's IRC server.
   - The bot authenticates itself using `twitch_auth()`.

3. **Message Handling**:

   - Incoming messages are processed by `handle_twitch_msg()`.
   - Commands like `PING`, `PRIVMSG`, and `JOIN` are handled appropriately.

4. **Broadcasting**:

   - Messages are broadcasted to other parts of the application using `TWITCH_BROADCAST`.

5. **Text-to-Speech**:
   - Messages not starting with the bot command prefix are sent to the TTS queue for speech synthesis.

## Configuration

The module uses two main configuration structures:

- `TwitchConfig`: Stores Twitch connection details.
- `BotSpeechConfig`: Manages TTS-related settings.

Both configurations implement the `PersistentConfig` trait, allowing them to be loaded and saved persistently.

## Error Handling

The module uses the `anyhow` crate for error handling. Errors are logged and propagated as needed.

## Example Usage

To start the Twitch client:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    twitch_client::start().await
}
```

## Notes

- The bot uses `tokio-tungstenite` for WebSocket communication.
- The maximum message line length is defined by `TWITCH_MAX_MSG_LINE_LENGTH`.
- The bot's nickname and channel are dynamically updated based on server responses.

## Future Improvements

- Add support for additional Twitch IRC commands.
- Improve error handling and reconnection logic.
- Enhance TTS functionality with more voice options.
