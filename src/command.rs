use std::{env, str::FromStr};
use tokio::process;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, space0},
    combinator::opt,
    multi::{many_m_n, separated_list0},
    IResult,
};

use super::{builtin::Builtin, ALIASES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub keyword: String,
    pub args: Vec<String>,
}

impl Command {
    /// Interprets this shell-like [`Command`] based on the keyword and arguments.
    ///
    /// Returns the exit code of the process.
    ///
    /// # Panics
    ///
    /// Panics if the error isn't a raw OS error or the process' code could not be extracted.
    ///
    /// # Returns
    ///
    /// This function returns the exit code of the process being executed.
    ///
    /// If the command is a key inside of the [`rshell::ALIASES`]. It executes the aliased command.
    ///
    /// This function is asynchronous so that it can run asynchronous processess
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///     let command = match rshell::command::Command::parse("ls / -a") {
    ///         Ok(result) => result.1[0].clone(),
    ///         Err(error) => {
    ///             eprintln!("rshell: {error}");
    ///             return;
    ///         }
    ///     };
    ///
    ///     let exit_code = command.interpret().await;
    ///     match exit_code {
    ///         0 => println!("Program executed successfully"),
    ///         code => eprintln!("Program exited with error code {code}"),
    ///     }
    /// }
    /// ```
    pub async fn interpret(&self) -> i32 {
        match Builtin::from_str(&self.keyword) {
            Ok(Builtin::Alias) => Builtin::alias(&self.args),
            Ok(Builtin::Cd) => Builtin::cd(&self.args),
            Ok(Builtin::Echo) => Builtin::echo(&self.args),
            Ok(Builtin::Exit) => std::process::exit(Builtin::exit(&self.args)),
            Ok(Builtin::History) => Builtin::history(&self.args).await,
            Ok(Builtin::Pwd) => Builtin::pwd(&self.args),
            Err(command) => {
                if command.is_empty() {
                    return 0;
                }

                let command = {
                    let alias_lock = match ALIASES.lock() {
                        Ok(lock) => lock,
                        Err(error) => {
                            eprintln!("rshell: {error}");
                            return 1;
                        }
                    };

                    let alias = alias_lock.get(command.clone().as_str());

                    if let Some(alias) = alias {
                        alias.to_string()
                    } else {
                        command
                    }
                };

                let mut process = process::Command::new(command)
                    .args(self.args.clone())
                    .spawn();

                if let Err(error) = process {
                    eprintln!("{}", error);
                    return error.raw_os_error().unwrap();
                }

                // Wait for the command to run.
                match process.as_mut().unwrap().wait().await {
                    Ok(process) => process.code().unwrap(),
                    Err(error) => {
                        eprintln!("rshell: {error}");
                        2
                    }
                }
            }
        }
    }

    /// This function tokenizes a string and returns a [`Result<(&str, Command), nom::Err<nom::error::Error>>`]
    ///
    /// # Errors
    ///
    /// This function will return an error if something went wrong while tokenizing.
    pub fn parse(i: &str) -> IResult<&str, Vec<Self>> {
        // match any whitespace before
        let (i, _) = space0(i)?;

        // if no command is given
        if i.is_empty() || i == "\n" {
            return Ok((
                i,
                vec![Self {
                    keyword: String::new(),
                    args: Vec::new(),
                }],
            ));
        }

        // get parsed command and its arguments.
        let result = separated_list0(tag("&&"), parts)(i)?;

        let i = result.0;
        let commands = result.1;

        println!("{:?}", commands);

        let commands: Vec<Self> = commands
            .iter()
            .map(|parts| {
                let parts: Vec<String> = parts
                    .iter()
                    .map(|part| {
                        if let Some(var) = part.strip_prefix("${") {
                            if let Some(var) = var.strip_suffix('}') {
                                let (var, default) = var.split_once(":-").unwrap_or((var, ""));
                                env::var(var).unwrap_or_else(|_| default.to_string())
                            } else {
                                String::new()
                            }
                        } else if let Some(var) = part.strip_prefix('$') {
                            env::var(var).unwrap_or_default()
                        } else {
                            part.clone()
                        }
                    })
                    .collect();

                Self {
                    keyword: parts.get(0).unwrap_or(&String::new()).clone(),
                    args: parts[1..].to_vec(),
                }
            })
            .collect();

        Ok((i, commands))
    }
}

#[doc(hidden)]
fn parts(i: &str) -> IResult<&str, Vec<String>> {
    let mut result = vec![];
    let mut i = i;

    while let (i2, Some(part)) = opt(string)(i)? {
        result.push(part);
        i = i2;
    }

    Ok((i, result))
}

#[doc(hidden)]
fn string(i: &str) -> IResult<&str, String> {
    let (i, _) = space0(i)?;

    let (i, result) = many_m_n(
        1,
        usize::MAX,
        alt((plain_string, single_quoted_string, double_quoted_string)),
    )(i)?;

    Ok((i, result.join("")))
}

#[doc(hidden)]
fn plain_string(i: &str) -> IResult<&str, &str> {
    take_while1(|c| !vec!['\'', '"', ' ', '\r', '\n'].contains(&c))(i)
}

#[doc(hidden)]
fn single_quoted_string(i: &str) -> IResult<&str, &str> {
    let (i, _) = char('\'')(i)?;
    let (i, result) = take_while(|c| !vec!['\''].contains(&c))(i)?;
    let (i, _) = char('\'')(i)?;

    Ok((i, result))
}

#[doc(hidden)]
fn double_quoted_string(i: &str) -> IResult<&str, &str> {
    let (i, _) = char('"')(i)?;
    let (i, result) = take_while(|c| !vec!['"'].contains(&c))(i)?;
    let (i, _) = char('"')(i)?;

    Ok((i, result))
}

#[cfg(test)]
mod command_parse_tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        assert_eq!(
            Command::parse("ls / -a\n"),
            Ok((
                "\n",
                vec![Command {
                    keyword: String::from("ls"),
                    args: vec![String::from("/"), String::from("-a")],
                }]
            ))
        );
    }

    #[test]
    fn test_newline() {
        assert_eq!(
            Command::parse("\n"),
            Ok((
                "\n",
                vec![Command {
                    keyword: String::new(),
                    args: Vec::new()
                }]
            ))
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(
            Command::parse(""),
            Ok((
                "",
                vec![Command {
                    keyword: String::new(),
                    args: Vec::new()
                }],
            ))
        );
    }

    #[test]
    fn test_variables() {
        assert_eq!(
            Command::parse("echo $USER\n"),
            Ok((
                "\n",
                vec![Command {
                    keyword: String::from("echo"),
                    args: vec![env::var("USER").unwrap()],
                }],
            ))
        );
    }
}
