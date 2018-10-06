//! `node_rs` provides a binary to run a binary vote on a permissioned blockchain.
//!
//! **[WIP] This library is still work in progress and not audited in any way. Use at your own risk.**
//!
//! ## Usage
//!
//! ```sh
//! Run a node of a permissioned e-voting blockchain
//!
//! USAGE:
//!     node_rs [FLAGS] [SUBCOMMAND]
//!
//! FLAGS:
//!     -h, --help         Prints help information
//!     -V, --version      Prints version information
//!     -v, --verbosity    Turn up the verbosity of the log output
//!
//! SUBCOMMANDS:
//!     help     Prints this message or the help of the given subcommand(s)
//!     start    Start a new node
//! ```
//!
//! ## Requirements
//!
//! **Please note the requirements below for a successful setup of the network.**
//!
//!
//! ### Genesis
//! In order to run a node of a permissioned blockchain, a configuration
//! for the network must be configured. Consistently with other blockchains,
//! this configuration is defining the hash of the initial block.
//! It is stored in the same directory as the binary is located and must be
//! called `genesis.json`.
//!
//! An example of such a file can look like the following:
//!
//! ```json
//! {
//!   "version": "0.1.0",
//!   "clique": {
//!     "block_period": 15,
//!     "signer_limit": 2
//!   },
//!   "sealer": [
//!     "127.0.0.1:9000",
//!     "127.0.0.1:9001",
//!     "127.0.0.1:9002"
//!   ]
//! }
//! ```
//!
//! **Parameters**:
//! * `version`: Version specifies the version of the blockchain binary which is used.
//! * `clique`: This blockchain uses a simplified implementation of the Clique
//!    protocol as initially proposed to the Ethereum blockchain as
//!    Proof-of-Authority [sybil control mechanism](https://twitter.com/el33th4xor/status/1006931658338177024?s=12).
//!     * `block_period`: This is the period until a new block is generated
//!     * `signer_limit`: How many epochs a node must wait until its his turn again to mint a new block
//! * `sealer`: A set of IPv4 addresses of nodes which form the network.
//!
//! *Note: In order to let multiple nodes build a network successfully, this
//! configuration must be equal, as its hash is used in the Genesis block.
//! Nodes with different genesis files (even a single empty line) will
//! not build a canonical chain!*
//!
//! ### Public Key
//!
//! In order to count encrypted votes in a [homomorphic](https://en.wikipedia.org/wiki/Homomorphic_encryption) fashion,
//! each node needs a copy of the same public key. A keypair can be generated
//! using the binary of [generator_rs](https://github.com/provotum/generator-rs).
//! The obtained copy of a public key must be stored in `public_key.json` in the
//! same directory as the binary.
//!
//! ### Public UCIV
//!
//! Allowing a voter to be sure, that his encrypted vote still represents
//! his actual voting choice is known as `individual cast-as-intended verifiability`.
//! Allowing anyone to proof that each vote represents what a voter intended
//! it to be, is called `universal cast-as-intended verifiability (UCIV)` according
//! to [this paper](https://fc16.ifca.ai/voting/papers/EGHM16.pdf).
//!
//! This blockchain aims at providing `UCIV` by utilizing a
//! zero-knowledge proof. In order to verify that each vote is indeed
//! cast as intended, you further need to provide a `public_uciv.json` file
//! in the same directory as the binary is invoked. As the public key,
//! this information can be generated using [generator_rs](https://github.com/provotum/generator-rs).
//!
//! ## Running a permissioned Voting network
//!
//! Now, once you have met the requirements stated above,
//! you can start the permissioned voting blockchain.
//! To let the nodes reach consensus in an early stage, follow
//! the procedure outlined below:
//!
//! :warning: :warning: :warning:
//!
//! **This example requires to have `genesis.json` setup as in the above example.**
//!
//! 1. Start your first node by running `node_rs -v start -s 127.0.0.1:9000 127.0.0.1:3000`.
//!    The flag `-v` will let you output debug information, increase the
//!    verbosity using `-vv` to also show more detailed statements.
//!    `-s` tells the node to start minting blocks.
//!    Provide as first argument the first IP address of the `sealer` key
//!    of `genesis.json`. Specify as second argument any IPv4 address
//!    on which the node will listen for RPC connections of a client.
//! 2. **Important**: Let the node mint the first block until you start
//!    a further one!
//!
//!    Start your second node, this time by adding the flags `-p` to the
//!    command, yielding `node_rs -v start -s -p 127.0.0.1:9001 127.0.0.1:3001`.
//!    `-p` will tell the node to first obtain a copy of the already running
//!    nodes. If their canonical chain are longer, they will replace
//!    the chain of the node you've just started.
//!    **Note**: You will likely some warning output, telling you that
//!    connection attempts to other nodes failed. This is expected, as
//!    currently you've not yet started all nodes which are defined in the
//!    `genesis.json`.
//! 3. **Important**: Let the _second_ node mint the second block until you start
//!    a further one!
//!
//!    Eventually, after the first two nodes have exchanged their initial blocks,
//!    you are ready to start the third one. For that, run `node_rs -v start -s -p 127.0.0.1:9002 127.0.0.1:3002`.
//!
//!
//! That's it, now you should see new blocks being minted every `block_period` seconds.
//!
//!
//! ## Submitting Data to the Voting blockchain
//! In order to submit votes to the chain,
//! please refer to the readme of [client_rs](https://github.com/provotum/client-rs).

#![crate_type = "lib"]
#![crate_name = "node_rs"]

extern crate futures;
extern crate bytes;
extern crate rand;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate sha1;
extern crate bincode;

extern crate num;
extern crate crypto_rs;

/// Holds all functionality related to the blockchain itself.
pub mod chain;

/// Holds all functionality related to the blockchain configuration, e.g. Genesis.
pub mod config;

/// Holds all functionality related to the networking stuff.
pub mod p2p;

/// Holds all functionality related to the protocol used to communicate blocks and transactions.
pub mod protocol;