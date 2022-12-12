use tokio::process;

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
    pub keyword: String,
    pub args: Vec<String>,
}

impl Command {
    /// Interprets self ([`Command`]) based on its keyword and arguments.
    ///
    /// Returns the exit code of the process.
    ///
    /// # Panics
    ///
    /// Panics if the spawned command process' exit code could not be extracted.
    ///
    /// # Returns
    ///
    /// This function returns the exit code of the process being executed.
    ///
    /// If the command is a key inside of the `rshell::ALIASES`. It executes the aliased command.
    ///
    /// This function is asynchronous so that it can run asynchronous processess
    pub async fn interpret(&self) -> i32 {
        let mut args = vec![self.keyword.clone()];
        args.extend(self.args.clone());
        let args = args.as_slice();
        match Builtin::run(args).await {
            Ok(result) => result,
            Err(command) => {
                let command = command.to_string();
                if command.is_empty() {
                    return 0;
                }

                let process = process::Command::new(command)
                    .args(self.args.clone())
                    .spawn();

                match process {
                    Ok(mut process) => match process.wait().await {
                        Ok(process) => process.code().unwrap(),
                        Err(error) => {
                            error!("{error}");
                            2
                        }
                    },
                    Err(error) => {
                        error!("{error}");
                        3
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
    pub async fn run(i: &str) -> Result<i32, parser::Error> {
        let mut scanner = Scanner::new(i);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let commands = match parser.parse() {
            Ok(commands) => commands,
            Err(error) => {
                return Err(error);
            }
        };

        for command in commands {
            let exit_code = command.interpret().await;
            if exit_code != 0 {
                return Ok(exit_code);
            }
        }

        Ok(0)
    }
}
