use std::env;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};

extern crate stopwatch;
use stopwatch::Stopwatch;

mod peer;
pub use self::peer::Peer;

/// A Connection which stores information about a connection through a TcpListener.
///
/// # Fields
/// `msg_size` - Stores message size for a Conenction, that is how many characters it reads and writes.
/// `taken` - more for server side, a mutex safe bool so that we can safely check whether a server only has one client.
/// `peer` - A Option<peer> currently representing the person we are talking to or not.
/// `sender` - String channel for sending messages.
/// `receiver` - A mutex safe String channel for receiving messages.
pub struct Connection {
    msg_size: usize,
    pub taken: Option<bool>,
    peer: Option<Peer>,
}

/// Called by server to arg check for server port.
///
/// # Returns
/// `String` - the port.
pub fn set_port() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Error: Usage ./r2wc-server [addr] [port]");
        ::std::process::exit(0x0100);
    }

    // Can always call unwrap safely here because otherwise case is handled above.
    return format!("{}:{}", args.get(1).unwrap(), args.get(2).unwrap());
}

/// Called by server to create a TcpListener and set nonblocking mode.
///
/// # Returns
/// `TcpListener` - a server side conenction of a TcpListener.
pub fn create_server() -> TcpListener {
    let server = TcpListener::bind(&set_port()).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    return server;
}

/// Called by client to arg check for server hostname and port.
///
/// # Returns
/// `String` - the hostname and port configured.
pub fn set_server_port() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Error: Usage ./r2wc-client [host] [port]");
        ::std::process::exit(0x0100);
    }

    // Can always call unwrap safely here because otherwise case is handled above.
    return format!("{}:{}", args.get(1).unwrap(), args.get(2).unwrap());
}

/// Called by client to create a TcpStream and set nonblocking mode.
///
/// # Returns
/// `TcpStream` - a client side connection of a TcpListener.
pub fn connect_server() -> TcpStream {
    let stream = TcpStream::connect(&set_server_port()).expect("Stream failed to connect");
    stream
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    return stream;
}

impl Connection {
    pub fn get_peer(&self) -> Option<Peer> {
        return self.peer.clone();
    }

    /// Creates a new connection given arguments.
    ///
    /// # Arguments
    /// * `msg_size` - A usize that represents how large the messages can be.
    /// * `taken` - A option bool to represent a server connection being taken.
    ///
    /// # Returns
    ///  `Connection` - the newly created connection.
    pub fn new_connection(msg_size: usize, taken: Option<bool>) -> Connection {
        return Connection {
            msg_size: msg_size,
            taken: taken,
            peer: None,
        };
    }

    /// Creates a new pre-configured server connection given an argument.
    ///
    /// # Arguments
    /// * `msg_size` - A usize that represents how large the messages can be.
    ///
    /// # Returns
    ///  `Connection` - the newly created connection.
    pub fn new_server_connection(msg_size: usize) -> (Connection, TcpListener) {
        return (
            Connection {
                msg_size: msg_size,
                taken: Some(false),
                peer: None,
            },
            create_server(),
        );
    }

    /// Creates a new pre-configured client connection given an argument.
    ///
    /// # Arguments
    /// * `msg_size` - A usize that represents how large the messages can be.
    ///
    /// # Returns
    ///  `Connection` - the newly created connection.
    pub fn new_client_connection(msg_size: usize) -> Connection {
        return Connection {
            msg_size: msg_size,
            taken: None,
            peer: Some(Peer::new(connect_server(), String::from("Server"))),
        };
    }

    /// Turns waiting for a client into a blocking call until a Client connects.
    ///
    /// Called on a connection and mutates it to have the Client as it's peer.
    ///
    /// # Arguments
    /// * `server` - A &TcpListener so we can wait on that server for a client.
    pub fn await_client(&mut self, server: &TcpListener) {
        loop {
            match Peer::get_client(&server) {
                Some(c) => {
                    self.peer = Some(c);
                    self.taken = Some(true);
                    return;
                }
                None => continue,
            }
        }
    }

    /// Turns waiting for a client call into a blocking call for 100ms.
    ///
    /// Called on a connection and mutates it to have the Client as it's peer.
    ///
    /// # Arguments
    /// * `server` - A &TcpListener so we can wait on that server for a client.
    pub fn await_client_timeout(&mut self, server: &TcpListener) {
        let start = Stopwatch::start_new();

        while start.elapsed_ms() < 100 {
            match Peer::get_client(&server) {
                Some(c) => {
                    self.peer = Some(c);
                    self.taken = Some(true);
                    return;
                }
                None => continue,
            }
        }
    }

    /// Rejects other clients from connecting our server.
    ///
    /// Called on a connection, for convience also returns the server taken status and the rejected client if one exists.
    ///
    /// # Arguments
    /// * `server` - A &TcpListener so we can wait on that server for a client.
    ///
    /// # Returns
    /// `(bool, Option<Peer>)` - The server's status of taken by a client, and the possible rejected client.
    pub fn reject_other_clients(&self, server: &TcpListener) -> (bool, Option<Peer>) {
        match self.taken {
            Some(t) => {
                if t {
                    return (true, Peer::get_client(server));
                } else {
                    return (false, None);
                }
            }
            None => return (false, None),
        }
    }

    /// Sends a message to the peer.
    ///
    /// Called on a connection, returns a string message sent or if peer is empty.
    ///
    /// # Arguments
    /// * `msg` - A String of the message to send to the peer.
    ///
    /// # Returns
    /// `(String, Stopwatch)` - Message Sent along with a format or Empty if there was no current peer.
    pub fn send_message(&self, msg: String) -> (String, Stopwatch) {
        match self.peer.clone() {
            Some(peer) => {
                let mut writer = BufWriter::new(peer.stream());

                let mut buff = msg.clone().into_bytes();
                buff.resize(self.msg_size, 0);
                let sent_time = Stopwatch::start_new();
                writer.write_all(&buff).expect("Writing to socket failed.");
                return (format!("Message sent {:?}", buff), sent_time);
            }
            None => return (String::from("Empty"), Stopwatch::start_new()),
        }
    }

    /// Receives a peer's message.
    ///
    /// Called on a connection, returns a string message, mutates conenction on client disconnect.
    ///
    /// # Returns
    /// `String` - The received messaged, blocked, disconencted, or empty depending on the situation.
    pub fn receive_message(&mut self) -> String {
        let mut buff = vec![0; self.msg_size];
        let pos_peer = &self.peer.clone();

        match pos_peer {
            Some(peer) => {
                let mut reader = BufReader::new(peer.stream());

                match reader.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        return msg;
                    }

                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                        return String::from("Blocked")
                    }

                    Err(_) => {
                        self.taken = Some(false);
                        self.peer = None;
                        return String::from("Disconnected");
                    }
                }
            }
            None => return String::from("Empty"),
        }
    }

    /// Sends a message to the peer that the peer's message has been received.
    ///
    /// Called on a connection.
    pub fn notify_message_received(&self) {
        self.send_message(String::from("Message Received."));
    }
}

impl Clone for Connection {
    fn clone(&self) -> Connection {
        Connection {
            msg_size: self.msg_size.clone(),
            taken: self.taken.clone(),
            peer: self.peer.clone(),
        }
    }
}
