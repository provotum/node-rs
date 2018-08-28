use std::collections::{HashSet, HashMap};
use std::vec::Vec;
use bincode;
use sha1::Sha1;

use ::chain::block::Block;
use ::chain::transaction::Transaction;
use ::config::genesis::Genesis;
use ::chain::chain_visitor::ChainVisitor;


pub struct Chain {
    /// the hash of the genesis configuration
    genesis_configuration_hash: String,
    /// all known blocks
    blocks: HashMap<String, Block>,
    /// a matrix creating the relation between blocks
    adjacent_matrix: HashMap<String, Vec<String>>
}

impl Chain {

    pub fn new(genesis: Genesis) -> Self {
        // create the genesis block with an empty hash and no transactions
        let genesis_block_hash = String::new();
        let trxs: Vec<Transaction> = vec![];
        let genesis_block: Block = Block::new(genesis_block_hash.clone(), trxs);

        let mut blocks = HashMap::new();
        blocks.insert(genesis_block_hash.clone(), genesis_block.clone());

        // Add an entry for the genesis block in the adjacent matrix,
        // i.e. initialize children of the genesis block as an empty vector.
        let mut adjacent_matrix: HashMap<String, Vec<String>> = HashMap::new();
        adjacent_matrix.insert(genesis_block_hash, vec![]);

        // Create a sha1 digest of the genesis configuration so that we can later
        // ensure, that we only accept blocks from a chain with the same configuration.
        let bytes = bincode::serialize(&genesis).unwrap();
        let digest: String = Sha1::from(bytes).hexdigest();

        Chain {
            genesis_configuration_hash: digest,
            blocks,
            adjacent_matrix: HashMap::new()
        }
    }

    /// Returns true, if the parent of the given block exists, false otherwise.
    pub fn has_parent_of_block(self, block: Block) -> bool {
        let parent_block = self.adjacent_matrix.get(&block.previous);

        parent_block.is_some()
    }

    /// Add the block to the currently longest path of the chain.
    pub fn add_block(&mut self, block: Block) {
        // TODO: find longest path

        let parent_block = self.adjacent_matrix.get(&block.previous);
        assert!(parent_block.is_some(), "Parent block with hash {:?} of block {:?} does not exist. Have you forgot to call has_parent_of_block before adding it?", block.previous.clone(), block.clone());


        self.blocks.insert(block.current.clone(), block);
    }

    /// Visit the chain starting from the node identified by `start_node_hash`
    /// and traversing all its children.
    ///
    /// - `start_node_hash`: The hash of the node where the chain should be first visited from.
    /// - `chain_visitor`: The visitor to use.
    pub fn visit_chain<F>(&self, start_node_hash: String, chain_visitor: F)
        where F: ChainVisitor {

        if ! self.adjacent_matrix.contains_key(&start_node_hash) {
            warn!("Could not find start node with hash {:?} in chain. Not invoking chain walker", start_node_hash);
            return;
        }

        // this should never panic, as we check above that the start node exists
        let start_node = self.adjacent_matrix.get(&start_node_hash).unwrap();

        for child_block_hash in start_node.iter() {
            let child_block = self.blocks.get(child_block_hash);
            match child_block {
                Some(child_block) => {
                    chain_visitor.visit_block(child_block.clone());
                },
                None => {
                    panic!("Inconsistent state: Block {:?} referenced but not contained in available blocks.", child_block_hash)
                }
            }
        }
    }

}