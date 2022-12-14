use crate::error;

use crate::ALIASES;
use async_recursion::async_recursion;
use clap::Arg;
use clap::ArgAction;

use std::{
    env,
    fmt::Display,
    io::BufRead,
    path::{Path, PathBuf},
    str::FromStr,
};

pub(crate) enum Builtin {
    Alias,
    Builtin,
    Cd,
    Echo,
    Exit,
    History,
    Pwd,
}

pub(crate) enum ErrorKind {
    InvalidInput,
    InvalidBuiltin,
}

pub(crate) struct Error<T = String> {
    pub(crate) kind: ErrorKind,
    pub(crate) message: T,
}

impl<T> Error<T>
where
    T: Display,
{
    pub(crate) fn new(kind: ErrorKind, message: T) -> Self {
        Self { kind, message }
    }
}

impl<T> Display for Error<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl FromStr for Builtin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alias" => Ok(Self::Alias),
            "echo" => Ok(Self::Echo),
            "exit" | "bye" => Ok(Self::Exit),
            "builtin" => Ok(Self::Builtin),
            "history" => Ok(Self::History),
            "cd" | "chdir" => Ok(Self::Cd),
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
    pub(crate) async fn alias(args: &[String]) -> i32 {
        let args = clap::Command::new("alias")
            .arg(
                Arg::new("alias-name")
                    .action(ArgAction::Set)
                    .required(false),
            )
            .try_get_matches_from(args);

        let Ok(args) = args else {
            eprintln!("alias: bad argument");
            return 1;
        };

        let mut lock = ALIASES.lock().await;

        let Ok(Some(alias_name)) = args.try_get_one::<String>("alias-name") else {
            for (key, value) in lock.aliases.clone() {
                println!("{key}={value}");
            }
            return 0;
        };

        if alias_name.contains('=') {
            let (key, value) = alias_name.split_once('=').unwrap();
            let value = value.trim_matches('\'').trim_matches('"');
            lock.set(key.to_string(), value.to_string());
        } else if let Some(value) = lock.get(alias_name) {
            println!("{alias_name}={value}");
        } else {
            eprintln!("alias: {alias_name} not found");
            return 2;
        }

        0
    }

    /// Mimics `builtin` builtin Unix shell command. [Linux man page]()
    #[async_recursion]
    #[must_use]
    pub(crate) async fn builtin(args: &[String]) -> i32 {
        match Self::run(&args[1..]).await {
            Ok(result) => result,
            Err(error) => match error.kind {
                ErrorKind::InvalidBuiltin => {
                    error!("no such builtin: {error}");
                    1
                }
                ErrorKind::InvalidInput => {
                    error!("{error}");
                    2
                }
            },
        }
    }

    /// Mimics `cd` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man1/cd.1p.html)
    #[must_use]
    pub(crate) fn cd(args: &[String]) -> i32 {
        if args.len() != 2 {
            eprintln!("cd: expected [PATH] argument");
            return 1;
        }

        let path = Path::new(&args[1]);

        if !path.exists() {
            eprintln!("cd: no such file or directory: {}", path.display());
            return 2;
        }

        if let Err(error) = std::env::set_current_dir(path) {
            eprintln!("cd: {error}");
            return 3;
        }

        0
    }

    /// Mimics `echo` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man1/echo.1p.html)
    #[must_use]
    pub(crate) fn echo(args: &[String]) -> i32 {
        println!("{}", args[1..].join(" "));
        0
    }

    /// Mimics `exit` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man3/exit.3.html)
    #[must_use]
    pub(crate) fn exit(args: &[String]) -> i32 {
        args.get(0)
            .unwrap_or(&String::from("0"))
            .parse()
            .unwrap_or(0)
    }

    /// Mimics `history` builtin Unix shell command. [Linux man page](https://www.man7.org/linux/man-pages/man3/history.3.html)
    ///
    /// # Panics
    ///
    /// Panics if line from history file could not be read.
    pub(crate) async fn history(_args: &[String]) -> i32 {
        let mut history = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()));
        history.push(".rshistory");

        let Ok(history) = tokio::fs::read(history).await else {
            error!("could not read from ~/.rshistory");
            return 1;
        };

        for (i, line) in history.lines().enumerate() {
            println!("{} {}", i + 1, line.unwrap());
        }
        0
    }

    /// Mimics `pwd` builtin Unix shell command. [Linux man page](https://man7.org/linux/man-pages/man1/pwd.1.html)
    #[must_use]
    pub(crate) fn pwd(_args: &[String]) -> i32 {
        let Ok(current_dir) = std::env::current_dir() else {
            error!("could not find current directory");
            return 1;
        };

        println!("{}", current_dir.display());
        0
    }

    /// Runs a builtin if it is one.
    ///
    /// # Errors
    ///
    /// This function will return an error if the command is not a builtin [`std::io::ErrorKind::InvalidInput`].
    pub(crate) async fn run(args: &[String]) -> Result<i32, Error> {
        if args.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                String::from("expected 1 argument"),
            ));
        }

        match Self::from_str(args[0].as_str()) {
            Ok(Self::Alias) => Ok(Self::alias(args).await),
            Ok(Self::Builtin) => Ok(Self::builtin(args).await),
            Ok(Self::Cd) => Ok(Self::cd(args)),
            Ok(Self::Echo) => Ok(Self::echo(args)),
            Ok(Self::Exit) => Ok(Self::exit(args)),
            Ok(Self::History) => Ok(Self::history(args).await),
            Ok(Self::Pwd) => Ok(Self::pwd(args)),
            Err(command) => Err(Error::new(ErrorKind::InvalidBuiltin, command)),
        }
    }
}
