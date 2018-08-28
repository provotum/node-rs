use std::collections::{HashSet, HashMap};
use std::vec::Vec;
use bincode;
use sha1::Sha1;

use ::chain::block::Block;
use ::chain::transaction::Transaction;
use ::config::genesis::Genesis;
use ::chain::chain_visitor::{ChainVisitor, HeaviestBlockVisitor};
use ::chain::chain_walker::{ChainWalker, LongestPathWalker};


pub struct Chain {
    /// the hash of the genesis configuration
    pub genesis_configuration_hash: String,
    /// all known blocks
    pub blocks: HashMap<String, Block>,
    /// a matrix creating the relation between blocks
    pub adjacent_matrix: HashMap<String, Vec<String>>
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
            adjacent_matrix
        }
    }

    /// Returns true, if the parent of the given block exists, false otherwise.
    pub fn has_parent_of_block(self, block: Block) -> bool {
        let parent_block = self.adjacent_matrix.get(&block.previous);

        parent_block.is_some()
    }

    /// Add the block as child to its corresponding parent.
    /// Panics, if the parent block specified does not exist.
    /// Therefore, invoke `has_parent_of_block` first.
    pub fn add_block(&mut self, block: Block) {
        trace!("matrix: {:?}", self.adjacent_matrix);

        // add block hash to its parent as child
        self.adjacent_matrix
            // in-place modification of the vector
            .entry(block.previous.clone())
            .and_modify(|parent_block_children| {
                    parent_block_children.push(block.current.clone());
            });

        // add a new entry for the block we've inserted
        // having currently no children
        self.adjacent_matrix
            // in-place modification of the vector
            .entry(block.current.clone())
            .or_insert(vec![]);

        // insert the block finally
        self.blocks.insert(block.current.clone(), block);
    }
}