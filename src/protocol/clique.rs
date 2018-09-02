use ::chain::block::Block;
use ::chain::chain::Chain;
use ::chain::transaction::Transaction;
use ::config::genesis::Genesis;
use ::p2p::codec::Message;
use std::vec::Vec;
use std::net::{SocketAddr};

pub trait ProtocolHandler {
    fn handle(&mut self, message: Message) -> Message;
}

pub struct CliqueProtocol {
    transactions: Vec<Transaction>,
    signer_index: usize,
    signer_count: usize,
    signer_limit: usize,
    chain: Chain,
}

impl CliqueProtocol {
    pub fn new(own_address: SocketAddr, genesis: Genesis) -> Self {
        let own_signer_index = genesis.sealer.clone()
            .iter()
            .enumerate()
            .find(|&element| element.1.eq(&own_address.clone()))
            .expect("Could not find own socket address in sealers of genesis configuration")
            .0;
        trace!("Found own sealer index to be {} for own listening address {} in genesis configuration", own_signer_index.clone(), own_address.clone());

        let own_signer_count = genesis.sealer.len().clone();
        trace!("Found a total of {} sealer in genesis configuration", own_signer_count.clone());

        CliqueProtocol {
            transactions: vec![],
            signer_index: own_signer_index,
            signer_count: own_signer_count,
            signer_limit: genesis.clique.signer_limit,
            chain: Chain::new(genesis),
        }
    }

    fn is_leader(&self) -> bool {
        let current_block_number = self.chain.get_current_block_number();
        let expected_leader_index = current_block_number % self.signer_count;
        let am_i_leader = self.signer_index == expected_leader_index;

        info!("Current block number is {}, expected leader is {}. Am I the leader? {}", current_block_number, expected_leader_index, am_i_leader);

        am_i_leader
    }

    fn is_co_leader(&self) -> bool {
        let current_block_number = self.chain.get_current_block_number();

        let lower_leader_index_bound = (current_block_number % self.signer_count) + 1;
        let upper_leader_index_bound = lower_leader_index_bound + self.signer_limit;

        let am_i_co_leader = self.signer_limit >= lower_leader_index_bound && self.signer_limit <= upper_leader_index_bound;

        info!("Current block number is {}, leader idx bound is [{}..{}]. Am I co-leader? {}", current_block_number, lower_leader_index_bound, upper_leader_index_bound, am_i_co_leader);

        am_i_co_leader
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

                Message::TransactionAccept
            },
            Message::TransactionAccept => unimplemented!("Not yet implemented: Block accept"),
            Message::BlockRequest(_) => unimplemented!("Not yet implemented: Return block requested"),
            Message::BlockPayload(_) => unimplemented!("Not yet implemented: Add block to chain"),
            Message::BlockAccept => unimplemented!("Not yet implemented: Block accept"),
        }
    }
}