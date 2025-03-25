# Twitch Client Module

This module implements a Twitch client for interacting with Twitch's IRC-based chat system. It includes functionality for connecting to Twitch, handling messages, and managing bot configurations.

## Key Components

### 1. **Static Variables**

- `TWITCH_BOT_INFO`: Stores bot information such as nickname and channel.
- `TWITCH_BROADCAST`: A broadcast channel for sending IRC messages.
- `TWITCH_RECEIVER`: A message queue for outgoing messages.
- `TWITCH_MAX_MSG_LINE_LENGTH`: Maximum length of a single message line.

### 2. **Traits**

- `IntoIrcPRIVMSG`: Converts a message into IRC `PRIVMSG` format.
- `WsMessageHandler`: Converts a message into WebSocket text format.

### 3. **Structures**

#### `BotSpeechConfig`

- Stores speech configuration for text-to-speech (TTS).
- Implements `Default` and `PersistentConfig` traits for loading and saving configurations.

#### `TwitchBotInfo`

- Manages bot nickname, channel, and speech configuration.
- Provides asynchronous methods to get and set these properties.

#### `TwitchConfig`

- Stores Twitch connection settings such as server, nickname, channel, authentication token, and capabilities.
- Implements `Default` and `PersistentConfig` traits.

### 4. **Functions**

#### `start()`

- Initializes the Twitch client and starts the WebSocket connection.
- Handles incoming and outgoing messages, including PING/PONG for connection health.

#### `twitch_auth(config: &TwitchConfig)`

- Authenticates the bot with Twitch using the provided configuration.

#### `split_lines(message: impl AsRef<str>)`

- Splits a long message into multiple lines, adhering to the maximum line length.

#### `handle_twitch_msg(text: impl AsRef<str>)`

- Processes incoming Twitch messages and handles commands like `PING`, `PRIVMSG`, and `JOIN`.

## Usage

1. **Initialization**

   - Load configurations using `TwitchConfig::load()` and `BotSpeechConfig::load()`.

2. **Starting the Client**

   - Call the `start()` function to begin the WebSocket connection and handle Twitch messages.

3. **Customizing Configurations**
   - Modify `TwitchConfig` or `BotSpeechConfig` to customize the bot's behavior.

## Example

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    twitch_client::start().await?;
    Ok(())
}
```

## Default Configuration

The Twitch client uses a configuration file in TOML format. Below is the default configuration with explanations for each field:

```toml
# filepath: .config/twitch_config.toml
server = "irc.chat.twitch.tv" # The Twitch IRC server address
nick = "justinfan69696942" # Default bot nickname (anonymous user)
channel = "icsboyx" # Default Twitch channel to join
auth_token = "1234567890" # Default OAuth token (replace with a valid token)
irc_cap_req = ["twitch.tv/commands", "twitch.tv/membership", "twitch.tv/tags"] # Requested IRC capabilities
ping_interval = 180 # Interval (in seconds) for sending PING messages to keep the connection alive
```

### Explanation of Fields

- `server`: Specifies the Twitch IRC server address. The default is `irc.chat.twitch.tv`.
- `nick`: The bot's nickname. By default, it uses an anonymous user nickname.
- `channel`: The Twitch channel the bot will join. Replace with your desired channel name.
- `auth_token`: The OAuth token for authenticating with Twitch. Replace this with a valid token.
- `irc_cap_req`: A list of IRC capabilities to request. These enable additional features like commands, membership, and tags.
- `ping_interval`: The interval in seconds for sending PING messages to maintain the connection.

## Dependencies

- `tokio`: For asynchronous runtime.
- `tokio_tungstenite`: For WebSocket communication.
- `serde`: For serialization and deserialization.
- `msedge_tts`: For text-to-speech functionality.
- `anyhow`: For error handling.

## Notes

- Ensure the `CONFIG_DIR` is correctly set up for loading and saving configurations.
- The bot uses a default nickname and channel, which can be overridden in the configuration file.
