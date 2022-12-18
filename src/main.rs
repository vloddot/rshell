use rshell::{
    Command, GREEN_FG_COLOR, PREVIOUS_EXIT_CODE, PROMPT_UNICODE, RED_FG_COLOR, RESET_FG_COLOR,
    RSHELL_RC, RSHISTORY, SIGINT_EXIT_CODE,
};

use signal_hook::{consts::SIGINT, iterator::Signals};

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use tokio::{
    fs::OpenOptions,
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    // get home directory
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => Some(dir),
        Err(_) => None,
    };

    let home_dir = home_dir.map(PathBuf::from);

    // open history file to store commands into history
    let mut history = if let Some(home_dir) = home_dir.clone() {
        let history = home_dir.join(RSHISTORY);

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

    init(home_dir.as_deref()).await;

    let mut signals = Signals::new([SIGINT])?;

    'main_loop: loop {
        for signal in signals.pending() {
            if let SIGINT = signal {
                *PREVIOUS_EXIT_CODE.lock().await = SIGINT_EXIT_CODE;
                continue 'main_loop;
            }
        }

        let current_dir = std::env::current_dir()?;

        print_prompt(home_dir.as_deref(), &current_dir).await;
        std::io::stdout().flush()?;

        let command = read_command().await;

        // write command into history
        if let Some(ref mut history) = history {
            history.write_all(command.as_bytes()).await?;
        }

        let (code, _) = match Command::run(&command).await {
            (Ok(code), duration) => (code, duration),
            (Err(error), duration) => {
                rshell::error!("{error}");
                (error.kind().code(), duration)
            }
        };

        *PREVIOUS_EXIT_CODE.lock().await = code;
    }
}

async fn init(home_dir: Option<&Path>) {
    if let Some(home_dir) = home_dir {
        let shellrc = home_dir.join(RSHELL_RC);

        let shellrc = match tokio::fs::read(shellrc).await {
            Ok(rc) => Some(rc),
            Err(_) => None,
        };

        if let Some(shellrc) = shellrc {
            let mut lines = shellrc.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if let (Err(_), _) = Command::run(&line).await {
                    return;
                }
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
///
/// # Examples
///
/// ```no_run
/// print_prompt(0, "/Users/any", "/Users/any/sandbox") // prints "~/sandbox ❯ " with the ❯ character green
/// print_prompt(42069, "/Users/any", "/Users/any/sandbox") // prints "~/sandbox ❯ " with the ❯ character red
/// ```
async fn print_prompt(home_dir: Option<&Path>, current_dir: &Path) {
    // print the current directory
    if let Some(home_dir) = home_dir {
        print!(
            "{} ",
            current_dir
                .display()
                .to_string()
                .replace(&home_dir.display().to_string(), "~")
        );
    } else {
        print!("{} ", current_dir.display());
    }

    // print the prompt and reset the color
    print!(
        "{}{}{} ",
        match *PREVIOUS_EXIT_CODE.lock().await {
            0 => GREEN_FG_COLOR.to_string(),
            _ => RED_FG_COLOR.to_string(),
        },
        PROMPT_UNICODE,
        RESET_FG_COLOR
    );
}

/// Reads a command from stdin and returns it.
///
/// # Panics
///
/// Panics if the [`BufReader`] couldn't read from stdin.
///
/// # Exits
///
/// Exits the program if the character read is an EOF character (CTRL+D).
async fn read_command() -> String {
    let mut command = String::new();

    let bytes = BufReader::new(io::stdin())
        .read_line(&mut command)
        .await
        .expect("Failed to read line");

    // EOF reached.
    if bytes == 0 {
        println!();
        std::process::exit(0);
    }

    command
}
