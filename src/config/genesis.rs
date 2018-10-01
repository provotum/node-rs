use std::vec::Vec;
use serde_json;
use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr};
use crypto_rs::el_gamal::encryption::PublicKey;
use crypto_rs::cai::uciv::ImageSet;
use std::path::Path;

/// Use Deserialize from Serde, Hash from std::hash
#[derive(Serialize, Deserialize, Debug)]
pub struct GenesisData {
    pub version: String,
    pub clique: CliqueConfig,
    pub sealer: Vec<SocketAddr>
}

/// A configuration element for clique specific values.
#[derive(Serialize, Deserialize, Debug)]
pub struct CliqueConfig {
    pub block_period: u64,
    pub signer_limit: usize
}

/// The configuration for the blockchain, usually
/// included in the first block of a chain, and therefore often referred to
/// as genesis block.
#[derive(Serialize, Deserialize)]
pub struct Genesis {
    pub version: String,
    pub clique: CliqueConfig,
    pub sealer: Vec<SocketAddr>,
    pub public_key: PublicKey,
    pub public_uciv: Vec<ImageSet>
}

impl Genesis {

    /// Create a new Genesis configuration based on a specific configuration.
    ///
    /// - genesis_file_name: The file name of the genesis configuration.
    ///                      Must reside in the same directory as the binary is launched.
    /// - public_uciv: The public universal cast-as-intended verifiability (UCIV) information.
    /// - public_key: The public key used for encrypting votes.
    ///
    /// Panics if the content of the configured genesis file is not valid w.r.t. a genesis block.
    ///
    pub fn new(genesis_file_name: &str, public_uciv_file_name: &str, public_key_file_name: &str) -> Self {
        // Read the genesis file
        let genesis_str_path = "./".to_owned() + genesis_file_name;
        let genesis_path = Path::new(genesis_str_path.as_str());
        if ! genesis_path.exists() {
            panic!("Missing genesis file at ./{}", genesis_file_name);
        }

        let mut file = File::open("./".to_owned() + genesis_file_name).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let genesis_data: GenesisData = match serde_json::from_str(&contents) {
            Ok(genesis_data) => {
                trace!("Read genesis configuration: {:?}", genesis_data);
                genesis_data
            },
            Err(e) => {
                panic!("Failed to transform file {:?} into Genesis configuration: {:?}", file, e);
            }
        };

        // read the UCIV information from the path provided
        let uciv_str_path = "./".to_owned() + public_uciv_file_name;
        let uciv_path = Path::new(uciv_str_path.as_str());
        if ! uciv_path.exists() {
            panic!("Missing public UCIV file at ./{}", public_uciv_file_name);
        }

        trace!("Reading public UCIV information from {}", public_uciv_file_name);
        let mut public_uciv_file = File::open("./".to_owned() + public_uciv_file_name).unwrap();
        let mut public_uciv_buffer = String::new();
        public_uciv_file.read_to_string(&mut public_uciv_buffer).unwrap();

        let public_uciv: Vec<ImageSet> = match serde_json::from_str(&public_uciv_buffer) {
            Ok(public_uciv_data) => {
                public_uciv_data
            }
            Err(e) => {
                panic!("Failed to transform file {:?} into ImageSet: {:?}", public_uciv_file, e);
            }
        };

        // read public key from path provided
        let public_key_str_path = "./".to_owned() + public_key_file_name;
        let public_key_path = Path::new(public_key_str_path.as_str());
        if ! public_key_path.exists() {
            panic!("Missing public key file at ./{}", public_key_file_name);
        }

        trace!("Reading public key from {}", public_key_file_name);
        let public_key = PublicKey::new(public_key_file_name);

        assert!(genesis_data.version.len() > 0, "Version parameter must be specified");
        assert!(genesis_data.clique.block_period > 0, "Clique block period must be greater than zero");
        assert!(genesis_data.sealer.len() > 0, "There must be at least a single sealer");

        // TODO: if only one sealer -> what should the signer_limit value be?

        Genesis {
            version: genesis_data.version,
            clique: genesis_data.clique,
            sealer: genesis_data.sealer,
            public_key,
            public_uciv
        }
    }


}