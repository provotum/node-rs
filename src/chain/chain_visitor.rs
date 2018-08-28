use ::chain::block::Block;

pub trait ChainVisitor {
    fn visit_block(&mut self, block: &Block);
}

pub struct HeaviestBlockVisitor {
    pub heaviest_block: String
}

impl HeaviestBlockVisitor {

    pub fn new() -> HeaviestBlockVisitor {
        HeaviestBlockVisitor {
            heaviest_block: String::new()
        }
    }
}

impl ChainVisitor for HeaviestBlockVisitor {

    fn visit_block(&mut self, block: &Block) {
        self.heaviest_block = block.current.clone();
    }
}