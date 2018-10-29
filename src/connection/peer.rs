use std::net::{TcpListener, TcpStream};

/// A Peer which holds the Stream to conenct them by and who it is.
pub struct Peer {
    stream: TcpStream,
    who: String,
}

impl Peer {
    /// Creates a new Option<Peer>, by grabbing one from given server.
    ///
    /// # Arguments
    /// * `server` - A &TcpListener so we can accept a connection.
    ///
    /// # Returns
    ///  `Option<Peer>` - A peer if one was grabbed from the server TcpListener.
    pub fn get_client(server: &TcpListener) -> Option<Peer> {
        if let Ok((stream, addr)) = server.accept() {
            stream
                .set_nonblocking(true)
                .expect("failed to initiate non-blocking");
            return Some(Peer {
                stream: stream,
                who: format!("{}", addr),
            });
        }

        return None;
    }

    /// Creates a new Peer, given a TcpStream and String.
    ///
    /// # Arguments
    /// * `stream` - A TcpStream to store to communicate witht he peer.
    /// * `who` - A String that represents who the peer may be.
    ///
    /// # Returns
    ///  `Peer` - the newly created a peer.
    pub fn new(stream: TcpStream, who: String) -> Peer {
        return Peer {
            stream: stream,
            who: who,
        };
    }

    /// Accessor method for a Peer's TcpStream.
    ///
    /// Called on a Peer.
    ///
    /// # Returns
    ///  `&TcpStream` - the Peer's TcpStream.
    pub fn stream(&self) -> &TcpStream {
        return &self.stream;
    }

    /// Accessor method for a Peer's TcpStream.
    ///
    /// Called on a Peer.
    ///
    /// # Returns
    ///  `&String` - the Peer's identifier.
    pub fn who(&self) -> &String {
        return &self.who;
    }
}

/// Clones a Peer by returning a new instance of one.
impl Clone for Peer {
    fn clone(&self) -> Peer {
        Peer {
            stream: self
                .stream()
                .try_clone()
                .expect("Could not clone TcpStream."),
            who: self.who().clone(),
        }
    }
}
