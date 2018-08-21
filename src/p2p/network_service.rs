use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::str::FromStr;
use std::result::Result;
use futures::future::{self, Either};
use futures::sync::mpsc;
use futures::Stream;
use futures::Future;
use tokio;
use tokio::runtime::Runtime;
use tokio::net::{TcpListener, TcpStream};
use p2p::peer::{Shared, Peer};
use p2p::codec::Lines;
use bytes::Bytes;
use p2p::thread::ThreadPool;

pub struct NetworkService {
    ///
    /// The original address on which a peer should be listening on
    address: &'static str,

    state: Arc<Mutex<Shared>>,

    thread_pool: ThreadPool,
}

///
/// # Network Service
///
/// Starts a blocking Tokio runtime which listens for incoming
/// connections on a particular socket. Each connection
/// is considered to be a separate peer to which the network service is connected to.
impl NetworkService {
    ///
    /// Create a new network service.
    ///
    /// * `address` - The address on which the network service should listen for incoming connections.
    pub fn new(address: &'static str) -> NetworkService {
        // Create the shared state. This is how all the peers communicate.
        //
        // The server task will hold a handle to this. For every new client, the
        // `state` handle is cloned and passed into the task that processes the
        // client connection.
        let state = Arc::new(Mutex::new(Shared::new()));

        let thread_pool = ThreadPool::new(4);

        NetworkService {
            address,
            state,
            thread_pool,
        }
    }

    ///
    /// Start to listen on the configured port.
    pub fn listen(&self) {
        let addr = self.address.parse().unwrap();

        // Bind a TCP listener to the socket address.
        //
        // Note that this is the Tokio TcpListener, which is fully async.
        let listener = TcpListener::bind(&addr).unwrap();
        let actual_addr = &listener.local_addr();

        // Create the shared state. This is how all the peers communicate.
        //
        // The server task will hold a handle to this. For every new client, the
        // `state` handle is cloned and passed into the task that processes the
        // client connection.
        let state = self.state.clone();

        // The server task asynchronously iterates over and processes each
        // incoming connection.
        let server = listener
            .incoming()
            .for_each(move |socket| {
                // Spawn a task to process the connection
                process(socket, state.clone());
                Ok(())
            })
            .map_err(|err| {
                // All tasks must have an `Error` type of `()`. This forces error
                // handling and helps avoid silencing failures.
                //
                // In our example, we are only going to log the error to STDOUT.
                println!("accept error = {:?}", err);
            });

        // find the address to which the listener was truly bound
        println!("server running on {:?}", actual_addr);

        // Start the Tokio runtime.
        //
        // The Tokio is a pre-configured "out of the box" runtime for building
        // asynchronous applications. It includes both a reactor and a task
        // scheduler. This means applications are multithreaded by default.
        //
        // This function blocks until the runtime reaches an idle state. Idle is
        // defined as all spawned tasks have completed and all I/O resources (TCP
        // sockets in our case) have been dropped.
        //
        // In our example, we have not defined a shutdown strategy, so this will
        // block until `ctrl-c` is pressed at the terminal.
        self.thread_pool.execute(move || {
            tokio::run(server);
        });
    }

    pub fn send(&self) {
        let peers = &self.state.lock().unwrap().peers;

        // Now, send the line to all other peers
        for (addr, tx) in peers {
            // The send only fails if the rx half has been dropped,
            // however this is impossible as the `tx` half will be
            // removed from the map before the `rx` is dropped.
            let result = tx.unbounded_send(Bytes::from("Hello from send"));

            match result {
                Result::Ok(()) => println!("Sent message to peer at address {:?}", addr),
                Result::Err(e) => println!("Failed to send message to peer at address {:?}. Error was {:?}", addr, e),
            }
        }
    }

    pub fn connect(&self, peer_address: &str) {
        let socket_peer_address = SocketAddr::from_str(peer_address).unwrap();

        let connect_future = TcpStream::connect(&socket_peer_address)
            .map_err(|e| {
                println!("connection failed error = {:?}", e);
            })
            .and_then(|stream| {
                let addr = stream.peer_addr().unwrap();
                //let (tx, rx) = mpsc::unbounded();

                println!("Adding peer at address {:?}", addr);

                // add peer to our list
                //self.state.lock().unwrap().peers.insert(addr, tx);

                future::ok(())
            })
            .map_err(|e| {
                println!("Failed to connect. Error was {:?}", e)
            });

        tokio::spawn(connect_future);
    }
}

///
/// Spawn a task to manage the socket.
///
/// This will read the first line from the socket to identify the client, then
/// add the client to the set of connected peers in the chat service.
fn process(socket: TcpStream, state: Arc<Mutex<Shared>>) {
    // Wrap the socket with the `Lines` codec that we wrote above.
    //
    // By doing this, we can operate at the line level instead of doing raw byte
    // manipulation.
    let lines = Lines::new(socket);

    // The first line is treated as the client's name. The client is not added
    // to the set of connected peers until this line is received.
    //
    // We use the `into_future` combinator to extract the first item from the
    // lines stream. `into_future` takes a `Stream` and converts it to a future
    // of `(first, rest)` where `rest` is the original stream instance.
    let connection = lines.into_future()
        // `into_future` doesn't have the right error type, so map the error to
        // make it work.
        .map_err(|(e, _)| e)
        // Process the first received line as the client's name.
        .and_then(|(name, lines)| {
            // If `name` is `None`, then the client disconnected without
            // actually sending a line of data.
            //
            // Since the connection is closed, there is no further work that we
            // need to do. So, we just terminate processing by returning
            // `future::ok()`.
            //
            // The problem is that only a single future type can be returned
            // from a combinator closure, but we want to return both
            // `future::ok()` and `Peer` (below).
            //
            // This is a common problem, so the `futures` crate solves this by
            // providing the `Either` helper enum that allows creating a single
            // return type that covers two concrete future types.
            let name = match name {
                Some(name) => name,
                None => {
                    // The remote client closed the connection without sending
                    // any data.
                    return Either::A(future::ok(()));
                }
            };

            println!("`{:?}` is joining the chat", name);

            // Create the peer.
            //
            // This is also a future that processes the connection, only
            // completing when the socket closes.
            let peer = Peer::new(
                name,
                state,
                lines);

            // Wrap `peer` with `Either::B` to make the return type fit.
            Either::B(peer)
        })
        // Task futures have an error of type `()`, this ensures we handle the
        // error. We do this by printing the error to STDOUT.
        .map_err(|e| {
            println!("connection error = {:?}", e);
        });

    // Spawn the task. Internally, this submits the task to a thread pool.
    tokio::spawn(connection);
}