use ::chain::block::Block;

pub trait ChainVisitor {
    fn visit_block(&mut self, block: &Block);
}

/// This visitor expects to be called exactly once
/// with the heaviest block in the chain.
///
/// After it is called, it's heaviest_block contains
/// a hash of the heaviest block currently in the chain.
pub struct HeaviestBlockVisitor {
    /// The hash of the string once it is assigned,
    /// or None, if this visitor was never visited.
    pub heaviest_block: Option<String>
}

impl HeaviestBlockVisitor {
    /// Create a new `HeaviestBlockVisitor` having
    /// a `None` hash of the heaviest block.
    pub fn new() -> HeaviestBlockVisitor {
        HeaviestBlockVisitor {
            heaviest_block: None
        }
    }
}

impl ChainVisitor for HeaviestBlockVisitor {
    /// Expects to be called only once. Will panic otherwise.
    fn visit_block(&mut self, block: &Block) {
        match self.heaviest_block {
            Some(ref block_hash) => {
                panic!("Cannot assign the heaviest block a second time. Previous heaviest block was {:?}", block_hash);
            }
            None => {
                self.heaviest_block = Some(block.current.clone());
            }
        }
    }
}