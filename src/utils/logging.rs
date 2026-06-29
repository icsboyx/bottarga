use std::{
    collections::HashMap,
    fmt::Display,
    sync::{LazyLock, RwLock},
};

pub static LOG_MODULE_COLOR: LazyLock<ModuleColor> = LazyLock::new(|| ModuleColor::default());

#[derive(Debug)]
pub struct ModuleColor {
    module_colors: RwLock<HashMap<String, ANSIColor>>,
}

impl Default for ModuleColor {
    fn default() -> Self {
        Self {
            module_colors: RwLock::new(HashMap::new()),
        }
    }
}

impl ModuleColor {
    pub fn set_color(&self, module: impl AsRef<str>, color: ANSIColor) {
        if let Ok(mut colors) = self.module_colors.write() {
            colors.insert(module.as_ref().to_string(), color);
        } else {
            eprintln!(
                "Failed to acquire write lock for module colors. Could not set color for module '{}'.",
                module.as_ref()
            );
        }
    }

    pub fn get_color(&self, module: impl AsRef<str>) -> ANSIColor {
        self.module_colors
            .read()
            .ok()
            .and_then(|colors| colors.get(module.as_ref()).copied())
            .unwrap_or_default()
    }
}

#[macro_export]
macro_rules! set_module_log_color {
    ($color:expr) => {{ $crate::utils::logging::LOG_MODULE_COLOR.set_color(module_path!().split("::").last().unwrap(), $color) }};
    ($module:expr, $color:expr) => {{ $crate::utils::logging::LOG_MODULE_COLOR.set_color($module, $color) }};
}

#[macro_export]
macro_rules! log {
    () => {
        $crate::log!("");
    };

    ($($arg:tt)*) => {{
        let module = module_path!().rsplit("::").next().unwrap_or_default();
        let color = $crate::utils::logging::LOG_MODULE_COLOR.get_color(module);


        std::println!(
            "{}[ {} ] {} {}\x1b[0m",
            color,
            $crate::timestamp!(millis),
            if cfg!(debug_assertions) {
                $crate::here!()
            } else {
                format!("[ {:<20} ]", module)
            },
            format!($($arg)*)
        );
    }};
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ANSIColor {
    #[default]
    Default,

    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Orange,
    Grey,

    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,

    LightGrey,
    BrightWhite,

    Reset,
}

impl Display for ANSIColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            ANSIColor::Default => "",

            ANSIColor::Black => "\x1b[30m",
            ANSIColor::Red => "\x1b[31m",
            ANSIColor::Green => "\x1b[32m",
            ANSIColor::Yellow => "\x1b[33m",
            ANSIColor::Blue => "\x1b[34m",
            ANSIColor::Magenta => "\x1b[35m",
            ANSIColor::Cyan => "\x1b[36m",
            ANSIColor::Orange => "\x1b[38;5;208m",
            ANSIColor::White => "\x1b[37m",
            ANSIColor::Grey => "\x1b[90m",
            ANSIColor::LightRed => "\x1b[91m",
            ANSIColor::LightGreen => "\x1b[92m",
            ANSIColor::LightYellow => "\x1b[93m",
            ANSIColor::LightBlue => "\x1b[94m",
            ANSIColor::LightMagenta => "\x1b[95m",
            ANSIColor::LightCyan => "\x1b[96m",

            ANSIColor::LightGrey => "\x1b[97m",
            ANSIColor::BrightWhite => "\x1b[1;97m",

            ANSIColor::Reset => "\x1b[0m",
        };

        write!(f, "{code}")
    }
}

#[allow(non_snake_case)]
pub trait ColorText {
    fn RED(&self) -> impl Display
    where
        Self: Sized + Display,
    {
        format!("{}{}{}", ANSIColor::Red, self, ANSIColor::Reset)
    }
    fn GREEN(&self) -> impl Display
    where
        Self: Sized + Display,
    {
        format!("{}{}{}", ANSIColor::Green, self, ANSIColor::Reset)
    }
    fn YELLOW(&self) -> impl Display
    where
        Self: Sized + Display,
    {
        format!("{}{}{}", ANSIColor::Yellow, self, ANSIColor::Reset)
    }
    fn BLUE(&self) -> impl Display
    where
        Self: Sized + Display,
    {
        format!("{}{}{}", ANSIColor::Blue, self, ANSIColor::Reset)
    }

    fn MAGENTA(&self) -> impl Display
    where
        Self: Sized + Display,
    {
        format!("{}{}{}", ANSIColor::Magenta, self, ANSIColor::Reset)
    }

    fn CYAN(&self) -> impl Display
    where
        Self: Sized + Display,
    {
        format!("{}{}{}", ANSIColor::Cyan, self, ANSIColor::Reset)
    }

    fn ORANGE(&self) -> impl Display
    where
        Self: Sized + Display,
    {
        format!("{}{}{}", ANSIColor::Orange, self, ANSIColor::Reset)
    }
}

impl<T> ColorText for T where T: Display {}
