#![crate_type = "lib"]
#![crate_name = "node"]

extern crate env_logger;

extern crate futures;
extern crate bytes;
extern crate tokio_core;
extern crate tokio_timer;
extern crate getopts;
extern crate rand;
extern crate uuid;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

pub mod p2p;
pub mod protocol;