use ::chain::block::Block;
use ::chain::transaction::{Transaction, TransactionType};
use crypto_rs::el_gamal::ciphertext::CipherText;
use crypto_rs::el_gamal::additive::Operate;
use crypto_rs::el_gamal::encryption::{PublicKey, encrypt};
use crypto_rs::arithmetic::mod_int::ModInt;
use num::Zero;
use std::collections::HashSet;

pub trait ChainVisitor {
    /// Visit a particular block
    fn visit_block(&mut self, height: usize, block: &Block);
}

/// This visitor expects to be called on each level
/// in order to find a transaction with a particular identifier.
pub struct FindTransactionVisitor {
    transaction_identifier: String,
    found_transaction: Option<Transaction>,
}

impl FindTransactionVisitor {
    /// Create a new find transaction visitor
    ///
    /// - trx_identifier: The identifier of the transaction to search for
    pub fn new(trx_identifier: String) -> FindTransactionVisitor {
        FindTransactionVisitor {
            transaction_identifier: trx_identifier,
            found_transaction: None,
        }
    }

    /// Get the found transaction.
    /// Returns None if the transaction could not be found, the transaction otherwise.
    pub fn get_found_transaction(&self) -> Option<Transaction> {
        self.found_transaction.clone()
    }
}

impl ChainVisitor for FindTransactionVisitor {
    /// Visit a block of the blockchain.
    fn visit_block(&mut self, _height: usize, block: &Block) {
        match self.found_transaction {
            Some(_) => {
                return;
            }
            None => {
                for transaction in block.data.transactions.clone() {
                    if self.transaction_identifier.eq(&transaction.identifier) {
                        self.found_transaction = Some(transaction.clone());
                    }

                    break;
                }
            }
        }
    }
}

/// This visitor expects to be called exactly once
/// with the heaviest block in the chain.
///
/// After it is called, it's heaviest_block contains
/// a hash of the heaviest block currently in the chain.
pub struct HeaviestBlockVisitor {
    /// The height of the heaviest block.
    pub height: Option<usize>,
    /// The hash of the string once it is assigned,
    /// or None, if this visitor was never visited.
    pub heaviest_block: Option<String>,
}

impl HeaviestBlockVisitor {
    /// Create a new `HeaviestBlockVisitor` having
    /// a `None` hash of the heaviest block.
    pub fn new() -> HeaviestBlockVisitor {
        HeaviestBlockVisitor {
            height: None,
            heaviest_block: None,
        }
    }
}

impl ChainVisitor for HeaviestBlockVisitor {
    /// Expects to be called only once. Will panic otherwise.
    fn visit_block(&mut self, height: usize, block: &Block) {
        match self.heaviest_block {
            Some(ref block_hash) => {
                panic!("Cannot assign the heaviest block a second time. Previous heaviest block was {:?}", block_hash);
            }
            None => {
                self.height = Some(height);
                self.heaviest_block = Some(block.identifier.clone());
            }
        }
    }
}

/// Sums up all votes contained in the transactions, after the voting has been opened
/// and until it is closed again.
///
/// Expects to be walked from the bottom up of the chain
/// to the root to work correctly.
pub struct SumCipherTextVisitor {
    sum_cipher_text: CipherText,
    total_votes: usize,
    zero_cipher_text: CipherText,
    is_voting_opened: bool,
    is_voting_closed: bool,
    traversed_vote_idx: HashSet<usize>,
}

impl SumCipherTextVisitor {
    pub fn new(public_key: PublicKey) -> SumCipherTextVisitor {
        let cipher_text = encrypt(&public_key, ModInt::zero());

        SumCipherTextVisitor {
            sum_cipher_text: cipher_text.clone(),
            total_votes: 0,
            zero_cipher_text: cipher_text,
            is_voting_opened: false,
            is_voting_closed: true,
            traversed_vote_idx: HashSet::new(),
        }
    }

    pub fn get_votes(&self) -> (usize, CipherText) {
        // Now check that the voting was opened.
        // Note, that we cannot do this during block traversal as we do not know
        // when we've arrived at the root of the chain. Yes, we may check the parent hash
        // to be null/empty but this creates a dependency on how the genesis block is structured.
        if self.is_voting_opened {
            return (self.total_votes, self.sum_cipher_text.clone());
        } else {
            warn!("Voting was never opened.");
            return (0, self.zero_cipher_text.clone());
        }
    }
}

impl ChainVisitor for SumCipherTextVisitor {
    fn visit_block(&mut self, _height: usize, block: &Block) {
        // Note: The blockchain is visited from the newest block first and is then
        // traversed from the bottom up.

        debug!("Counting votes in block {:?}", block.identifier.clone());

        // homomorphically add the cipher text
        for transaction in block.data.transactions.clone() {
            match transaction.trx_type {
                TransactionType::VoteOpened => {
                    info!("Found open vote transaction {:?}", transaction.identifier.clone());
                    self.is_voting_opened = true
                }
                TransactionType::VoteClosed => {
                    info!("Found close vote transaction {:?}", transaction.identifier.clone());
                    self.is_voting_closed = true
                }
                TransactionType::Vote => {
                    // chain is traversed bottom up, so check first whether the voting
                    // was closed at the end.
                    if !self.is_voting_closed {
                        warn!("Skipping to count vote in transaction {:?} as voting was not yet closed", transaction.identifier.clone());
                    } else {

                        // check whether we already counted a vote for the same voter
                        let trx_data = transaction.data.unwrap();
                        if self.traversed_vote_idx.contains(&trx_data.voter_idx) {
                            info!("Voter with index {:?} has voted already. Ignoring transaction {:?}", trx_data.voter_idx, transaction.identifier.clone())
                        } else {
                            info!("Counting vote in transaction {:?}", transaction.identifier.clone());
                            self.sum_cipher_text = self.sum_cipher_text.clone().operate(trx_data.cipher_text);
                            self.total_votes = self.total_votes + 1;
                            self.traversed_vote_idx.insert(trx_data.voter_idx);
                        }
                    }
                }
            }
        }
    }
}