# Macros Documentation

This document provides an overview of the macros available in the `macros.rs` file.

## `now!`

Generates a formatted string representing the current date and time in the format `YYYY-MM-DD HH:MM:SS`.

### Example

```rust
let current_time = now!();
println!("{}", current_time);
```

---

## `here!`

Generates a string containing the current file name and line number.

### Example

```rust
let location = here!();
println!("{}", location);
```

---

## `log_write!`

Logs a message with a timestamp and optional color formatting.

### Example

```rust
log_write!(colored::Color::Green, "This is a log message.");
```

---

## `log_write_original!`

Logs a message with a timestamp and optional color formatting (original implementation).

### Example

```rust
log_write_original!(colored::Color::Blue, "Original log message.");
```

---

## `log!`

Logs a message in white color.

### Example

```rust
log!("This is a log message.");
```

---

## `log_error!`

Logs an error message in red color.

### Example

```rust
log_error!("This is an error message.");
```

---

## `log_warning!`

Logs a warning message in yellow color.

### Example

```rust
log_warning!("This is a warning message.");
```

---

## `log_debug!`

Logs a debug message in blue color. Only active in debug builds.

### Example

```rust
log_debug!("This is a debug message.");
```

---

## `log_debug_error!`

Logs a debug error message in red color. Only active in debug builds.

### Example

```rust
log_debug_error!("This is a debug error message.");
```

---

## `log_trace!`

Logs a trace message in magenta color. Only active in debug builds.

### Example

```rust
log_trace!("This is a trace message.");
```

---

## `log_trace_error!`

Logs a trace error message in red color. Only active in debug builds.

### Example

```rust
log_trace_error!("This is a trace error message.");
```

---

## `log_debugc!`

Logs a debug message with a custom color. Only active in debug builds.

### Example

```rust
log_debugc!(colored::Color::Cyan, "This is a custom debug message.");
```
