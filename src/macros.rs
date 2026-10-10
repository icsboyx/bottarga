#[macro_export]
macro_rules! now {
    () => {{
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        format!("{}", now)
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
    (None) => {{
        let payload = format!("{} I'm here", here!());
        let prefix = now!();
        println!("| {} | {}", prefix, payload);
    }};
    (None, $($arg:tt)*) => {{
        let payload = format!($($arg)*);
        let prefix = now!();
        println!("| {} | {}", prefix, payload);
    }};
    ($color:expr) => {{
        use colored::Colorize;
        let payload = format!("{} I'm here", here!());
        let prefix = now!();
        println!("| {} | {}", prefix, payload.color($color));
    }};
    ($color:expr, $($arg:tt)*) => {{
        use colored::Colorize;
        let payload = format!($($arg)*);
        let prefix = now!();
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
        log_write!(None)
    }};
    ($($arg:tt)*) => {{
        log_write!(None, $($arg)*)
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
        log_write!(Yellow,"[WARNING] ")
    }};
    ($($arg:tt)*) => {{
        use colored::Color::Yellow;
        let payload = format!($($arg)*);
        log_write!(Yellow, "[WARNING] {}", payload);
    }};
}

#[macro_export]
macro_rules! log_debug {
    () => {
    #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        log_write!(Blue, "[DEBUG] ");
    }};
    ($($arg:tt)*) => {
    #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        let mut payload = format!($($arg)*);
        if payload.as_str().contains("PASS oauth"){payload = "PASS oauth :[ ************* Sensitive Data Content ************* ]".to_string()};
        log_write!(Blue, "[DEBUG] {}", payload);
    }};
}

#[macro_export]
macro_rules! log_debug_error {
    () => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        log_write!(Red)
    }};
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!(Red, "[DEBUG] {} {}", here!(), payload)
    }};
}

#[macro_export]
macro_rules! log_trace {
    () => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        log_write!(Magenta, "[TRACE] ")
    }};
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!(Magenta, "[TRACE] {} {}", here!(), payload)
    }};
}

#[macro_export]
macro_rules! log_trace_error {
    () => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        log_write!(Red, "[TRACE] ")
    }};
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!(Red, "[TRACE] {} {}", here!(), payload)
    }};
}

#[macro_export]
macro_rules! log_debugc {
    ($color:expr) => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        log_write!($color)
    }};
    ($color:expr,$($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
        use colored::Color::*;
        let payload = format!($($arg)*);
        log_write!($color, "{}", payload)
    }};
}
