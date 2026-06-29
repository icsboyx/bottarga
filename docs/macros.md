# Macros

Bottarga uses macros for lightweight logging, timestamps, and source locations.

## Logging

Common logging macros:

- `log!(...)`: normal stdout log.
- `log_error!(...)`: error-style stdout log.
- `log_warning!(...)`: warning-style stdout log.
- `log_debug!(...)`: debug-only stdout log.
- `log_trace!(...)`: debug-only trace log.
- `err_log!(...)`: stderr log with timestamp and location/module context.

Debug and trace macros are compiled only for debug builds where applicable.

## Source Location

- `here!()`: returns the current file and line.
- `here_v2!()`: returns file, line, and column in a padded format.

## Timestamps

- `now!()`: legacy local timestamp.
- `now_v2!()`: local timestamp with microsecond precision.
- `now_utc!()`: UTC timestamp with microsecond precision.
- `timestamp!()`: local timestamp with selectable precision.
- `timestamp_utc!()`: UTC timestamp with selectable precision.

Supported precision arguments for `timestamp!` and `timestamp_utc!`:

- `millis` / `milliseconds`
- `micros` / `micro`
- `nanos` / `nano`
- a custom chrono format string literal

Example:

```rust
log!("Twitch client started");
err_log!("HTTP request failed: {}", error);
let location = here!();
let ts = timestamp!(millis);
```
