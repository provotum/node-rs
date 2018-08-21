use futures::sync::mpsc;
use rand::{Rng, ThreadRng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;
use ::p2p::codec::Msg;
use std::io;
use uuid::Uuid;

type Tx = mpsc::UnboundedSender<Msg>;

pub struct Peer {
    pub id: Uuid,
    pub addr: SocketAddr,
    pub peers: HashMap<Uuid, (Tx, SocketAddr)>,
    pub rng: Rc<RefCell<ThreadRng>>,
}

impl Peer {
    ///
    /// Broadcast the given message to all currently connected peers.
    pub fn broadcast(&self, m: String) {
        println!("broadcasting: {}", m);
        for tx_socket_addr_pair in self.peers.values() {
            let tx = &tx_socket_addr_pair.0;
            tx.send(Msg::Payload(m.clone())).expect("tx send failed");
        }
    }

    pub fn send_random(&self, m: Msg) {
        // println!("sending {:?} to random node", m);
        let high = self.peers.len();
        loop {
            for v in self.peers.values() {
                let tx = &v.0;
                if self.rng.borrow_mut().gen_range(0, high) == 0 {
                    tx.send(m).expect("tx send failed");
                    return;
                }
            }
        }
    }

    ///
    /// Add a peer if it sends us a ping request to which we
    /// reply with a corresponding pong message.
    pub fn handle_ping(&mut self, m: (Uuid, SocketAddr), tx: Tx) -> Result<(), io::Error> {
        println!("received ping: {:?}", m);
        match self.peers.get(&m.0) {
            Some(_) => {
                println!("PING ALREADY EXIST! {:?}", m);
                // TODO drop this connection
                Ok(())
            }
            None => {
                println!("ADDING NODE! {:?}", m);
                let tx2 = tx.clone();
                self.peers.insert(m.0, (tx, m.1));
                mpsc::UnboundedSender::send(&tx2, Msg::Pong((self.id, self.addr)))
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "tx failed"))
            }
        }
    }

    ///
    /// Add a peer if it sends us a pong response
    /// and we do not have a connection established to it yet.
    pub fn handle_pong(&mut self, m: (Uuid, SocketAddr), tx: Tx) -> Result<(), io::Error> {
        println!("received pong: {:?}", m);
        match self.peers.get(&m.0) {
            Some(_) => {
                println!("NODE ALREADY EXISTS {:?}", m);
                // TODO drop this connection
            }
            None => {
                println!("ADDING NODE! {:?}", m);
                self.peers.insert(m.0, (tx, m.1));
            }
        }
        Ok(())
    }

    ///
    /// Handle the given payload message.
    pub fn handle_payload(&self, m: String, tx: Tx) -> Result<(), io::Error> {
        println!("received payload: {}", m);
        Ok(())
    }
}
