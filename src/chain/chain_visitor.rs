use ::chain::block::Block;
use crypto_rs::el_gamal::ciphertext::CipherText;
use crypto_rs::el_gamal::additive::Operate;

pub trait ChainVisitor {
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
    pub sum_cipher_text: CipherText,
    pub total_votes: usize
}

impl SumCipherTextVisitor {
    pub fn new(zero_cipher_text: CipherText) -> SumCipherTextVisitor {
        SumCipherTextVisitor {
            sum_cipher_text: zero_cipher_text,
            total_votes: 0
        }
    }
}

impl ChainVisitor for SumCipherTextVisitor {
    fn visit_block(&mut self, _height: usize, block: &Block) {
        // homomorphically add the cipher text
        for transaction in block.data.transactions.clone() {
            self.sum_cipher_text = self.sum_cipher_text.clone().operate(transaction.cipher_text);
            self.total_votes = self.total_votes + 1;
        }
    }
}