use std::collections::{HashSet, HashMap};
use std::vec::Vec;
use bincode;
use std::io::Write;
use sha1::Sha1;

use ::chain::block::{Block};
use ::config::genesis::Genesis;


pub struct Chain {
    genesis_block: Block,
    blocks: HashSet<Block>,
    adjacent_matrix: HashMap<String, Vec<String>>
}

impl Chain {

    pub fn new(genesis: Genesis) -> Self {
        let bytes = bincode::serialize(&genesis).unwrap();
        let digest = Sha1::from(bytes).hexdigest();

        Chain {
            genesis_block: Block::new(digest, vec![]),
            blocks: HashSet::new(),
            adjacent_matrix: HashMap::new()
        }
    }

    pub fn add_block(&mut self, block: Block) {
        // // TODO: find longest path

        self.blocks.insert(block);
    }

    fn get_block_from_heaviest_path(self) {

    }
}