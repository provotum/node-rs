use std::vec::Vec;
use serde_json;
use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr};

/// The configuration for the blockchain, usually
/// included in the first block of a chain, and therefore often referred to
/// as genesis block.
///
/// Use Deserialize from Serde, Hash from std::hash
#[derive(Serialize, Deserialize, Debug)]
pub struct Genesis {
    pub version: String,
    pub clique: CliqueConfig,
    pub sealer: Vec<SocketAddr>
}

/// A configuration element for clique specific values.
#[derive(Serialize, Deserialize, Debug)]
pub struct CliqueConfig {
    pub block_period: u64,
    pub signer_limit: usize,
}

impl Genesis {

    /// Create a new Genesis configuration based on a specific configuration.
    ///
    /// - `genesis_file_name`: The file name of the genesis configuration.
    ///                      Must reside in the same directory as the binary is launched.
    ///
    /// Panics if the content of the configured genesis file is not valid w.r.t. a genesis block.
    ///
    pub fn new(genesis_file_name: &str) -> Self {
        // Read the input file to string.
        let mut file = File::open("./".to_owned() + genesis_file_name).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let genesis_data: Genesis = match serde_json::from_str(&contents) {
            Ok(genesis_data) => {
                trace!("Read genesis configuration: {:?}", genesis_data);
                genesis_data
            },
            Err(e) => {
                panic!("Failed to transform file {:?} into Genesis configuration: {:?}", file, e);
            }
        };

        assert!(genesis_data.version.len() > 0, "Version parameter must be specified");
        assert!(genesis_data.clique.block_period > 0, "Clique block period must be greater than zero");
        assert!(genesis_data.sealer.len() > 0, "There must be at least a single sealer");

        // TODO: if only one sealer -> what should the signer_limit value be?

        genesis_data
    }


}