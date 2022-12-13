# rshell
An open source shell written in Rust. Made for simplicity.

This shell is still in beta. It is not recommended for personal use currently.
Pull requests are allowed.
## Index

### [command.rs](src/command.rs)
High level Command spawning is handled here.

### [builtin.rs](src/builtin.rs)
This is where all the shell builtins are created.

### [lang](src/lang/mod.rs)
Scanning and parsing commands is handled in this module.

## Known Issues

### Interrupts
When typing in a command, the I/O is blocked until the Return key is pressed. This is bad
because we want to listen to SIGINTs and such:
```
~/rshell > ^C # now it's still trying to get the command and will wait until Return is pressed to send the SIGINT
```

### Specific Keys
Keys like right arrows and CTRL+L are handled as any normal terminal app does.