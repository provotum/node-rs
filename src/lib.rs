#![crate_type = "lib"]
#![crate_name = "node"]

extern crate env_logger;

extern crate futures;
extern crate bytes;
extern crate getopts;
extern crate rand;
extern crate uuid;

pub mod p2p;
pub mod protocol;