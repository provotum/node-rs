use std::vec::Vec;
use std::option::Option;
use std::boxed::Box;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::borrow::BorrowMut;
use bincode;
use sha1::Sha1;

use uuid::Uuid;

use ::chain::transaction::Transaction;

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct BlockContent {
    transactions: Vec<Transaction>
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Block {
    data: BlockContent,
    previous: String,
    current: String
}

impl Block {

    /// Create a new block with the given parameters:
    ///
    /// - `previous_hash`: The hash of the previous block
    /// - `transactions`` A vector of transactions figuring as the data of this block
    pub fn new(previous_hash: String, transactions: Vec<Transaction>) -> Self {

        let block_content = BlockContent {
            transactions
        };

        let bytes = bincode::serialize(&block_content).unwrap();
        let digest = Sha1::from(bytes).hexdigest();

        Block {
            data: block_content,
            previous: previous_hash,
            current: digest
        }
    }
}