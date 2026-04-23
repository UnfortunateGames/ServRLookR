# ServRLookR

## About ServRLookR

A rudimentary server manager, This is a simple command parser, along with a pseudo-server
objects you can interact with. You can:

- Create
- Edit
- Read from a file
- Call servers
- Run servers
- Delete servers

## Goals for me

[x] - Play around with Rust
[x] - Actually make a project with Rust
[x] - Understand why people like it
[ ] - Rust the machine
[ ] - Abandon C

## How to build

Rust has actually made this a lot easier with cargo. In contrast to CMake's complexity,
you can run this single command:

```bash
cargo run
```

This should make it run after Rust compiles my code.

## Usage

My apologies for not fully implementing the `help` command in ServRLookR. So in this
section I'll explain what commands do and what to do.

`help` it is a valid command, but I never actually finished it.

`exit` is very self-explanatory, it exits you out of the program.

`call [ID]` this will call the server with your specified *ID* if it doesn't find any,
an error will be shown, if it finds one it will either show it's downed message or
proper message.

`run [ID]` will run a server with *ID*, if it's down, it won't run, else when it's
running it will continuously get called every command run.

`stop [ID]` this will stop a currently running server along with the properties of
`call` and `run`

`shutdown [ID]` this will make a server's status DOWN, no matter if it's running,
shutting down a downed server will show an error.

`add` will prompt you to make a server. It will ask for: The name, message, error
message, and it's unique ID, respectively.

`remove [ID]` it will delete a server with *ID* off the server list.

`edit [ID]` this is the same as add, overwriting the original data, but only with the
option of also keeping the old data.

`read [PATH]` this will create a server of the file *PATH*. The file structure must be:

```none
NAME
STATUS
MESSAGE
ERROR MESSAGE
UNIQUE ID
```

No trailing new lines more than 1, else it will fail.

`wait [SECONDS]` also pretty self-explanatory, it will sleep for *SECONDS* amount of
seconds.

## Review

This made me realize that Rust is a really great process, you can make really horrible code
and you'll notice it almost immediently. It monetizes on actual good code, as there can
only be one way to solve things, or many if you include the `unsafe` block use.

And after seeing said bad code, you will be enticed to "*solve*" the code. This will result
in you making a better version than the one before. A design choice I hope that some future
languages will adopt this feature, making discomfort from "*bad*" code in general.
