# R2WC - A rust 2 way communication channel.

## About
This is a simple 2 way communication channel with 2 users. The server and client.
The server runs the binary locally or bound to 0.0.0.0 for remote connections and on a port.
The server will only accpet one client at a time and remain open after a client leaves waiting for the next one.
The client can connect to a server given a host and port, and disconnects once they leave.

## How to run
1. Have rust and rust nightly installed.
2. Have cargo package manager installed.
3. Clone the repo.
4. Overide the repo to use nightly or set your default rust compiler to nightly.
5. Run `Cargo build --release`
6. This should populate a folder ./target/release with 2 binaries. r2wc-server and r2wc-client.
7. To run the server call the server give a address for local or remote(127.0.0.1 or 0.0.0.0) and a port.
8. To run the server call the client give a address and a port.
9. Notes the max message size in the ui I wrote is capped at 255 characters(old school texting style).
10. Type ":quit" or hit ctrl-L to exit.

## Using
You can also choose to use the tools I wrote to develop your own ui.
To do so do similar processes as above but in client.rs and server.rs write the respecitive ui.
