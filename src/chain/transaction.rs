use crypto_rs::cai::uciv::CaiProof;
use crypto_rs::el_gamal::ciphertext::CipherText;
use crypto_rs::el_gamal::membership_proof::MembershipProof;

/// Use Deserialize from Serde, Hash from std::hash
#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    voter_idx: usize,
    cipher_text: CipherText,
    membership_proof: MembershipProof,
    cai_proof: CaiProof,
}

impl Transaction {
    pub fn is_valid(&self) -> bool {
        // TODO:
        return true;
    }
}