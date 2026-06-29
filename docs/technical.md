# Technical Documentation

This section describes Bottarga's runtime modules and internal behavior.

For setup and day-to-day use, see the [User guide](user_guide.md).

## Runtime Flow

```text
main.rs
  |
  v
TASKS_MANAGER
  |
  +--> Twitch client
  |      |
  |      +--> EventSub WebSocket notifications
  |      +--> TwitchChatMessage broadcast
  |
  +--> Bot commands
  |      |
  |      +--> built-in commands
  |      +--> external commands
  |      +--> Helix chat replies
  |
  +--> TTS
  |      |
  |      +--> text sanitization
  |      +--> msedge-tts synthesis
  |
  +--> Audio player
         |
         +--> Kira playback
         +--> optional Linux PulseAudio sink playback
```

## Module Reference

- [Main runtime](main.md): task startup order and global configuration directory.
- [Twitch client](twitch_client.md): EventSub WebSocket, OAuth token loading, chat event handling, and Helix replies.
- [Bot commands](bot_commands.md): command registry and built-in command behavior.
- [External commands](bot_external_commands.md): configurable commands from `.config/ExternalBotCommands.toml`.
- [TTS](tts.md): voice database, per-user voices, text cleanup, and speech synthesis.
- [Audio player](audio_player.md): audio queue, stop command, Kira playback, and Linux sink playback.
- [Users](users.md): persisted user voice preferences.
- [Common utilities](common.md): TOML persistence, broadcast channels, and async queues.
- [Task manager](task_manager.md): task registration and restart behavior.
- [Task stats](task_stats.md): periodic task status logging.
- [Macros](macros.md): logging, timestamps, and source-location helpers.

## Configuration Files

The active configuration directory is `.config`. Runtime configuration files are generated automatically when missing.

Current files used by the application:

- `TwitchToken.toml`
- `TwitchScopesConfig.toml`
- `StreamerChannel.toml`
- `UsersDB.toml`
- `UserDefaultVoiceConfig.toml`
- `VoiceDB.toml`
- `AudioControl.toml`
- `ExternalBotCommands.toml`
