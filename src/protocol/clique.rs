use ::chain::block::Block;
use ::chain::chain::Chain;
use ::chain::transaction::Transaction;
use ::config::genesis::Genesis;
use ::p2p::codec::Message;
use std::vec::Vec;

pub trait ProtocolHandler {
    fn handle(&mut self, message: Message) -> Message;
}

pub struct CliqueProtocol {
    transactions: Vec<Transaction>,
    chain: Chain,
}

impl CliqueProtocol {
    pub fn new(genesis: Genesis) -> Self {
        CliqueProtocol {
            transactions: vec![],
            chain: Chain::new(genesis),
        }
    }
}

impl ProtocolHandler for CliqueProtocol {
    fn handle(&mut self, message: Message) -> Message {
        // TODO: actually handle message

        match message {
            Message::None => Message::None,
            Message::Ping => Message::Pong,
            Message::Pong => Message::None,
            Message::TransactionPayload(transaction) => {
                self.transactions.push(transaction);
                
                Message::None
            },
            Message::BlockRequest(_) => unimplemented!("Not yet implemented: Return block requested"),
            Message::BlockPayload(_) => unimplemented!("Not yet implemented: Add block to chain"),
        }
    }
}