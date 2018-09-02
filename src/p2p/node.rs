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

    listen_address: SocketAddr,

    rpc_listen_address: SocketAddr,

    /// A cache of peers we have connected to
    /// or from which we received connections.
    peers: Arc<Mutex<HashSet<SocketAddr>>>,

    protocol: Arc<Mutex<CliqueProtocol>>,
}

impl Node {

    /// Creates a new node with the provided genesis configuration.
    pub fn new(listen_address: SocketAddr, rpc_listen_address: SocketAddr, genesis: Genesis) -> Node {
        Node {
            // TODO: increase thread pool size for creating more connections
            thread_pool: ThreadPool::new(3),

            listen_address: listen_address.clone(),

            rpc_listen_address: rpc_listen_address.clone(),

            // TODO: explain why we need an atomic reference and a mutex here
            peers: Arc::new(Mutex::new(HashSet::from_iter(genesis.sealer.iter().cloned()))),

            protocol: Arc::new(Mutex::new(CliqueProtocol::new(listen_address, genesis))),
        }
    }

    /// Start a listener on the bootstrap address.
    ///
    /// Read all bytes until EOF (when underlying socket is closed) from the given stream
    /// and return a message back to the incoming sender.
    /// Then close the stream in order to signal EOF for the receiving node.
    pub fn listen(&self) {
        let listener = TcpListener::bind(&self.listen_address).unwrap();
        info!("Listening for incoming connections on {:?}", listener.local_addr());
        // clone the mutex of the chain
        let clique_protocol_handler = Arc::clone(&self.protocol);

        self.thread_pool.execute(move || {
            for stream in listener.incoming() {
                let mut cloned_stream = stream.unwrap().try_clone().unwrap();
                let cloned_clique_protocol_handler = Arc::clone(&clique_protocol_handler);

                trace!("Got incoming stream on {:?} from {:?}", cloned_stream.local_addr(), cloned_stream.peer_addr());

                // TODO: Drop connection if not from authorized node

                trace!("handling incoming node connection");

                let mut buffer_str = String::new();
                let result = cloned_stream.read_to_string(&mut buffer_str);
                match result {
                    Ok(amount_bytes_received) => {
                        trace!("Read {:?} bytes from incoming connection", amount_bytes_received);

                        if 0 == amount_bytes_received {
                            trace!("No bytes received on incoming connection. Dropping connection without response");
                            let shutdown_result = cloned_stream.shutdown(Shutdown::Both);
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
                let response = cloned_clique_protocol_handler.lock().unwrap().handle(request);
                let encoded_response = JsonCodec::encode(response);

                // send some data back
                let mut stream_clone = cloned_stream.try_clone().unwrap();
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
        });
    }

    pub fn listen_rpc(&self) {
        let rpc_listener = TcpListener::bind(&self.rpc_listen_address).unwrap();
        info!("Listening for incoming RPC connections on {:?}", rpc_listener.local_addr());

        let cloned_clique_protocol_handler = Arc::clone(&self.protocol);
        let known_peers = Arc::clone(&self.peers);
        let own_address = self.listen_address.clone();

        self.thread_pool.execute(move || {
            for incoming_stream in rpc_listener.incoming() {
                let mut stream = incoming_stream.unwrap();

                trace!("Handling incoming RPC stream on {:?} from {:?}", stream.local_addr(), stream.peer_addr());

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
                let (response, broadcast_response) = cloned_clique_protocol_handler.lock().unwrap().handle_rpc(request);
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

                // now broadcast the message to all other peers
                for peer_addr in known_peers.lock().unwrap().iter() {
                    if own_address.eq(peer_addr) {
                        // avoid connecting to ourselves
                        continue;
                    }

                    let stream = TcpStream::connect(&peer_addr);

                    match stream {
                        Ok(mut stream) => {
                            trace!("Sending to {:?}", stream.peer_addr());

                            Node::handle_outgoing_connection(&mut stream, broadcast_response.clone());
                        }
                        Err(e) => {
                            warn!("Failed to connect to {:?} due to {:?}", peer_addr, e);
                        }
                    }
                }
            }
        });
    }

    /// Send a Ping message to all known peers
    pub fn connect(&mut self) {

        // create a reference which we can share across threads
        let peers = Arc::clone(&self.peers);

        for peer_addr in peers.lock().unwrap().iter() {
            if self.listen_address.eq(peer_addr) {
                // avoid connecting to ourselves
                continue;
            }

            let stream = TcpStream::connect(&peer_addr);

            match stream {
                Ok(mut stream) => {
                    trace!("Successfully connected to {:?}", stream.peer_addr());

                    Node::handle_outgoing_connection(&mut stream, Message::Ping);
                }
                Err(e) => {
                    warn!("Failed to connect to {:?} due to {:?}", peer_addr, e);
                }
            }
        }
    }

    fn handle_outgoing_connection(stream: &mut TcpStream, message: Message) {
        trace!("handling outgoing connection");

        let request = JsonCodec::encode(message);

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