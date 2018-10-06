use ::config::genesis::Genesis;
use ::p2p::codec::{Codec, JsonCodec, Message};
use ::p2p::thread::ThreadPool;
use ::protocol::clique::{CliqueProtocol, ProtocolHandler};
use std::{thread, time};
use std::collections::HashSet;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::iter::FromIterator;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

/// Forms a node in the blockchain.
///
/// Each node manages its own thread pool on which it starts dedicated threads
/// to listen for incoming connections. In addition, connection attempts to other
/// nodes are also spawn on the thread pool.
pub struct Node {
    /// A pool of threads maintaining tasks of this node
    /// such as listening for incoming connections,
    /// broadcasting messages or signing blocks.
    thread_pool: ThreadPool,

    /// The address of this node on which it listens for
    /// incoming messages of other nodes.
    listen_address: SocketAddr,

    /// The address of this node on which it listens
    /// for incoming RPC messages.
    rpc_listen_address: SocketAddr,

    /// A fixed set of peers to which this node should connect
    /// and broadcast messages to.
    ///
    /// As this set is used among different threads, a
    /// atomic reference counter (ARC) and a Mutex are used
    /// to avoid concurrent overwrites.
    peers: Arc<Mutex<HashSet<SocketAddr>>>,

    /// A protocol handling incoming messages to some
    /// specified behaviour.
    ///
    /// As this protocol is used among different threads, a
    /// atomic reference counter (ARC) and a Mutex are used
    /// to avoid concurrent overwrites.
    protocol: Arc<Mutex<CliqueProtocol>>,
}

