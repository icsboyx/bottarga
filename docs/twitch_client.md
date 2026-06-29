# Twitch Client

The Twitch client connects Bottarga to Twitch chat through EventSub WebSocket.

It is responsible for:

- loading and validating the OAuth token
- loading the broadcaster channel
- opening the EventSub WebSocket connection
- subscribing to chat and whisper events
- converting chat notifications into `TwitchChatMessage`
- broadcasting chat messages to bot commands
- pushing normal chat text into the TTS queue
- sending replies through the Helix chat API

## Startup

On startup, `tw_client::start()` performs these steps:

1. Load and validate `TwitchToken.toml`.
2. Load `TwitchScopesConfig.toml`, creating it with default scopes if missing.
3. If the token is missing, invalid, or does not contain all configured scopes, print a Twitch authorization URL and ask for the redirected URL.
4. Load `StreamerChannel.toml`.
5. If the channel is missing, ask for a channel login and resolve it through Helix users API.
6. Connect to `wss://eventsub.wss.twitch.tv/ws`.
7. On `session_welcome`, create EventSub subscriptions for chat and whispers.

## Chat Message Flow

When a `channel.chat.message` notification arrives, the client extracts:

- sender login
- message text
- Twitch message id

The extracted data becomes:

```rust
TwitchChatMessage {
    sender,
    payload,
    message_id,
}
```

The message is then:

- broadcast through `TWITCH_BROADCAST` for command handling
- converted to a TTS message and pushed into `TTS_QUEUE`

Messages sent by the bot itself are ignored to avoid feedback loops.

## Replies

Replies use:

```rust
tw_api::send_chat_message(...)
```

`TwitchChatMessage::reply(...)` sends a threaded reply using the original Twitch `message_id` when available.

## Configuration

The Twitch client uses:

- `.config/TwitchToken.toml`
- `.config/TwitchScopesConfig.toml`
- `.config/StreamerChannel.toml`

It uses the shared `.config` directory and `PersistentConfig` TOML files.

`TwitchScopesConfig.toml` contains the OAuth scopes requested during authorization. The default file is generated from the scopes compiled into the application.

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
