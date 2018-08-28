pub mod transaction;
pub mod block;
pub mod chain;
pub mod chain_visitor;

/// `ChainWalker`s walk the blockchain in a specific, implementation-defined order.
/// Each such walker is provided with a `ChainVisitor` which in turn is invoked by the walker
/// at crucial steps during the traversal of the chain. When and if a visitor is invoked
/// is specific to an implementation of a `ChainWalker`.
pub mod chain_walker;
