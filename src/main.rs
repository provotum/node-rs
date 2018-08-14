extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate tokio;
extern crate node;

use std::env;
use node::p2p::thread::ThreadPool;
use node::p2p::network_service::NetworkService;

fn main() {

    let pool = ThreadPool::new(4);
    pool.execute(|| {
        NetworkService::new("127.0.0.1:6142")
            .listen();
    });


}