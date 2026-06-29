# Users

The users module stores per-user TTS voice preferences.

## Runtime Data

`USER_DB` is a global `RwLock<UsersDB>`.

`UsersDB` maps Twitch logins to `User` records. Each `User` stores:

- `nick`
- `speech_config`

## New Users

When a chat user is seen for the first time, Bottarga creates a user record automatically.

The default voice is selected from `TTS_VOCE_BD` using the filter in `.config/UserDefaultVoiceConfig.toml`.

## Updating Voices

The `!reset_voice <filter>` command updates the sender's voice. The filter is matched against the serialized voice metadata, and one matching voice is selected randomly.

## Persistence

User data is saved with `PersistentConfig` in:

```text
.config/UsersDB.toml
```
