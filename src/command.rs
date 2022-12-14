use tokio::{
    io, process,
    time::{Duration, Instant},
};

use crate::{
    error,
    lang::{
        builtin::Builtin,
        parser::{self, Parser},
        scanner::Scanner,
    },
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
    async fn interpret(&self) -> (i32, Duration) {
        let start = Instant::now();

        let mut args = self.args.clone();
        args.insert(0, self.keyword.clone());

        (
            match Builtin::run(&args).await {
                Ok(code) => code,
                Err(command) => {
                    let command = command.to_string();
                    if command.is_empty() {
                        0
                    } else {
                        let process = process::Command::new(command.clone())
                            .args(self.args.clone())
                            .spawn();

                        match process {
                            Ok(mut process) => match process.wait().await {
                                Ok(process) => process.code().unwrap(),
                                Err(error) => {
                                    error!("{error}");
                                    1
                                }
                            },
                            Err(error) => {
                                if let io::ErrorKind::NotFound = error.kind() {
                                    error!("command not found: {command}");
                                } else {
                                    error!("{error}");
                                }
                                2
                            }
                        }
                    }
                }
            },
            start.elapsed(),
        )
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
    pub async fn run(command: &str) -> (Result<i32, parser::Error>, Duration) {
        let mut scanner = Scanner::new(command);
        let tokens = scanner.scan_tokens().await;

        let mut parser = Parser::new(tokens);
        let commands = match parser.parse_tokens() {
            Ok(commands) => commands,
            Err(error) => {
                return (Err(error), Duration::default());
            }
        };

        let mut duration_sum = Duration::default();
        for command in commands {
            let (exit_code, duration) = command.interpret().await;
            if exit_code != 0 {
                return (Ok(exit_code), duration);
            }
            duration_sum += duration;
        }

        (Ok(0), duration_sum)
    }
}
