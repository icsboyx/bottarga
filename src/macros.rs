#[macro_export]
macro_rules! now {
    () => {{
        // Import necessary items
        use std::time::{SystemTime, UNIX_EPOCH};

        // Get the current time since the Unix epoch
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

        // Convert duration to seconds
        let seconds = duration_since_epoch.as_secs();

        // Constants for time conversion
        const DAYS_IN_YEAR: u64 = 365;
        const SECONDS_IN_MINUTE: u64 = 60;
        const SECONDS_IN_HOUR: u64 = 3600;
        const SECONDS_IN_DAY: u64 = 86400;

        // Convert seconds to a rough approximation of date components
        let years_since_epoch = seconds / (SECONDS_IN_DAY * DAYS_IN_YEAR);
        let year = 1970 + years_since_epoch as i32; // Start from Unix epoch (1970)

        let days_since_epoch = (seconds / SECONDS_IN_DAY) % DAYS_IN_YEAR;
        let month = 1 + (days_since_epoch / 30); // Approximate month (not accounting for leap years)
        let day = 1 + (days_since_epoch % 30); // Approximate day

        // Convert seconds into hours, minutes, and seconds
        let hours = (seconds / SECONDS_IN_HOUR) % 24; // Get hours in a 24-hour format
        let minutes = (seconds % SECONDS_IN_HOUR) / SECONDS_IN_MINUTE;
        let seconds = seconds % SECONDS_IN_MINUTE;

        format!(
            "{}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hours, minutes, seconds
        )
    }};
}
#[macro_export]
macro_rules! here {
    () => {
        format!("File: {} | Line: {} |", file!(), line!())
    };
    ($($arg:tt)*) => {
        format!("File: {} | Line: {} | {}", file!(), line!(), format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! log_write {
    ($color:expr) => {{
        use colored::Colorize;
        let prefix = now!();
        let payload = format!("{} I'm here", here!());
        let padding = format!("{}{}","\n"," ".repeat(prefix.len() + 5));
        let payload = payload
        let payload = payload.replace("\n", padding.as_ref());

        println!("| {} | {}", prefix, payload.color($color));
    }};
    ($color:expr,$($arg:tt)*) => {{
        use colored::Colorize;
        let prefix = now!();
        let payload = format!($($arg)*);
        let padding = format!("{}{}","\n"," ".repeat(prefix.len() + 5));
        let mut payload = payload.replace("\n", padding.as_ref());
        if payload.as_str().contains("PASS oauth"){payload = "PASS oauth:[ ************* Sensitive Data Content ************* ]".to_string()};
        println!("| {} | {}", prefix, payload.color($color));
    }};
}

#[macro_export]
macro_rules! log_write_original {
    ($color:expr) => {{
        use colored::Colorize;
        let payload = format!("{} I'm here" , here!());
        let prefix = now!();
        println!("| {} | {}", prefix, payload.color($color));
    }};
    ($color:expr,$($arg:tt)*)  => {{
        use colored::Colorize;
        let payload = format!($($arg)*);
        let prefix = now!();
        println!("| {} | {}", prefix, payload.color($color));
    }};
}

#[macro_export]
macro_rules! log {
    () => {{
        use colored::Color::White;
        log_write!(White)
    }};
    ($($arg:tt)*) => {{
        use colored::Color::White;
        log_write!(White, $($arg)*)
    }};
}

#[macro_export]
macro_rules! log_error {
    () => {{
        use colored::Color::Red;
        log_write!(Red)
    }};
    ($($arg:tt)*) => {{
        use colored::Color::Red;
        log_write!(Red, $($arg)*)
    }};
}

#[macro_export]
macro_rules! log_warning {
    () => {{
        use colored::Color::Yellow;
        log_write!(Yellow)
    }};
    ($($arg:tt)*) => {{
        use colored::Color::Yellow;
        log_write!(Yellow, $($arg)*)
    }};
}

#[macro_export]
macro_rules! log_debug {
    () => {{
        use colored::Color::*;
        log_write!(Blue, $($arg)*);
    }};
    ($($arg:tt)*) => {{
        use colored::Color::*;
        log_write!(Blue, $($arg)*);
    }};
}

#[macro_export]
macro_rules! log_debug_error {
    () => {{
        use colored::Color::*;
        log_write!(Red)
    }};
    ($($arg:tt)*) => {{
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!(Red, "{} {}", here!(), payload)
    }};
}

#[macro_export]
macro_rules! log_trace {
    () => {{
        use colored::Color::C
        log_write!(Magenta)
    }};
    ($($arg:tt)*) => {{
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!(Magenta, "{} {}", here!(), payload)
    }};
}

#[macro_export]
macro_rules! log_trace_error {
    () => {{
        use colored::Color::*;
        log_write!(Red)
    }};
    ($($arg:tt)*) => {{
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!(Red, "{} {}", here!(), payload)
    }};
}

#[macro_export]
macro_rules! log_debugc {
    ($color:expr) => {{
        use colored::Color::*;
        log_write!($color)
    }};
    ($color:expr,$($arg:tt)*) => {{
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!($color, "{}", payload)
    }};
}
