#![warn(clippy::all, clippy::pedantic, clippy::style, clippy::use_self)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod builtin;
pub mod command;

pub use command::Command;

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

impl Default for Aliases {
    fn default() -> Self {
        Self::new()
    }
}
