use ::chain::block::Block;
use ::chain::chain::Chain;
use ::chain::chain_visitor::ChainVisitor;

/// A ChainWalker walks the given chain in a particular order
/// and can invoke the given visitor at any point during its traversal.
///
/// Note, that it is legal for a ChainWalker to invoke the provided visitor multiple
/// times, if desired. Each visitor needs to handle such a case.
pub trait ChainWalker {
    /// Visit the given chain in a particular order.
    /// The concrete implementation will specify when and for which blocks
    /// the provided visitor should be invoked.
    ///
    /// - `chain` The chain to walk.
    /// - `visitor` The visitor to invoke during chain traversal.
    fn walk_chain<F: ChainVisitor>(&self, chain: &Chain, visitor: &mut F);
}

/// The heaviest block walker walks the given chain to find
/// the deepest block currently known and invokes any provided visitor
/// with the block found at the end of the longest path.
pub struct HeaviestBlockWalker {}

impl HeaviestBlockWalker {
    pub fn new() -> HeaviestBlockWalker {
        HeaviestBlockWalker {}
    }

    fn traverse_level(parent_level: usize, parent_block: &Block, chain: &Chain) -> (usize, String) {
        let mut most_deepest_block = (parent_level, parent_block.identifier.clone());

        // get all children of the current parent block
        let children = chain.adjacent_matrix.get(parent_block.identifier.as_str()).unwrap();

        let current_child_level = parent_level + 1;
        for child_hash in children.iter() {
            let child = chain.blocks.get(child_hash).unwrap();

            let result: (usize, String) = HeaviestBlockWalker::traverse_level(current_child_level, child, &chain);

            // update current most deepest depth and the corresponding block hash
            if result.0 > most_deepest_block.0 {
                most_deepest_block.0 = result.0;
                most_deepest_block.1 = result.1;
            }
        }

        most_deepest_block
    }
}

impl ChainWalker for HeaviestBlockWalker {
    /// Visits the given chain to find the deepest block in the chain, i.e. the one
    /// having the most parents. Once found, it will invoke the given visitor with
    /// the corresponding found block.
    ///
    /// - `chain`: The chain to search for the deepest block.
    /// - `visitor`: A visitor which should be invoked with the deepest block found.
    ///
    /// A visitor can be of any type as long as it implements the trait `ChainVisitor`.
    fn walk_chain<F: ChainVisitor>(&self, chain: &Chain, visitor: &mut F) {
        let genesis_children = chain.adjacent_matrix.get(chain.genesis_identifier_hash.clone().as_str()).unwrap();

        let mut current_deepest_block = (0, chain.genesis_identifier_hash.clone());
        for genesis_child_hash in genesis_children.iter() {
            let genesis_child = chain.blocks.get(genesis_child_hash).unwrap();

            // genesis child is already at depth 1
            let result: (usize, String) = HeaviestBlockWalker::traverse_level(1, genesis_child, &chain);

            // update current most deepest depth and the corresponding block hash
            if result.0 > current_deepest_block.0 {
                current_deepest_block.0 = result.0;
                current_deepest_block.1 = result.1;
            }
        }

        // visit the block being at the most deepest position
        let deepest_block = chain.blocks.get(current_deepest_block.1.as_str()).unwrap();
        visitor.visit_block(current_deepest_block.0, deepest_block);
    }
}

pub struct LongestPathWalker {}

impl LongestPathWalker {
    pub fn new() -> LongestPathWalker {
        LongestPathWalker {}
    }

    fn traverse_level(parent_level: usize, parent_block: &Block, chain: &Chain) -> (usize, String) {
        let mut most_deepest_block = (parent_level, parent_block.identifier.clone());

        // get all children of the current parent block
        let children = chain.adjacent_matrix.get(parent_block.identifier.as_str()).unwrap();

        let current_child_level = parent_level + 1;
        for child_hash in children.iter() {
            let child = chain.blocks.get(child_hash).unwrap();

            let result: (usize, String) = LongestPathWalker::traverse_level(current_child_level, child, &chain);

            // update current most deepest depth and the corresponding block hash
            if result.0 > most_deepest_block.0 {
                most_deepest_block.0 = result.0;
                most_deepest_block.1 = result.1;
            }
        }

        most_deepest_block
    }

