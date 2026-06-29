# Task Stats

`task_stats.rs` is a small monitoring task for task-manager state.

It periodically reads statistics from `TASKS_MANAGER` and logs them. This is useful when the task is registered as part of the runtime, but it is not currently started by `main.rs`.

## Behavior

The task loops forever:

1. Read task stats from `TASKS_MANAGER`.
2. Log the collected stats.
3. Sleep for ten seconds.

## Notes

The current startup sequence registers Twitch, TTS, audio playback, and bot command tasks. If task statistics should be emitted at runtime, register `task_stats::start()` in `main.rs`.
