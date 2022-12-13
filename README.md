# rshell
An open source shell written in Rust. Made for simplicity.

This shell is still in beta. It is not recommended for personal use currently.
Pull requests are allowed.
## Index
### [command.rs](src/command.rs)
This is where all the text parsing is done and is created using [nom](https://crates.io/crates/nom). Command spawning is also handled here.
### [builtin.rs](src/builtin.rs)
This is where all the shell builtins are created.
### [lib.rs](src/lib.rs)
This is where shell command aliases are defined to be used in multiple files.

## Known Issues
### Interrupts
When typing in a command, the I/O is blocked until the Return key is pressed. This is bad
because we want to listen to SIGINTs and such:
```
~/rshell > ^C # now it's still trying to get the command and will wait until Return is pressed to send the SIGINT
```
