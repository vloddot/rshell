use std::env;
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use super::ALIASES;

pub enum Builtin {
    Alias,
    Cd,
    Echo,
    Exit,
    History,
    Pwd,
}

impl FromStr for Builtin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alias" => Ok(Self::Alias),
            "echo" => Ok(Self::Echo),
            "exit" => Ok(Self::Exit),
            "history" => Ok(Self::History),
            "cd" => Ok(Self::Cd),
            "pwd" => Ok(Self::Pwd),
            command => Err(command.to_string()),
        }
    }
}

impl Builtin {
    /// Mimics `alias` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man1/alias.1p.html)
    ///
    /// # Panics
    ///
    /// Panics if the alias lock could not be obtained.
    #[must_use]
    pub fn alias(args: &[String]) -> i32 {
        let mut lock = match ALIASES.lock() {
            Ok(lock) => lock,
            Err(_) => return 1,
        };

        match args.len() {
            0 => {
                for key in lock.aliases.keys() {
                    println!("{}='{}'", key, lock.get(key).unwrap());
                }
                0
            }
            1 => {
                if args[0].contains('=') {
                    let (key, value) = args[0].split_once('=').unwrap();
                    lock.set(key.to_string(), value.to_string());
                    0
                } else if let Some(value) = lock.get(args[0].clone().as_str()) {
                    println!("{}='{}'", args[0], value);
                    0
                } else {
                    println!("{} not found", args[0]);
                    2
                }
            }
            _ => {
                eprintln!("Too many arguments");
                3
            }
        }
    }

    /// Mimics `cd` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man1/cd.1p.html)
    #[must_use]
    pub fn cd(args: &[String]) -> i32 {
        let home_dir = env::var("HOME").unwrap_or_else(|_| "/".to_string());

        if let Err(error) = std::env::set_current_dir(args.get(0).unwrap_or(&home_dir)) {
            eprintln!("rshell: {}", error);
            1
        } else {
            0
        }
    }

    /// Mimics `echo` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man1/echo.1p.html)
    #[must_use]
    pub fn echo(args: &[String]) -> i32 {
        println!("{}", args.join(" "));
        0
    }

    /// Mimics `history` builtin Unix shell command. [Linux man page](https://www.man7.org/linux/man-pages/man3/history.3.html)
    ///
    /// # Panics
    ///
    /// Panics if line from history file could not be read.
    pub async fn history(_args: &[String]) -> i32 {
        let mut history = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()));
        history.push(".rshistory");

        let Ok(history) = tokio::fs::read(history).await else {
            eprintln!("rshell: could not read from ~/.rshistory");
            return 1;
        };

        for (i, line) in history.lines().enumerate() {
            println!("{} {}", i + 1, line.unwrap());
        }
        0
    }

    /// Mimics `pwd` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man1/pwd.1.html)
    #[must_use]
    pub fn pwd(_args: &[String]) -> i32 {
        let Ok(current_dir) = std::env::current_dir() else {
            eprintln!("rshell: could not find current directory");
            return 1;
        };
        println!("{}", current_dir.display());
        0
    }

    /// Mimics `exit` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man3/exit.3.html)
    #[must_use]
    pub fn exit(args: &[String]) -> i32 {
        if args.is_empty() {
            return 0;
        }

        args[0].parse().unwrap_or(0)
    }
}
