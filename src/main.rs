extern crate clap;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate node_rs;
extern crate pretty_env_logger;

use clap::{App, Arg, SubCommand};
use env_logger::Target;
use node_rs::config::genesis::Genesis;
use node_rs::p2p::node::Node;
use std::net::SocketAddr;
use std::path::Path;

fn main() {
    let matches = App::new("node_rs")
        .version("0.1.0")
        .author("Raphael Matile <raphael.matile@gmail.com>")
        .about("Run a node of a permissioned e-voting blockchain")
        .arg(Arg::with_name("verbosity")
            .help("Turn up the verbosity of the log output")
            .short("v")
            .long("verbosity")
            .multiple(true)
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Start a new node")
                .arg(Arg::with_name("listen_address")
                    .required(true)
                    .takes_value(true)
                    .index(1)
                    .help("The address on which the started node should listen for incoming connections of other nodes. In the format <IPv4>:<Port>")
                )
                .arg(Arg::with_name("rpc_listen_address")
                    .required(true)
                    .takes_value(true)
                    .index(2)
                    .help("The address on which the started node should listen for RPC connections from clients. In the format <IPv4>:<Port>")
                )
                .arg(Arg::with_name("ping")
                    .short("p")
                    .long("ping")
                    .help("Ping all nodes defined in the genesis block")
                )
                .arg(Arg::with_name("sign")
                    .short("s")
                    .long("sign")
                    .help("Sign blocks after starting the node")
                )
        )
        .get_matches();

    let log_filter;
    match matches.occurrences_of("verbosity") {
        0 => { log_filter = "node_rs=info" }
        1 => { log_filter = "node_rs=debug" }
        2 => { log_filter = "node_rs=trace" }
        _ => { log_filter = "node_rs=trace" }
    }

    // init logger
    pretty_env_logger::formatted_builder().unwrap()
        //let's just set some random stuff.. for more see
        //https://docs.rs/env_logger/0.5.0-rc.1/env_logger/struct.Builder.html
        .target(Target::Stdout)
        .parse(log_filter)
        .init();


    match matches.subcommand_name() {
        Some("start") => {
            let subcommand_matches = matches.subcommand_matches("start").unwrap();

            let listen_address: SocketAddr = subcommand_matches.value_of("listen_address").unwrap().parse::<SocketAddr>().unwrap();
            let rpc_listen_address: SocketAddr = subcommand_matches.value_of("rpc_listen_address").unwrap().parse::<SocketAddr>().unwrap();

            let has_ping: bool = subcommand_matches.is_present("ping");
            let has_sign: bool = subcommand_matches.is_present("sign");

            // get configuration
            let genesis_path = Path::new("genesis.json");
            if !genesis_path.exists() {
                error!("Genesis configuration not found at './genesis.json'");
                std::process::exit(1);
            }

            let public_key_path = Path::new("public_key.json");
            if !public_key_path.exists() {
                error!("Public key not found at './public_key.json'");
                std::process::exit(1);
            }

            let public_uciv = Path::new("public_uciv.json");
            if !public_uciv.exists() {
                error!("Public universal cast-as-intended verifiability (UCIV) configuration not found at './public_uciv.json'");
                std::process::exit(1);
            }

            let genesis = Genesis::new("genesis.json", "public_uciv.json", "public_key.json");
            let mut node = Node::new(listen_address, rpc_listen_address, genesis);

            node.listen();
            node.listen_rpc();

            if has_ping {
                node.request_chain_copy();
            }

            if has_sign {
                node.sign();
            }
        }
        Some(&_) | None => {
            // an unspecified or no command was used
            println!("{}", matches.usage())
        }
    }
}
