# rshell
An open source shell written in Rust. Made for simplicity.

This shell is still in beta. It is not recommended for personal use currently.
Pull requests are allowed.
## Index
### [command.rs](src/command.rs)
This is where all the text parsing is done and is created using [nom](https://crates.io/crates/nom).
### [command.rs](src/builtin.rs)
This is where all the shell builtins are created.
### [lib.rs](src/lib.rs)
This is where shell command aliases are defined to be used in multiple files.
