use tokio::{io, process};

use std::time::Duration;

use crate::{
    error,
    lang::{
        builtin::Builtin,
        parser::{self, Parser},
        scanner::Scanner,
    },
    SIGINT_EXIT_CODE,
};

#[derive(Clone, Debug, Default)]
pub struct Command {
    pub(crate) keyword: String,
    pub(crate) args: Vec<String>,
}

impl Command {
    /// Interprets the command based on its keyword and arguments.
    ///
    /// Returns the exit code of the process.
    ///
    /// # Errors
    /// This function also uses the [`error!`] macro to report errors to stdout.
    ///
    /// # Panics
    ///
    /// Panics if the spawned command process' exit code could not be extracted.
    ///
    /// # Returns
    ///
    /// This function returns the exit code of the process being executed and the time it took.
    ///
    /// It returns an exit code of 1 if waiting for the process to finish failed.
    ///
    /// It returns an exit code of 2 if the process couldn't be spawned
    /// due to the command not existing, some low level I/O issues, etc.
    ///
    /// # Command aliases
    ///
    /// If the command is a key inside of the `rshell::ALIASES`. It executes the aliased command.
    async fn interpret(&self) -> Option<i32> {
        let mut args = self.args.clone();
        args.insert(0, self.keyword.clone());

        match Builtin::run(&args).await {
            Ok(code) => Some(code),
            Err(command) => {
                let command = command.to_string();

                if command.is_empty() {
                    Some(0)
                } else {
                    let process = process::Command::new(command.clone())
                        .args(self.args.clone())
                        .spawn();

                    match process {
                        Ok(mut process) => match process.wait().await {
                            Ok(process) => process.code(),
                            Err(error) => {
                                error!("{error}");
                                Some(1)
                            }
                        },
                        Err(error) => {
                            let kind = error.kind();
                            if let io::ErrorKind::NotFound = kind {
                                error!("command not found: {command}");
                            } else {
                                error!("{error}");
                            }
                            Some(kind as i32)
                        }
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn new(keyword: String, args: Vec<String>) -> Self {
        Self { keyword, args }
    }

    /// Runs a command from a string.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing throws an error.
    pub async fn run(command: &str) -> (Result<i32, parser::error::Error>, Duration) {
        let mut scanner = Scanner::new(command);
        let tokens = scanner.scan_tokens().await;

        let mut parser = Parser::new(tokens);
        let commands = match parser.parse_tokens() {
            Ok(commands) => commands,
            Err(error) => {
                return (Err(error), Duration::default());
            }
        };

        let start = tokio::time::Instant::now();
        for command in commands {
            let exit_code = command.interpret().await;

            if let Some(exit_code) = exit_code {
                if exit_code != 0 {
                    return (Ok(exit_code), start.elapsed());
                }
            } else {
                return (Ok(SIGINT_EXIT_CODE), start.elapsed());
            }
        }

        (Ok(0), start.elapsed())
    }
}
