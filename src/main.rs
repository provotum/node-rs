extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate tokio;
extern crate node;

use std::env;
use std::{thread, time};
use node::p2p::network_service::NetworkService;

fn main() {

    let network_service = NetworkService::new("127.0.0.1:6142");
    network_service.listen();
    network_service.connect("127.0.0.1:6142");

    let ten_millis = time::Duration::from_secs(30);
    thread::sleep(ten_millis);
    network_service.send();

}