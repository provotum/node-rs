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
    fn visit_chain<F: ChainVisitor>(&self, chain: &Chain, visitor: &mut F);
}

/// The longest path walker walks the given chain to find
/// the deepest block currently known and invokes any provided visitor
/// with the block found at the end of the longest path.
pub struct LongestPathWalker {}

impl LongestPathWalker {
    pub fn new() -> LongestPathWalker{
        LongestPathWalker {}
    }

    fn traverse_level(parent_level: usize, parent_block: &Block, chain: &Chain) -> (usize, String) {
        let mut most_deepest_block = (parent_level, parent_block.current.clone());

        // get all children of the current parent block
        let children = chain.adjacent_matrix.get(parent_block.current.as_str()).unwrap();

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
}

impl ChainWalker for LongestPathWalker {
    /// Visits the given chain to find the deepest block in the chain, i.e. the one
    /// having the most parents. Once found, it will invoke the given visitor with
    /// the corresponding found block.
    ///
    /// - `chain`: The chain to search for the deepest block.
    /// - `visitor`: A visitor which should be invoked with the deepest block found.
    ///
    /// A visitor can be of any type as long as it implements the trait `ChainVisitor`.
    fn visit_chain<F: ChainVisitor>(&self, chain: &Chain, visitor: &mut F) {
        let genesis_children = chain.adjacent_matrix.get(chain.genesis_identifier_hash.clone().as_str()).unwrap();

        let mut most_deepest_block = (0, chain.genesis_identifier_hash.clone());
        for genesis_child_hash in genesis_children.iter() {
            let genesis_child = chain.blocks.get(genesis_child_hash).unwrap();

            // genesis child is already at depth 1
            let result: (usize, String) = LongestPathWalker::traverse_level(1, genesis_child, &chain);

            // update current most deepest depth and the corresponding block hash
            if result.0 > most_deepest_block.0 {
                most_deepest_block.0 = result.0;
                most_deepest_block.1 = result.1;
            }
        }

        // visit the block being at the most deepest position
        let most_deepest_block = chain.blocks.get(most_deepest_block.1.as_str()).unwrap();
        visitor.visit_block(most_deepest_block);
    }
}

#[cfg(test)]
mod chain_walker_test {

    use ::config::genesis::{CliqueConfig, Genesis};
    use ::chain::block::{Block, BlockContent};
    use ::chain::chain::Chain;
    use ::chain::chain_visitor::HeaviestBlockVisitor;
    use ::chain::chain_walker::{ChainWalker, LongestPathWalker};


    /// Test that the longest chain is found if no conflicting
    /// branch is present, i.e. a branch having the exact amount of children
    /// as another once.
    #[test]
    fn test_longest_path() {
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

        // first level
        chain.add_block(Block {
            depth: 1,
            data: BlockContent {
                timestamp: 1,
                transactions: vec![]
            },
            previous: genesis_id,
            current: "1".to_string()
        });

        // second level
        chain.add_block(Block {
            depth: 2,
            data: BlockContent {
                timestamp: 2,
                transactions: vec![]
            },
            previous: "1".to_string(),
            current: "21".to_string()
        });

        chain.add_block(Block {
            depth: 2,
            data: BlockContent {
                timestamp: 3,
                transactions: vec![]
            },
            previous: "1".to_string(),
            current: "22".to_string()
        });

        // third level
        chain.add_block(Block {
            depth: 3,
            data: BlockContent {
                timestamp: 4,
                transactions: vec![]
            },
            previous: "22".to_string(),
            current: "3".to_string()
        });

        // fourth level
        chain.add_block(Block {
            depth: 4,
            data: BlockContent {
                timestamp: 5,
                transactions: vec![]
            },
            previous: "3".to_string(),
            current: "4".to_string()
        });

        let mut heaviest_block_walker = HeaviestBlockVisitor::new();
        let longest_path_walker = LongestPathWalker::new();
        longest_path_walker.visit_chain(&chain, &mut heaviest_block_walker);

        let option = heaviest_block_walker.heaviest_block;
        assert!(option.is_some());
        let expected_heaviest_block = option.unwrap();
        println!("expected heaviest block {:?}", expected_heaviest_block);
        assert!(expected_heaviest_block.eq(&"4".to_string()));
    }

    #[test]
    fn test_longest_path_for_two_blocks() {
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

        // first level
        chain.add_block(Block {
            depth: 1,
            data: BlockContent {
                timestamp: 1,
                transactions: vec![]
            },
            previous: genesis_id,
            current: "1".to_string()
        });

        let mut heaviest_block_walker = HeaviestBlockVisitor::new();
        let longest_path_walker = LongestPathWalker::new();
        longest_path_walker.visit_chain(&chain, &mut heaviest_block_walker);

        let option = heaviest_block_walker.heaviest_block;
        assert!(option.is_some());
        let expected_heaviest_block = option.unwrap();
        println!("expected heaviest block {:?}", expected_heaviest_block);
        assert!(expected_heaviest_block.eq(&"1".to_string()));
    }

    #[test]
    fn test_longest_path_for_empty_chain() {
        let genesis = Genesis {
            version: "test_version".to_string(),
            clique: CliqueConfig {
                block_period: 10,
                signer_limit: 1
            },
            sealer: vec![]
        };

        let chain = Chain::new(genesis);

        let mut heaviest_block_walker = HeaviestBlockVisitor::new();
        let longest_path_walker = LongestPathWalker::new();
        longest_path_walker.visit_chain(&chain, &mut heaviest_block_walker);

        let option = heaviest_block_walker.heaviest_block;
        assert!(option.is_some());
        let expected_heaviest_block = option.unwrap();
        assert!(chain.blocks.get(expected_heaviest_block.as_str()).unwrap().depth.eq(&0));
    }

}