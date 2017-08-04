pub mod bitcoin;
pub mod server;
pub mod client;
pub mod util;

extern crate serde;
extern crate hyper;
extern crate serde_json;
extern crate futures;
extern crate tokio_core;
extern crate crypto;

#[macro_use]
extern crate serde_derive;

use std::env;
use std::thread;
use std::sync::{Mutex, Arc};

fn main() {
    let host = env::args().nth(1).unwrap();
    let username = env::args().nth(2).unwrap();
    let password = Some(env::args().nth(3).unwrap());

    let block_headers  = Vec::with_capacity(1000000) ;
    let block_headers_arc = Arc::new(Mutex::new(block_headers));

    let block_headers_arc_1 = block_headers_arc.clone();
    thread::spawn(move || {
        server::start(block_headers_arc_1);
    });

    let block_headers_arc_2 = block_headers_arc.clone();
    let c = thread::spawn(move || {
        client::start(block_headers_arc_2, host, username, password);
    });

    c.join();
}

