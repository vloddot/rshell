#![warn(clippy::all, clippy::pedantic, clippy::style, clippy::use_self)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod command;
mod lang;

pub use command::Command;

/// Green foreground color.
pub const GREEN_FG: termion::color::Fg<termion::color::Green> =
    termion::color::Fg(termion::color::Green);

/// Red foreground color.
pub const RED_FG: termion::color::Fg<termion::color::Red> = termion::color::Fg(termion::color::Red);

/// Reset foreground color.
pub const RESET_FG: termion::color::Fg<termion::color::Reset> =
    termion::color::Fg(termion::color::Reset);

pub const UNICODE_PROMPT: char = '❯';
pub const RSHISTORY: &str = ".rshistory";
pub const RSHELL_RC: &str = ".rshellrc";

lazy_static! {
    pub static ref ALIASES: Mutex<Aliases> = Mutex::new(Aliases::new());
}

pub struct Aliases {
    pub aliases: HashMap<String, String>,
}

impl Aliases {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.aliases.get(key)
    }

    fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Option<String> {
        self.aliases.insert(key, value)
    }
}

#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        eprintln!("rshell: {}", format_args!($($args)*))
    };
}
