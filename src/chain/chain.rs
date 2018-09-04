use std::collections::HashMap;
use std::vec::Vec;
use bincode;
use sha1::Sha1;

use ::chain::block::Block;
use ::chain::transaction::Transaction;
use ::config::genesis::Genesis;
use chain::chain_visitor::HeaviestBlockVisitor;
use chain::chain_walker::LongestPathWalker;
use chain::chain_walker::ChainWalker;

pub struct Chain {
    /// the hash of the genesis configuration
    pub genesis_configuration_hash: String,
    /// the hash of the genesis block
    pub genesis_identifier_hash: String,
    /// all known blocks
    pub blocks: HashMap<String, Block>,
    /// a matrix creating the relation between blocks
    pub adjacent_matrix: HashMap<String, Vec<String>>
}

impl Chain {

    pub fn new(genesis: Genesis) -> Self {
        // create the genesis block with an empty hash and no transactions
        let trxs: Vec<Transaction> = vec![];
        let genesis_block: Block = Block::new(0,String::new(), trxs);

        let mut blocks = HashMap::new();
        blocks.insert(genesis_block.current.clone(), genesis_block.clone());

        // Add an entry for the genesis block in the adjacent matrix,
        // i.e. initialize children of the genesis block as an empty vector.
        let mut adjacent_matrix: HashMap<String, Vec<String>> = HashMap::new();
        adjacent_matrix.insert(genesis_block.current.clone(), vec![]);

        // Create a sha1 digest of the genesis configuration so that we can later
        // ensure, that we only accept blocks from a chain with the same configuration.
        let bytes = bincode::serialize(&genesis).unwrap();
        let digest: String = Sha1::from(bytes).hexdigest();

        trace!("Genesis block hash is: {:?}", genesis_block.current.clone());

        Chain {
            genesis_configuration_hash: digest,
            genesis_identifier_hash: genesis_block.current.clone(),
            blocks,
            adjacent_matrix
        }
    }

    pub fn get_current_block_number(&self) -> usize {
        let depth = self.get_current_block().depth;

        depth + 1
    }

    pub fn get_current_block_timestamp(&self) -> u64 {
        self.get_current_block().data.timestamp
    }

    pub fn get_current_block(&self) -> Block {
        let mut heaviest_block_walker = HeaviestBlockVisitor::new();
        let longest_path_walker = LongestPathWalker::new();
        longest_path_walker.visit_chain(&self, &mut heaviest_block_walker);

        let option = heaviest_block_walker.heaviest_block;
        assert!(option.is_some());
        let heaviest_block_reference = option.unwrap();

        (*self.blocks.get(&heaviest_block_reference).unwrap()).clone()
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
        // add block hash to its parent as child
        self.adjacent_matrix
            // in-place modification of the vector
            .entry(block.previous.clone())
            .and_modify(|parent_block_children| {
                if ! parent_block_children.contains(&block.current.clone()) {
                    parent_block_children.push(block.current.clone());
                }
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

#[cfg(test)]
mod chain_test {

    use ::config::genesis::{CliqueConfig, Genesis};
    use ::chain::block::{Block, BlockContent};
    use ::chain::chain::Chain;

    #[test]
    fn test_add_duplicate_block() {
        let genesis = Genesis {
            version: "test_version".to_string(),
            clique: CliqueConfig {
                block_period: 10,
                signer_limit: 1
            },
            sealer: vec![]
        };


        let mut chain = Chain::new(genesis);
        let genesis_id = chain.genesis_identifier_hash.clone();

        let block = Block {
            depth: 1,
            data: BlockContent {
                timestamp: 1,
                transactions: vec![]
            },
            previous: genesis_id.clone(),
            current: "1".to_string()
        };

        assert!(chain.blocks.len().eq(&1));

        // first level
        chain.add_block(block.clone());
        chain.add_block(block.clone());

        // genesis block and the first of the duplicates
        assert!(chain.blocks.len().eq(&2));

        // assert that adjacent matrix is also correct:
        // i.e. only one child is present for the genesis block
        assert!(chain.adjacent_matrix.get(&genesis_id.clone()).unwrap().len().eq(&1));
    }

}