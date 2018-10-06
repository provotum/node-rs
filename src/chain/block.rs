use std::vec::Vec;
use bincode;
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

use ::chain::transaction::Transaction;

/// The content of a block.
/// All contained fields are hashed and represent the identifier of the block.
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct BlockContent {
    pub parent: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
}

/// A block containing the identifier as well as its content, building
/// up the identifier.
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub identifier: String,
    pub data: BlockContent,
}

impl Block {

    /// Create a new block with the given parameters:
    ///
    /// - `previous_hash`: The hash of the previous block
    /// - `transactions`` A vector of transactions figuring as the data of this block
    pub fn new(previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

        let block_content = BlockContent {
            parent: previous_hash,
            timestamp: since_the_epoch,
            transactions
        };

        // we only want to hash the transactions to make sure, that these
        // are not duplicated. We don't care about the references of the block
        let bytes = bincode::serialize(&block_content).unwrap();
        let digest = Sha1::from(bytes).hexdigest();

        Block {
            identifier: digest,
            data: block_content,
        }
    }
}