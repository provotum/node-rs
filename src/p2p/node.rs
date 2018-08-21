use std::net::{SocketAddr, TcpStream, TcpListener, Shutdown};
use std::thread;
use std::collections::HashMap;
use std::io::Write;
use std::io::Read;

use ::p2p::thread::ThreadPool;


pub struct Node {
    thread_pool: ThreadPool,
    peers: HashMap<SocketAddr, TcpStream>,
}

impl Node {

    pub fn new() -> Node {

        Node {
            thread_pool: ThreadPool::new(2),
            peers: HashMap::new(),
        }
    }

    ///
    /// Start a listener on the bootstrap address
    pub fn listen(&self, bootstrap_address: SocketAddr) {
        let listener = TcpListener::bind(&bootstrap_address).unwrap();
        println!("Listening for incoming connections on {:?}", listener.local_addr());

        self.thread_pool.execute(move || {
            for stream in listener.incoming() {
                let mut cloned_stream = stream.unwrap().try_clone().unwrap();

                println!("Got incoming stream on {:?} from {:?}", cloned_stream.local_addr(), cloned_stream.peer_addr());

                thread::spawn( move || {
                    Node::handle_incoming_connection(&mut cloned_stream);
                });
            }
        });
    }

    ///
    /// Connect to a particular address
    pub fn connect(&mut self, connect_address: SocketAddr) {
        let mut stream = TcpStream::connect(&connect_address);

        match stream {
            Ok(mut stream) => {
                println!("Successfully connected to {:?}", stream.peer_addr());
                Node::handle_outgoing_connection(&mut stream);
                self.peers.insert(connect_address, stream);
            },
            Err(e) => {
                println!("Failed to connect to {:?} due to {:?}", connect_address, e);
            }
        }
    }

    fn handle_incoming_connection(stream: &mut TcpStream) {
        println!("handling incoming connection");

        let mut buffer_str = String::new();
        stream.read_to_string(&mut buffer_str);

        println!("Read string from incoming connection: {:?}", buffer_str);

        // send some data back
        let mut streamclone = stream.try_clone().unwrap();
        streamclone.write_all(&"hello from server".to_string().into_bytes());
        streamclone.flush();
        streamclone.shutdown(Shutdown::Read);


    }

    fn handle_outgoing_connection(stream: &mut TcpStream) {
        println!("handling outgoing connection");

        stream.write_all(&"hello".to_string().into_bytes());
        stream.flush();
        stream.shutdown(Shutdown::Write);

        println!("flushed written data");

        // wait for some incoming data on the same stream
        let mut buffer_str = String::new();
        stream.try_clone().unwrap().read_to_string(&mut buffer_str);

        println!("reading string from outgoing stream: {:?}", buffer_str);
    }
}