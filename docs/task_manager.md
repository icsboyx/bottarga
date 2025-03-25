# Task Manager

The `task_manager` module handles the management and monitoring of asynchronous tasks in Bottarga.

---

## Key Features

- **Task Management**: Add, execute, and monitor tasks.
- **Statistics**: Retrieve task execution stats like success rates and execution times.
- **Retry Mechanism**: Automatically restart tasks based on predefined limits.

---

## Usage

- **Add a Task**:

  ```rust
  TASKS_MANAGER.add_task(async {
      // Task logic
  }).await;
  ```

- **Retrieve Statistics**:
  ```rust
  let stats = TASKS_MANAGER.get_stats().await;
  println!("{:?}", stats);
  ```

---

## Integration

The `task_stats.rs` module periodically fetches task statistics from `TASKS_MANAGER` for real-time monitoring.

---

## Notes

- Ensure `TASKS_MANAGER` is initialized before use.
- Tasks should handle errors gracefully to avoid unnecessary retries.
