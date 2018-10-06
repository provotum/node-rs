use std::collections::HashMap;
use std::vec::Vec;

use ::chain::block::Block;
use ::chain::transaction::Transaction;
use chain::chain_visitor::HeaviestBlockVisitor;
use chain::chain_walker::HeaviestBlockWalker;
use chain::chain_walker::ChainWalker;

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Chain {
    /// the hash of the genesis configuration
    pub genesis_configuration_hash: String,
    /// the hash of the genesis block
    pub genesis_identifier_hash: String,
    /// all known blocks
    pub blocks: HashMap<String, Block>,
    /// a matrix creating the relation between blocks
    /// key is the parent, values are its children
    pub adjacent_matrix: HashMap<String, Vec<String>>
}

impl Chain {

    pub fn new(genesis_hash: String) -> Self {
        // create the genesis block with an empty hash and no transactions
        let trxs: Vec<Transaction> = vec![];
        let genesis_block: Block = Block::new(String::new(), trxs);

        let mut blocks = HashMap::new();
        blocks.insert(genesis_block.identifier.clone(), genesis_block.clone());

        // Add an entry for the genesis block in the adjacent matrix,
        // i.e. initialize children of the genesis block as an empty vector.
        let mut adjacent_matrix: HashMap<String, Vec<String>> = HashMap::new();
        adjacent_matrix.insert(genesis_block.identifier.clone(), vec![]);

        trace!("Genesis block hash is: {:?}", genesis_block.identifier.clone());

        Chain {
            genesis_configuration_hash: genesis_hash,
            genesis_identifier_hash: genesis_block.identifier.clone(),
            blocks,
            adjacent_matrix
        }
    }

    pub fn get_current_block_number(&self) -> usize {
        self.get_current_block().0
    }

    pub fn get_current_block_timestamp(&self) -> u64 {
        self.get_current_block().1.data.timestamp
    }

    pub fn get_current_block(&self) -> (usize, Block) {
        let mut heaviest_block_visitor = HeaviestBlockVisitor::new();
        let longest_path_walker = HeaviestBlockWalker::new();
        longest_path_walker.walk_chain(&self, &mut heaviest_block_visitor);

        let heaviest_block_height_option = heaviest_block_visitor.height;
        assert!(heaviest_block_height_option.is_some());
        let heaviest_block_height = heaviest_block_height_option.unwrap();

        let option = heaviest_block_visitor.heaviest_block;
        assert!(option.is_some());
        let heaviest_block_reference = option.unwrap();

        (heaviest_block_height, (*self.blocks.get(&heaviest_block_reference).unwrap()).clone())
    }

    /// Returns true, if the parent of the given block exists, false otherwise.
    pub fn has_parent_of_block(self, block: Block) -> bool {
        let parent_block = self.adjacent_matrix.get(&block.data.parent);

        parent_block.is_some()
    }

    /// Add the block as child to its corresponding parent.
    /// Panics, if the parent block specified does not exist.
    /// Therefore, invoke `has_parent_of_block` first.
    ///
    /// Returns true, if the block was added, false otherwise.
    pub fn add_block(&mut self, block: Block) -> bool {
        // add block hash to its parent as child
        let mut is_contained = false;
        self.adjacent_matrix
            // in-place modification of the vector
            .entry(block.data.parent.clone())
            .and_modify(|parent_block_children| {
                if ! parent_block_children.contains(&block.identifier.clone()) {
                    info!("Adding block {:?} containing {:?} transactions to chain.", block.identifier.clone(), block.data.transactions.len());
                    parent_block_children.push(block.identifier.clone());
                } else {
                    debug!("Not adding block {:?} as it is already contained.", block.identifier.clone());
                    is_contained = true;
                }
            });

        if is_contained {
            return false;
        }

        // add a new entry for the block we've inserted
        // having currently no children
        self.adjacent_matrix
            // in-place modification of the vector
            .entry(block.identifier.clone())
            .or_insert(vec![]);

        // insert the block finally,
        // returns None if no block was contained at the given key,
        // but returns the old value if a block was already contained with the same key.
        let previous_block_option = self.blocks.insert(block.identifier.clone(), block);

        // this is a sanity check only, we should never panic here, but if we do
        // this might cause a huge mess...
        match previous_block_option {
            None => {
                return true;
            }
            Some(previous_block) => {
                panic!("Double insert of block {:?}", previous_block.identifier.clone)
            }
        }
    }
}

#[cfg(test)]
mod chain_test {

    use ::chain::block::{Block, BlockContent};
    use ::chain::chain::Chain;

    #[test]
    fn test_add_duplicate_block() {
        let mut chain = Chain::new(String::new());
        let genesis_id = chain.genesis_identifier_hash.clone();

        let block = Block {
            identifier: "1".to_string(),
            data: BlockContent {
                parent: genesis_id.clone(),
                timestamp: 1,
                transactions: vec![]
            }
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