impl Node {
    /// Creates a new node.
    ///
    /// - `listen_addr` The address on which the node listens for incoming messages.
    /// - `rpc_listen_address` The address on which the node listens for incoming RPC messages.
    /// - `genesis` The genesis configuration which defines the behaviour of this node.
    ///             Must be equal for all nodes which should connect to the same network.
    pub fn new(listen_address: SocketAddr, rpc_listen_address: SocketAddr, genesis: Genesis) -> Node {
        Node {
            thread_pool: ThreadPool::new(4),
            listen_address: listen_address.clone(),
            rpc_listen_address: rpc_listen_address.clone(),
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

                let mut buffer_str = String::new();
                let result = cloned_stream.read_to_string(&mut buffer_str);
                match result {
                    Ok(amount_bytes_received) => {
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

                let request = JsonCodec::decode(buffer_str);
                trace!("Got request message {:?} from {:?}", request.clone(), cloned_stream.peer_addr());
                let response = cloned_clique_protocol_handler.lock().unwrap().handle(request);
                trace!("Sending response message {:?} to {:?}", response.clone(), cloned_stream.peer_addr());
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

                let request = JsonCodec::decode(buffer_str);
                trace!("Got RPC request message {:?} from {:?}", request.clone(), stream.peer_addr());
                let needs_response = cloned_clique_protocol_handler.lock().unwrap().handle_rpc(request);

                match needs_response {
                    None => {
                        trace!("RPC does not require any response. Closing stream to {:?}", stream.peer_addr());
                        let shutdown_result = stream.shutdown(Shutdown::Both);
                        match shutdown_result {
                            Ok(()) => {}
                            // happens when the peer already closed the connection
                            Err(ref e) if e.kind() == ErrorKind::NotConnected => {}
                            Err(e) => { trace!("Could not shutdown incoming RPC connection to {:?}: {:?}", stream.peer_addr(), e) }
                        }
                    }
                    Some((response, broadcast_response)) => {
                        trace!("Sending RPC response message {:?} to {:?}", response.clone(), stream.peer_addr());
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
                        trace!("Broadcast RPC handler message {:?}", broadcast_response.clone());
                        for peer_addr in known_peers.lock().unwrap().iter() {
                            if own_address.eq(peer_addr) {
                                // avoid connecting to ourselves
                                continue;
                            }

                            let stream = TcpStream::connect(&peer_addr);

                            match stream {
                                Ok(mut stream) => {
                                    Node::handle_outgoing_connection(&mut stream, broadcast_response.clone());
                                }
                                Err(e) => {
                                    warn!("Failed to connect to {:?} due to {:?}", peer_addr, e);
                                }
                            }
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
            let protocol = Arc::clone(&self.protocol);

            match stream {
                Ok(mut stream) => {
                    trace!("Successfully connected to {:?}", stream.peer_addr());

                    // request the chain of the other node
                    let response = Node::handle_outgoing_connection(&mut stream, Message::ChainRequest);
                    match response {
                        Some(message) => {
                            protocol.lock().unwrap().handle(message);
                        },
                        None => {
                            // noop
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to connect to {:?} due to {:?}", peer_addr, e);
                }
            }
        }
    }

    pub fn sign(&mut self) {
        let clique_protocol_handler = Arc::clone(&self.protocol);
        // create a reference which we can share across threads
        let peers = Arc::clone(&self.peers);
        let own_address = self.listen_address.clone();

        self.thread_pool.execute(move || {
            let mut has_logged_signed_recently = false;

            loop {
                // start with waiting
                thread::sleep(time::Duration::from_millis(1000));

                // check whether we have to do something
                let is_leader = clique_protocol_handler.lock().unwrap().is_leader();
                let is_co_leader = clique_protocol_handler.lock().unwrap().is_co_leader();
                if ! is_leader  && ! is_co_leader {
                    // any transactions a node may have must now be reset
                    clique_protocol_handler.lock().unwrap().reset_transaction_buffer();

                    // this is just to reduce log output spamming
                    if ! has_logged_signed_recently {
                        debug!("Signed recently, must wait for others...");
                        has_logged_signed_recently = true;
                    }
                    continue;
                }
                // reset so that we get notified again...
                has_logged_signed_recently = false;

                if !clique_protocol_handler.lock().unwrap().is_block_period_over() {
                    continue;
                }

                let current_block = clique_protocol_handler.lock().unwrap().create_current_block_and_reset_transaction_buffer();

                // check whether we are a co-leader and must wait to sign the block
                // for some time...
                if clique_protocol_handler.lock().unwrap().is_co_leader() {
                    debug!("I am co-leader and therefore adding wiggle before signing block {:?}", current_block.identifier.clone());
                    // add some "wiggle" time to let leader nodes announce their blocks first
                    thread::sleep(time::Duration::from_millis(1000));
                }

                info!("Signing block {:?}", current_block.identifier.clone());
                let block_to_broadcast = clique_protocol_handler.lock().unwrap().sign(current_block);

                match block_to_broadcast {
                    None => {
                        // noop
                    }
                    Some(block) => {
                        info!("Broadcasting block {:?}", block.identifier.clone());
                        let cloned_peers = Arc::clone(&peers);
                        // broadcast new block
                        for peer_addr in cloned_peers.lock().unwrap().iter() {
                            if own_address.clone().eq(peer_addr) {
                                // avoid connecting to ourselves
                                continue;
                            }

                            let stream = TcpStream::connect(&peer_addr);

                            match stream {
                                Ok(mut stream) => {
                                    trace!("Successfully connected to {:?}", stream.peer_addr());

                                    Node::handle_outgoing_connection(&mut stream, Message::BlockPayload(block.clone()));
                                }
                                Err(e) => {
                                    warn!("Failed to connect to {:?} due to {:?}", peer_addr, e);
                                }
                            }
                        }
                    }
                }

                // Scenario is as follows:
                // - I am co-leader
                // - I receive a transaction -> add transaction to buffer
                // - I receive a block from leader containing that transaction
                // - I'm the leader now, and still have the transaction im my buffer
                // - I create a block containing that transaction again.
                // => two different blocks with the same transaction in them

                // Idea on how to prevent this here instead while receiving the block:
                // Check whether we have the contained transactions of the block already in our buffer
                // eventually, reset the transaction buffer again as we might have received new
                // transactions while we were signing. Since we might not be a a co-leader
                // or leader again -> remove them.

                // => This will cause data loss in the following case:
                // - we receive a transaction during our signing process as co-leader
                // - of course, this transaction is not contained in the block
                // - resetting the transaction buffer will also cause the new one to be removed
                // - we are missing a transaction as leader. 
            }
        });
    }

    fn handle_outgoing_connection(stream: &mut TcpStream, message: Message) -> Option<Message> {
        let request = JsonCodec::encode(message);

        stream.write_all(&request.into_bytes()).unwrap();
        stream.flush().unwrap();
        let shutdown_result = stream.shutdown(Shutdown::Write);
        match shutdown_result {
            Ok(()) => {}
            Err(e) => {
                trace!("Could not shutdown outgoing write connection: {:?}", e);

                return None;
            }
        }

        // wait for some incoming data on the same stream
        let mut buffer_str = String::new();
        let read_result = stream.try_clone().unwrap().read_to_string(&mut buffer_str);

        match read_result {
            Ok(amount_bytes_received) => {
                if 0 == amount_bytes_received {
                    trace!("No bytes received on outgoing connection. Dropping connection without response");
                    let shutdown_result = stream.shutdown(Shutdown::Both);
                    match shutdown_result {
                        Ok(()) => {}
                        Err(e) => {
                            trace!("Failed to shutdown incoming connection: {:?}", e);
                        }
                    }

                    return None;
                }
            }
            Err(e) => {
                trace!("Failed to read bytes from incoming connection: {:?}", e);

                return None;
            }
        }

        let response = JsonCodec::decode(buffer_str);
        trace!("Got response from outgoing stream: {:?}", response);

        return Some(response);
    }
}