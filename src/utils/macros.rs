/// Prints a timestamped message to stderr.
///
/// In debug builds the message includes the current source location from
/// [`here!`]. In release builds it includes the current module path.
///
/// # Examples
///
/// ```ignore
/// err_log!("Request failed: {}", "timeout");
/// ```
#[macro_export]
macro_rules! err_log {
    () => {{
        $crate::err_log!("");
    }};
    ($($arg:tt)*) => {{

        std::eprintln!(
            "[ {} ] {} {}",
            $crate::timestamp!(millis),
            if cfg!(debug_assertions){crate::here!()} else {format!("[ {:<20} ]", module_path!())},
            format!($($arg)*)
        );
    }};
}

/// Re-export of `chrono` for macro expansions.
pub use chrono;

/// Returns the current source location as a string.
///
/// With no arguments, it returns a left-aligned, bracketed
/// `"<file>:<line>:<column>"` string.
/// With a format string, it appends the formatted message after the location.
///
/// # Examples
///
/// ```ignore
/// use botox::here;
///
/// let location = here!();
/// assert!(location.starts_with("[ "));
/// assert!(location.contains(".rs:"));
/// ```
///
/// ```ignore
/// use botox::here;
///
/// let message = here!("value = {}", 42);
/// assert!(message.contains("value = 42"));
/// ```
#[macro_export]
macro_rules! here_v2 {
    () => {
        format!(
            "[ {:<30} ]",
            format!("{}:{}:{}", file!(), line!(), column!())
        )
    };
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        format!(
            "[ {:<30} ] {}",
            format!("{}:{}:{}", file!(), line!(), column!()),
            format_args!($fmt $(, $arg)*)
        )
    };
}

/// Returns the current local date and time with microsecond precision.
///
/// The output format is `YYYY-MM-DD HH:MM:SS:ffffff`.
///
/// # Examples
///
/// ```ignore
/// use botox::now;
///
/// let value = now!();
/// assert!(!value.is_empty());
/// ```
#[macro_export]
macro_rules! now_v2 {
    () => {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S:%6f").to_string()
    };
}

/// Returns the current UTC date and time with microsecond precision.
///
/// The output format is `YYYY-MM-DD HH:MM:SS:ffffff`.
///
/// # Examples
///
/// ```ignore
/// use botox::now_utc;
///
/// let value = now_utc!();
/// assert!(!value.is_empty());
/// ```
#[macro_export]
macro_rules! now_utc {
    () => {
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S:%6f").to_string()
    };
}

/// Returns the current local timestamp as a formatted string.
///
/// Supported forms:
/// - `timestamp!()` for second precision
/// - `timestamp!(milliseconds)` or `timestamp!(millis)`
/// - `timestamp!(micro)` or `timestamp!(micros)`
/// - `timestamp!(nano)` or `timestamp!(nanos)`
/// - `timestamp!("<chrono format>")` for a custom format string
///
/// # Examples
///
/// ```ignore
/// use botox::timestamp;
///
/// let seconds = timestamp!();
/// let millis = timestamp!(millis);
/// let custom = timestamp!("%Y-%m-%d");
///
/// assert!(!seconds.is_empty());
/// assert!(!millis.is_empty());
/// assert_eq!(custom.len(), 10);
/// ```
// TODO! timestamp epoch (not human)
#[macro_export]
macro_rules! timestamp {
    // default: seconds precision (local time)
    () => {{ chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string() }};

    // ---- specific shorthands FIRST ----
    (milliseconds) => {{ chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string() }};
    (millis) => {{ $crate::timestamp!(milliseconds) }};

    (micro) => {{ chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string() }};
    (micros) => {{ $crate::timestamp!(micro) }};

    (nano) => {{ chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.9f").to_string() }};
    (nanos) => {{ $crate::timestamp!(nano) }};

    // custom format string — keep LAST, limit to literals so identifiers don’t get captured
    ($fmt:literal) => {{ chrono::Local::now().format($fmt).to_string() }};

    // helpful error for anything else
    ($unknown:tt) => {
        compile_error!(
            "unsupported timestamp! argument. Use: milliseconds|millis|micro|micros|nano|nanos|\"custom fmt\""
        );
    };
}

/// Returns the current UTC timestamp as a formatted string.
///
/// Supported forms:
/// - `timestamp_utc!()` for second precision
/// - `timestamp_utc!(milliseconds)` or `timestamp_utc!(millis)`
/// - `timestamp_utc!(micro)` or `timestamp_utc!(micros)`
/// - `timestamp_utc!(nano)` or `timestamp_utc!(nanos)`
/// - `timestamp_utc!("<chrono format>")` for a custom format string
///
/// # Examples
///
/// ```ignore
/// use botox::timestamp_utc;
///
/// let seconds = timestamp_utc!();
/// let millis = timestamp_utc!(millis);
/// let custom = timestamp_utc!("%Y-%m-%d");
///
/// assert!(!seconds.is_empty());
/// assert!(!millis.is_empty());
/// assert_eq!(custom.len(), 10);
/// ```
#[macro_export]
macro_rules! timestamp_utc {
    // default: seconds precision (Utc time)
    () => {{ chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string() }};

    // ---- specific shorthands FIRST ----
    (milliseconds) => {{ chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string() }};
    (millis) => {{ $crate::timestamp_utc!(milliseconds) }};

    (micro) => {{ chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string() }};
    (micros) => {{ $crate::timestamp_utc!(micro) }};

    (nano) => {{ chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.9f").to_string() }};
    (nanos) => {{ $crate::timestamp_utc!(nano) }};

    // custom format string — keep LAST, limit to literals so identifiers don’t get captured
    ($fmt:literal) => {{ chrono::Utc::now().format($fmt).to_string() }};

    // helpful error for anything else
    ($unknown:tt) => {
        compile_error!(
            "unsupported timestamp_utc! argument. Use: milliseconds|millis|micro|micros|nano|nanos|\"custom fmt\""
        );
    };
}
