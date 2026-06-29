# External Bot Commands

External commands are user-configurable chat commands loaded from `.config/ExternalBotCommands.toml`.

They are useful for simple replies and optional audio clips without changing Rust code.

## Configuration Format

```toml
[commands.hello]
activation_pattern = "hello"
aliases = ["hi"]
need_arg = false
custom_audio_url = ""
replay_text = "Hello, {SENDER}!"

[commands.for_president]
activation_pattern = "for_president"
aliases = []
need_arg = true
custom_audio_url = ""
replay_text = "{ARG} for President!"
```

## Fields

- `activation_pattern`: command trigger without the `!` prefix.
- `aliases`: optional alternative triggers.
- `need_arg`: when `true`, the command expects text after the trigger.
- `custom_audio_url`: optional audio URL to fetch and push to the audio queue.
- `replay_text`: chat reply template.

## Placeholders

- `{SENDER}`: replaced with the Twitch login that triggered the command.
- `{ARG}`: replaced with the text after the command trigger.

## Runtime Behavior

When an external command runs, Bottarga:

1. Builds the reply text.
2. Downloads and queues custom audio when `custom_audio_url` is not empty.
3. Queues the reply for TTS using the bot voice.
4. Replies in Twitch chat through Helix.
