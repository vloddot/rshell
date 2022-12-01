use rshell::{Command, GREEN_FG, RED_FG, RESET_FG, RSHELL_RC, RSHISTORY, UNICODE_PROMPT};

use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
};

use tokio::{
    fs::OpenOptions,
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
};

#[tokio::main]
async fn main() {
    // get home directory
    let home_dir = match env::var("HOME") {
        Ok(dir) => Some(dir),
        Err(_) => None,
    };

    let home_dir = home_dir.map(PathBuf::from);

    // open history file to store commands into history
    let history = home_dir.clone();

    let mut history = if let Some(mut history) = history {
        history.push(RSHISTORY);

        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(history)
            .await
        {
            Ok(history) => Some(history),
            Err(_) => None,
        }
    } else {
        None
    };

    // run shellrc
    let shellrc = home_dir.clone();

    if let Some(mut shellrc) = shellrc {
        shellrc.push(RSHELL_RC);

        let shellrc = match tokio::fs::read(shellrc).await {
            Ok(rc) => Some(rc),
            Err(_) => None,
        };

        if let Some(shellrc) = shellrc {
            let mut lines = shellrc.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let commands = match Command::parse(line.as_str()) {
                    Ok(command) => command.1,
                    Err(error) => {
                        eprintln!("rshell: {error}");
                        continue;
                    }
                };

                for command in commands {
                    let code = command.interpret().await;
                    if code != 0 {
                        break;
                    }
                }
            }
        }
    }

    let mut exit_code = 0;

    loop {
        let current_dir = env::current_dir().expect("Current directory not found.");

        print_prompt(exit_code, home_dir.as_deref(), &current_dir);

        let command = read_command().await;

        // write command into history
        if let Some(ref mut history) = history {
            history.write_all(command.as_bytes()).await.unwrap_or(());
        }

        // tokenize command
        let commands = match Command::parse(command.as_str()) {
            Ok(tokens) => tokens.1,
            Err(error) => {
                eprintln!("rshell: {error}");
                continue;
            }
        };

        // interpret command
        for command in commands {
            exit_code = command.interpret().await;
            if exit_code != 0 {
                break;
            }
        }
    }
}

/// Prints the shell prompt given the previous command's exit code, home directory
/// and current directory.
///
/// # Shell Prompt
///
/// Looks like this:
///     "\[~ if is relative to home directory\]/\[full path\] ❯ (green or red depending on exit code success or failure respectively)"
/// # Panics
///
/// Panics if flushing wasn't possible.
///
/// # Examples
///
/// ```no_run
/// print_prompt(0, "/Users/any", "/Users/any/sandbox") // prints "~/sandbox ❯ " with the ❯ character green
/// print_prompt(42069, "/Users/any", "/Users/any/sandbox") // prints "~/sandbox ❯ " with the ❯ character red
/// ```
fn print_prompt(exit_code: i32, home_dir: Option<&Path>, current_dir: &Path) {
    if let Some(home_dir) = home_dir {
        print!(
            "{} ",
            current_dir
                .to_str()
                .unwrap_or("/")
                .replace(home_dir.to_str().unwrap_or("/"), "~")
        );
    } else {
        print!("{} ", current_dir.to_str().unwrap_or("/"));
    }

    match exit_code {
        0 => print!("{}{} ", GREEN_FG, UNICODE_PROMPT),
        _ => print!("{}{} ", RED_FG, UNICODE_PROMPT),
    }
    print!("{}", RESET_FG);

    std::io::stdout().flush().expect("Could not flush.");
}

/// Reads a command from stdin and returns it.
///
/// # Panics
///
/// Panics if the `BufReader` couldn't read from stdin.
async fn read_command() -> String {
    let mut command = String::new();

    BufReader::new(io::stdin())
        .read_line(&mut command)
        .await
        .expect("Failed to read command");

    command
}
