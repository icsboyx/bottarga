# Task Manager

The task manager runs Bottarga's long-lived async services.

Each task is registered with:

- a name
- an async function factory
- a maximum restart count

## Behavior

When `run_tasks()` is called, the manager starts all registered tasks. If a task exits with an error, it can be restarted until its configured restart limit is reached.

This is used for:

- Twitch client
- TTS worker
- audio player
- command worker

## Example

```rust
TASKS_MANAGER
    .add("TTS", || Box::pin(tts::start()), 3)
    .await;
```

The example registers the TTS task and allows up to three restarts.
