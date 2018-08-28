use std::net::{SocketAddr, TcpStream, TcpListener, Shutdown};
use std::collections::HashSet;
use std::io::Write;
use std::io::Read;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::iter::FromIterator;

use ::p2p::thread::ThreadPool;
use ::p2p::codec::{Codec, JsonCodec, Message};
use ::protocol::clique::{CliqueProtocol, ProtocolHandler};
use ::config::genesis::Genesis;
use ::chain::chain::Chain;

/// Forms a node in the blockchain.
///
/// Each node manages its own thread pool on which it starts dedicated threads
/// to listen for incoming connections. In addition, connection attempts to other
/// nodes are also spawn on the thread pool.
///
/// Further, a node maintains a list of peers it has previously connected to or from
/// which connection attempts have been made.
pub struct Node {
    thread_pool: ThreadPool,

    /// A cache of peers we have connected to
    /// or from which we received connections.
    peers: Arc<Mutex<HashSet<SocketAddr>>>,

    chain: Arc<Mutex<Chain>>,
}

impl Node {

    /// Creates a new node with the provided genesis configuration.
    pub fn new(genesis: Genesis) -> Node {
        Node {
            // TODO: increase thread pool size for creating more connections
            thread_pool: ThreadPool::new(2),

            // TODO: explain why we need an atomic reference and a mutex here
            peers: Arc::new(Mutex::new(HashSet::from_iter(genesis.sealer.iter().cloned()))),

            chain: Arc::new(Mutex::new(Chain::new(genesis))),
        }
    }

    ///
    /// Start a listener on the bootstrap address
    pub fn listen(&self, bootstrap_address: SocketAddr) {
        let listener = TcpListener::bind(&bootstrap_address).unwrap();
        info!("Listening for incoming connections on {:?}", listener.local_addr());

        self.thread_pool.execute(move || {
            for stream in listener.incoming() {
                let mut cloned_stream = stream.unwrap().try_clone().unwrap();

                trace!("Got incoming stream on {:?} from {:?}", cloned_stream.local_addr(), cloned_stream.peer_addr());

                // TODO: Drop connection if not from authorized node

                Node::handle_incoming_connection(&mut cloned_stream);
            }
        });
    }

    ///
    /// Connect to a particular address
    pub fn connect(&mut self) {

        // create a reference which we can share across threads
        let peers = Arc::clone(&self.peers);

        for peer_addr in peers.lock().unwrap().iter() {
            let stream = TcpStream::connect(&peer_addr);

            match stream {
                Ok(mut stream) => {
                    trace!("Successfully connected to {:?}", stream.peer_addr());

                    Node::handle_outgoing_connection(&mut stream);
                }
                Err(e) => {
                    warn!("Failed to connect to {:?} due to {:?}", peer_addr, e);
                }
            }
        }
    }

    /// Read all bytes until EOF (when underlying socket is closed) from the given stream
    /// and return a message back to the incoming sender.
    /// Then close the stream in order to signal EOF for the receiving node.
    fn handle_incoming_connection(stream: &mut TcpStream) {
        trace!("handling incoming connection");

        let mut buffer_str = String::new();
        let result = stream.read_to_string(&mut buffer_str);
        match result {
            Ok(amount_bytes_received) => {
                trace!("Read {:?} bytes from incoming connection", amount_bytes_received);

                if 0 == amount_bytes_received {
                    trace!("No bytes received on incoming connection. Dropping connection without response");
                    let shutdown_result = stream.shutdown(Shutdown::Both);
                    match shutdown_result {
                        Ok(()) => {}
                        Err(e) => {
                            trace!("Failed to shutdown incoming connection: {:?}", e);
                        }
                    }

                    return;
                }
            }
            Err(e) => {
                trace!("Failed to read bytes from incoming connection: {:?}", e);

                return;
            }
        }

        trace!("Read string from incoming connection: {:?}. Converting into message", buffer_str);
        let request = JsonCodec::decode(buffer_str);
        let response = CliqueProtocol::handle(request);
        let encoded_response = JsonCodec::encode(response);

        // send some data back
        let mut stream_clone = stream.try_clone().unwrap();
        stream_clone.write_all(&encoded_response.into_bytes()).unwrap();
        stream_clone.flush().unwrap();

        let shutdown_result = stream_clone.shutdown(Shutdown::Read);
        match shutdown_result {
            Ok(()) => {}
            // happens when the peer already closed the connection
            Err(ref e) if e.kind() == ErrorKind::NotConnected => {}
            Err(e) => { trace!("Could not shutdown incoming connection: {:?}", e) }
        }
    }

    fn handle_outgoing_connection(stream: &mut TcpStream) {
        trace!("handling outgoing connection");

        let request = JsonCodec::encode(Message::Pong);

        stream.write_all(&request.into_bytes()).unwrap();
        stream.flush().unwrap();
        let shutdown_result = stream.shutdown(Shutdown::Write);
        match shutdown_result {
            Ok(()) => {}
            Err(e) => {
                trace!("Could not shutdown outgoing write connection: {:?}", e);

                return;
            }
        }

        trace!("flushed written data");

        // wait for some incoming data on the same stream
        let mut buffer_str = String::new();
        let read_result = stream.try_clone().unwrap().read_to_string(&mut buffer_str);

        match read_result {
            Ok(amount_bytes_received) => {
                trace!("Read {:?} bytes from outgoing connection", amount_bytes_received);

                if 0 == amount_bytes_received {
                    trace!("No bytes received on outgoing connection. Dropping connection without response");
                    let shutdown_result = stream.shutdown(Shutdown::Both);
                    match shutdown_result {
                        Ok(()) => {}
                        Err(e) => {
                            trace!("Failed to shutdown incoming connection: {:?}", e);
                        }
                    }

                    return;
                }
            }
            Err(e) => {
                trace!("Failed to read bytes from incoming connection: {:?}", e);

                return;
            }
        }

        let response = JsonCodec::decode(buffer_str);

        trace!("Got response from outgoing stream: {:?}", response);
    }
}