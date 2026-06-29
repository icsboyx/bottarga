# Text-To-Speech

The TTS module converts Twitch chat text into audio.

It uses `msedge-tts` for synthesis and stores per-user voice preferences in `.config/UsersDB.toml`.

## Message Flow

```text
TwitchChatMessage
      |
      v
voice_msg(...)
      |
      v
TTS_QUEUE
      |
      v
text_to_speech(...)
      |
      v
TTS_AUDIO_QUEUE
```

## Voice Selection

For normal chat users, the voice is read from `UsersDB`. If the user is new, a voice is selected with the default filter from `UserDefaultVoiceConfig`.

Bot-generated responses use a random voice matching:

```text
it-IT multi
```

## Text Cleanup

Before synthesis, the module:

- removes URLs
- replaces selected characters such as `&` and `%`

This prevents long links and awkward symbols from being read aloud.

## Commands

- `!list_locales`: lists unique locales from the voice database.
- `!reset_voice <filter>`: updates the sender's voice to a random voice matching all filter terms.

Examples:

```text
!reset_voice it-IT female
!reset_voice multilingual
```

## Configuration

- `.config/VoiceDB.toml`: generated voice catalog.
- `.config/UserDefaultVoiceConfig.toml`: default voice filter for new users.
- `.config/UsersDB.toml`: per-user voice choices.
