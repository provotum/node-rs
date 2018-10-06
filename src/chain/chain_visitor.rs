use ::chain::block::Block;
use ::chain::transaction::TransactionType;
use crypto_rs::el_gamal::ciphertext::CipherText;
use crypto_rs::el_gamal::additive::Operate;
use crypto_rs::el_gamal::encryption::{PublicKey, encrypt};
use crypto_rs::arithmetic::mod_int::ModInt;
use num::Zero;

pub trait ChainVisitor {
    /// Visit a particular block
    fn visit_block(&mut self, height: usize, block: &Block);
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
    pub heaviest_block: Option<String>
}

impl HeaviestBlockVisitor {
    /// Create a new `HeaviestBlockVisitor` having
    /// a `None` hash of the heaviest block.
    pub fn new() -> HeaviestBlockVisitor {
        HeaviestBlockVisitor {
            height: None,
            heaviest_block: None
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

pub struct SumCipherTextVisitor {
    sum_cipher_text: CipherText,
    total_votes: usize,
    zero_cipher_text: CipherText,
    is_voting_opened: bool,
    is_voting_closed: bool
}

impl SumCipherTextVisitor {
    pub fn new(public_key: PublicKey) -> SumCipherTextVisitor {
        let cipher_text = encrypt(&public_key, ModInt::zero());

        SumCipherTextVisitor {
            sum_cipher_text: cipher_text.clone(),
            total_votes: 0,
            zero_cipher_text: cipher_text,
            is_voting_opened: false,
            is_voting_closed: true
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

        info!("Counting votes in block {:?}", block.identifier.clone());

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
                    if ! self.is_voting_closed {
                        warn!("Skipping to count vote in transaction {:?} as voting was not yet closed", transaction.identifier.clone());
                    } else {
                        info!("Counting vote in transaction {:?}", transaction.identifier.clone());
                        self.sum_cipher_text = self.sum_cipher_text.clone().operate(transaction.data.unwrap().cipher_text);
                        self.total_votes = self.total_votes + 1;
                    }
                }
            }
        }
    }
}