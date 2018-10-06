/// A block of the blockchain.
pub mod block;

/// The data structure of the blockchain.
pub mod chain;

/// Visitors of the chain which can be used in combination with a chain walker.
pub mod chain_visitor;

/// `ChainWalker`s walk the blockchain in a specific, implementation-defined order.
/// Each such walker is provided with a `ChainVisitor` which in turn is invoked by the walker
/// at crucial steps during the traversal of the chain. When and if a visitor is invoked
/// is specific to an implementation of a `ChainWalker`.
pub mod chain_walker;

/// A transaction of the blockchain.
pub mod transaction;
