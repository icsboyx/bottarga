# Main Runtime

`main.rs` is the application entry point.

It defines:

```rust
pub static CONFIG_DIR: Option<&'static str> = Some(".config");
```

and starts the main async tasks through `TASKS_MANAGER`.

## Started Tasks

- `TWITCH_CLIENT`: connects to Twitch EventSub WebSocket and emits chat events.
- `TTS`: converts queued text into audio.
- `AUDIO_PLAYER`: plays queued audio bytes.
- `BOT_COMMANDS`: handles built-in and external chat commands.

## Startup Flow

```text
main()
  |
  v
register tasks
  |
  v
TASKS_MANAGER.list()
  |
  v
TASKS_MANAGER.run_tasks()
```

Each task returns an `eyre::Result<()>`. The task manager is responsible for restart behavior.
