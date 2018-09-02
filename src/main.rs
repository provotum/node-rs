extern crate futures;
extern crate getopts;
extern crate node;

extern crate log;
extern crate simple_logger;

use node::p2p::node::Node;
use std::env;
use std::vec::Vec;
use node::config::genesis::Genesis;



fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} listen_address rpc_listen_address [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // init logger
    simple_logger::init().unwrap();

    // get configuration
    let genesis = Genesis::new("genesis.json");

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("c", "connect", "the address to which to connect");
    opts.optflag("h", "", "print this help menu");

    if args.len() < 3 {
        print_usage(&program, &opts);
        return;
    }

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return;
    }

    let listen_addr = args[1].clone();
    let rpc_listen_addr = args[2].clone();
    let mut node = Node::new(listen_addr.parse().unwrap(), rpc_listen_addr.parse().unwrap(), genesis);

    node.listen();
    node.listen_rpc();

    if matches.opt_present("c") {
        node.connect();
    }
}
