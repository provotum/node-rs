extern crate tokio_core;
extern crate tokio_timer;
extern crate futures;
extern crate getopts;
extern crate node;

use futures::Future;
use tokio_core::reactor::Core;

use std::env;
use std::time::Duration;
use std::net::SocketAddr;
use std::vec::Vec;
use std::borrow::Borrow;
use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time};


use node::p2p::node::Node;

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn do_work(me: SocketAddr, msg: Option<String>, target_peer: Option<String>) {
    let mut peers_vec = Vec::new();

    match target_peer {
        Some(target_peer) => {
            println!("Should connect to target peer {:?}", target_peer);
            peers_vec = vec![target_peer]
        }
        None => (),
    }

    //let peers = vec!["127.0.0.1:12345", "127.0.0.1:12346", "127.0.0.1:12347"]

    // parse peers
    let peers = peers_vec.into_iter().map(|p| p.parse().unwrap());


    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let node = Node::new(me);
    let f = node.run(handle.clone(), peers);
    let timer = node.timer.clone();

    match msg {
        Some(msg) => {
            let f = timer.sleep(Duration::from_secs(3))
                .and_then(move |_| {
                        node.broadcast(msg);
                    Ok(())
                });
            handle.spawn(f.then(|_| Ok(())));
        }
        None => (),
    }

    // blocks thread
    core.run(f).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("a", "", "set the host address (required)", "ADDR");
    opts.optopt("t", "", "peer to which to connect to", "peer");
    opts.optopt("b", "", "broadcast a message after 1 second", "MSG");
    opts.optflag("h", "", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    match matches.opt_str("a") {
        Some(addr) => do_work(addr.parse().unwrap(), matches.opt_str("b"), matches.opt_str("t")),
        None => print_usage(&program, opts),
    }
}
