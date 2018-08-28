use ::chain::block::Block;

pub trait ChainVisitor {
    fn visit_block(&self, block: Block);
}

