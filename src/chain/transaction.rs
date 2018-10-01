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

/// Use Deserialize from Serde, Hash from std::hash
#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub voter_idx: usize,
    pub cipher_text: CipherText,
    pub membership_proof: MembershipProof,
    pub cai_proof: CaiProof,
}

impl Transaction {

    /// Verify whether the proofs submitted along with the transaction
    /// are valid with respect to the proofs submitted along with it.
    ///
    /// - public_key: The public key used to encrypt the vote
    /// - image_sets: The set of all voters' images
    pub fn is_valid(&self, public_key: PublicKey, image_sets: Vec<ImageSet>) -> bool {
        let voting_options: Vec<ModInt> = vec![
            ModInt::from_value(BigInt::one()),
            ModInt::from_value(BigInt::zero())
        ];

        trace!("Verifying membership proof...");
        let is_membership_proof_valid = self.membership_proof.verify(public_key.clone(), self.cipher_text.clone(), voting_options.clone());
        trace!("Is membership proof valid: {:?}", is_membership_proof_valid);

        trace!("Retrieving public UCIV for voter index {}", self.voter_idx);
        let image_set_option = image_sets.get(self.voter_idx as usize);
        let image_set: ImageSet = match image_set_option {
            Some(image_set) => {
                image_set.clone()
            },
            None => {
                error!("Could not find voter_index {} in public UCIV information. Transaction is invalid", self.voter_idx);
                return false;
            }
        };

        // If the image set has not an equal number of voting options
        // this is considered a configuration error.
        assert_eq!(image_set.images.len(), voting_options.len(), "The set of voting options and images of a voter must be equal");

        trace!("Verifying cast-as-intended proof...");
        let is_cai_proof_valid = self.cai_proof.verify(public_key, self.cipher_text.clone(), image_set, voting_options);
        trace!("Is cast-as-intended proof valid: {:?}", is_cai_proof_valid);

        is_membership_proof_valid && is_cai_proof_valid
    }
}