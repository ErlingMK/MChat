@Baptiste

- Stand in "ChatEngine" folder and run "cargo run --bin server" and then "cargo run --bin client" in two separate terminals.

## Message protocol:

message type <-- u8 (1 byte)
sender id <-- u32 (4 bytes)
receiver id <-- u32 (4 bytes)
date <-- (20 bytes)
content-length <-- u64 (8 bytes)
data <-- (as many bytes as specified in content-length)
