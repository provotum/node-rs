use crypto_rs::cai::uciv::CaiProof;
use crypto_rs::el_gamal::ciphertext::CipherText;
use crypto_rs::el_gamal::membership_proof::MembershipProof;
use crypto_rs::el_gamal::encryption::PublicKey;
use crypto_rs::arithmetic::mod_int::From;
use crypto_rs::arithmetic::mod_int::ModInt;
use crypto_rs::cai::uciv::ImageSet;
use num::{One, Zero};
use num::BigInt;
use std::vec::Vec;
use bincode;
use sha1::Sha1;
use std::cmp::{Eq, PartialEq};
use std::option::Option;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub enum TransactionType {
    Vote,
    VoteOpened,
    VoteClosed
}

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub struct TransactionData {
    pub voter_idx: usize,
    pub cipher_text: CipherText,
    pub membership_proof: MembershipProof,
    pub cai_proof: CaiProof,
}

/// Use Deserialize from Serde, Hash from std::hash
#[derive(Hash, Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub identifier: String,
    pub trx_type: TransactionType,
    pub data: Option<TransactionData>
}

impl Transaction {
    pub fn new_voting_opened() -> Transaction {
        // hash the transaction type as no data is needed
        let bytes = bincode::serialize(&TransactionType::VoteOpened).unwrap();
        let digest = Sha1::from(bytes).hexdigest();

        Transaction {
            identifier: digest,
            trx_type: TransactionType::VoteOpened,
            data: None
        }
    }

    pub fn new_voting_closed() -> Transaction {
        // hash the transaction type as no data is needed
        let bytes = bincode::serialize(&TransactionType::VoteClosed).unwrap();
        let digest = Sha1::from(bytes).hexdigest();

        Transaction {
            identifier: digest,
            trx_type: TransactionType::VoteClosed,
            data: None
        }
    }

    pub fn new_vote(voter_idx: usize, cipher_text: CipherText, membership_proof: MembershipProof, cai_proof: CaiProof) -> Transaction {
        let trx_data = TransactionData {
            voter_idx,
            cipher_text,
            membership_proof,
            cai_proof
        };
        // we only want to hash the transactions to make sure, that these
        // are not duplicated. We don't care about the references of the block
        let bytes = bincode::serialize(&trx_data).unwrap();
        let digest = Sha1::from(bytes).hexdigest();

        Transaction {
            identifier: digest,
            trx_type: TransactionType::Vote,
            data: Some(trx_data)
        }
    }

    /// Verify whether the proofs submitted along with the transaction
    /// are valid with respect to the proofs submitted along with it.
    ///
    /// - public_key: The public key used to encrypt the vote
    /// - image_sets: The set of all voters' images
    pub fn is_valid(&self, public_key: PublicKey, image_sets: Vec<ImageSet>) -> bool {
        if TransactionType::Vote != self.trx_type {
            trace!("Considering vote of type {:?} as valid", self.trx_type);
            return true;
        }

        let voting_options: Vec<ModInt> = vec![
            ModInt::from_value(BigInt::one()),
            ModInt::from_value(BigInt::zero())
        ];

        trace!("Verifying membership proof...");
        let is_membership_proof_valid = self.data.clone().unwrap().membership_proof.verify(public_key.clone(), self.data.clone().unwrap().cipher_text.clone(), voting_options.clone());
        trace!("Is membership proof valid: {:?}", is_membership_proof_valid);

        trace!("Retrieving public UCIV for voter index {}", self.data.clone().unwrap().voter_idx);
        let image_set_option = image_sets.get(self.data.clone().unwrap().voter_idx as usize);
        let image_set: ImageSet = match image_set_option {
            Some(image_set) => {
                image_set.clone()
            },
            None => {
                error!("Could not find voter_index {} in public UCIV information. Transaction is invalid", self.data.clone().unwrap().voter_idx);
                return false;
            }
        };

        // If the image set has not an equal number of voting options
        // this is considered a configuration error.
        assert_eq!(image_set.images.len(), voting_options.len(), "The set of voting options and images of a voter must be equal");

        trace!("Verifying cast-as-intended proof...");
        let is_cai_proof_valid = self.data.clone().unwrap().cai_proof.verify(public_key, self.data.clone().unwrap().cipher_text.clone(), image_set, voting_options);
        trace!("Is cast-as-intended proof valid: {:?}", is_cai_proof_valid);

        is_membership_proof_valid && is_cai_proof_valid
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Transaction) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for Transaction {}