    fn traverse_bottom_up<F: ChainVisitor>(child_level: usize, child_block: &Block, chain: &Chain, visitor: &mut F) {
        // check whether we've reached the genesis block
        // which we do not visit
        if child_block.data.parent == String::new() {
            return;
        }

        visitor.visit_block(child_level, child_block);

        let parent_block = chain.blocks.get(child_block.data.parent.as_str()).unwrap();
        Self::traverse_bottom_up(child_level - 1, parent_block, chain, visitor);
    }
}

impl ChainWalker for LongestPathWalker {
    fn walk_chain<F: ChainVisitor>(&self, chain: &Chain, visitor: &mut F) {
        let genesis_children = chain.adjacent_matrix.get(chain.genesis_identifier_hash.clone().as_str()).unwrap();

        let mut current_deepest_block = (0, chain.genesis_identifier_hash.clone());
        for genesis_child_hash in genesis_children.iter() {
            let genesis_child = chain.blocks.get(genesis_child_hash).unwrap();

            // genesis child is already at depth 1
            let result: (usize, String) = LongestPathWalker::traverse_level(1, genesis_child, &chain);

            // update current most deepest depth and the corresponding block hash
            if result.0 > current_deepest_block.0 {
                current_deepest_block.0 = result.0;
                current_deepest_block.1 = result.1;
            }
        }

        // visit the block being at the most deepest position
        let deepest_block = chain.blocks.get(current_deepest_block.1.as_str()).unwrap();
        LongestPathWalker::traverse_bottom_up(current_deepest_block.0, deepest_block, chain, visitor);
    }
}

#[cfg(test)]
mod chain_walker_test {

    use ::chain::block::{Block, BlockContent};
    use ::chain::chain::Chain;
    use ::chain::chain_visitor::{HeaviestBlockVisitor, SumCipherTextVisitor};
    use ::chain::chain_walker::{ChainWalker, HeaviestBlockWalker, LongestPathWalker};
    use ::chain::transaction::Transaction;
    use crypto_rs::el_gamal::encryption::{encrypt, PublicKey};
    use crypto_rs::el_gamal::ciphertext::CipherText;
    use crypto_rs::el_gamal::membership_proof::MembershipProof;
    use crypto_rs::arithmetic::mod_int::ModInt;
    use crypto_rs::cai::uciv::{CaiProof, PreImageSet, ImageSet};
    use num::One;

    /// Test that the longest chain is found if no conflicting
    /// branch is present, i.e. a branch having the exact amount of children
    /// as another once.
    #[test]
    fn test_heaviest_path() {
        let mut chain = Chain::new(String::new());
        let genesis_id = chain.genesis_identifier_hash.clone();

        // first level
        chain.add_block(Block {
            identifier: "1".to_string(),
            data: BlockContent {
                parent: genesis_id,
                timestamp: 1,
                transactions: vec![]
            }
        });

        // second level
        chain.add_block(Block {
            identifier: "21".to_string(),
            data: BlockContent {
                parent: "1".to_string(),
                timestamp: 2,
                transactions: vec![]
            }
        });

        chain.add_block(Block {
            identifier: "22".to_string(),
            data: BlockContent {
                parent: "1".to_string(),
                timestamp: 3,
                transactions: vec![]
            }
        });

        // third level
        chain.add_block(Block {
            identifier: "3".to_string(),
            data: BlockContent {
                parent: "22".to_string(),
                timestamp: 4,
                transactions: vec![]
            }
        });

        // fourth level
        chain.add_block(Block {
            identifier: "4".to_string(),
            data: BlockContent {
                parent: "3".to_string(),
                timestamp: 5,
                transactions: vec![]
            }
        });

        let mut heaviest_block_visitor = HeaviestBlockVisitor::new();
        let longest_path_walker = HeaviestBlockWalker::new();
        longest_path_walker.walk_chain(&chain, &mut heaviest_block_visitor);

        let heaviest_block_height = heaviest_block_visitor.height;
        assert!(heaviest_block_height.is_some(), "Expected that heaviest block height is of type Some()");
        assert!(heaviest_block_height.unwrap().eq(&4), "Expected that heaviest block height is 4");

        let heaviest_block = heaviest_block_visitor.heaviest_block;
        assert!(heaviest_block.is_some());
        let expected_heaviest_block = heaviest_block.unwrap();
        println!("expected heaviest block {:?}", expected_heaviest_block);
        assert!(expected_heaviest_block.eq(&"4".to_string()));
    }

