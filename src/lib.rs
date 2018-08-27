#![crate_type = "lib"]
#![crate_name = "node"]

extern crate env_logger;

extern crate futures;
extern crate bytes;
extern crate getopts;
extern crate rand;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate simple_logger;

pub mod p2p;
pub mod protocol;
pub mod config;