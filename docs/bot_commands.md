# Bot Commands

Bot commands are chat messages that start with the `!` prefix.

Incoming chat events are received by the Twitch client as `TwitchChatMessage` values and broadcast through `TWITCH_BROADCAST`. The command task subscribes to that broadcast, extracts the trigger, and runs the matching command handler.

## Built-In Commands

- `!help`: replies with all registered commands.
- `!list_locales`: replies with available TTS locales.
- `!reset_voice <filter>`: updates the sender's voice to a random voice matching the filter text.
- `!stop`: stops current audio playback.

## Command Registry

Commands are stored in `BOT_COMMANDS`.

Each command receives a `TwitchChatMessage`, which includes:

- `sender`: Twitch login of the chat user
- `payload`: full message text
- `message_id`: Twitch message id used for threaded replies

Handlers can reply with:

```rust
message.reply("response text").await?;
```

## External Commands

External commands are loaded and registered during command startup. See [External commands](bot_external_commands.md).