    #[test]
    fn test_heaviest_path_for_two_blocks() {
        let mut chain = Chain::new(String::new());
        let genesis_id = chain.genesis_identifier_hash.clone();

        // first level
        chain.add_block(Block {
            identifier: "1".to_string(),
            data: BlockContent {
                parent: genesis_id,
                timestamp: 1,
                transactions: vec![]
            }
        });

        let mut heaviest_block_visitor = HeaviestBlockVisitor::new();
        let longest_path_walker = HeaviestBlockWalker::new();
        longest_path_walker.walk_chain(&chain, &mut heaviest_block_visitor);

        let heaviest_block_height = heaviest_block_visitor.height;
        assert!(heaviest_block_height.is_some(), "Expected that heaviest block height is of type Some()");
        assert!(heaviest_block_height.unwrap().eq(&1), "Expected that heaviest block height is 1");

        let option = heaviest_block_visitor.heaviest_block;
        assert!(option.is_some());
        let expected_heaviest_block = option.unwrap();
        println!("expected heaviest block {:?}", expected_heaviest_block);
        assert!(expected_heaviest_block.eq(&"1".to_string()));
    }

    #[test]
    fn test_heaviest_path_for_empty_chain() {
        let chain = Chain::new(String::new());

        let mut heaviest_block_visitor = HeaviestBlockVisitor::new();
        let longest_path_walker = HeaviestBlockWalker::new();
        longest_path_walker.walk_chain(&chain, &mut heaviest_block_visitor);

        let heaviest_block_height = heaviest_block_visitor.height;
        assert!(heaviest_block_height.is_some(), "Expected that heaviest block height is of type Some()");
        assert!(heaviest_block_height.unwrap().eq(&0), "Expected that heaviest block height is 0");

        let option = heaviest_block_visitor.heaviest_block;
        assert!(option.is_some());
        let expected_heaviest_block = option.unwrap();
        assert!(chain.blocks.get(expected_heaviest_block.as_str()).unwrap().data.parent.eq(&String::new()));
    }

    #[test]
    fn test_longest_path_sum() {
        let mut chain = Chain::new(String::new());
        let genesis_id = chain.genesis_identifier_hash.clone();

        let public_key = PublicKey {
            p: ModInt::one(),
            q: ModInt::one(),
            h: ModInt::one(),
            g: ModInt::one(),
        };

        let cipher_text = CipherText {
            big_h: ModInt::one(),
            big_g: ModInt::one(),
            random: ModInt::one()
        };

        let pre_image_set = PreImageSet {
            pre_images: vec![ModInt::one()]
        };

        let image_set = ImageSet {
            images: vec![ModInt::one()]
        };

        let open_trx = Transaction::new_voting_opened();

        let trx = Transaction::new_vote(
            0,
            cipher_text.clone(),
            MembershipProof::new(public_key.clone(), ModInt::one(), cipher_text.clone(), vec![ModInt::one()]),
            CaiProof::new(public_key.clone(), cipher_text.clone(), pre_image_set.clone(), image_set.clone(), 0, vec![ModInt::one()]),
        );

        let close_trx = Transaction::new_voting_opened();

        // first level
        chain.add_block(Block {
            identifier: "1".to_string(),
            data: BlockContent {
                parent: genesis_id,
                timestamp: 1,
                transactions: vec![open_trx.clone(), trx.clone(), close_trx.clone()]
            }
        });

        let cipher_text = encrypt(&public_key.clone(), ModInt::one());

        let mut sum_cipher_text_visitor = SumCipherTextVisitor::new(cipher_text);
        let longest_path_walker = LongestPathWalker::new();
        longest_path_walker.walk_chain(&chain, &mut sum_cipher_text_visitor);

        assert_eq!(1, sum_cipher_text_visitor.total_votes);
    }

}