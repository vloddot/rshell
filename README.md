# rshell
An open source shell written in Rust. Made for simplicity.

This shell is still in beta. It is not recommended for personal use currently.
Pull requests are allowed.

## Known Issues

### Interrupts
When typing in a command, the I/O is blocked until the Return key is pressed. This is bad
because we want to listen to SIGINTs and such:
```
~/rshell > ^C # now it's still trying to get the command and will wait until Return is pressed to send the SIGINT
```

### Specific Keys
Keys like right arrows and CTRL+L are handled as any normal terminal app does.

## Unsupported Features
- or (||) syntax
- pipe (|) syntax
- semicolon (;) syntax
- block ({ ls }) syntax

