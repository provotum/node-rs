
/// Use Deserialize from Serde, Hash from std::hash
#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub from: String

    // TODO: add vote
    // TODO: add 0/1 zk proof
    // TODO: add cai zk proof
}

impl Transaction {

    pub fn is_valid(&self) -> bool {
        // TODO:
        return true;
    }
}