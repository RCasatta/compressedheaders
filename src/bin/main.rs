extern crate compressedheaders;
extern crate crypto;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;


use std::thread;
use std::sync::{Arc, Mutex};
use compressedheaders::{server, client};
use compressedheaders::bitcoin::Config;

fn main() {
    let config = Config::read().unwrap();

    let block_headers_bytes = Vec::new();
    let block_headers_bytes_arc = Arc::new(Mutex::new(block_headers_bytes));

    let block_headers_bytes_arc_1 = block_headers_bytes_arc.clone();
    thread::spawn(move || {
        server::start(block_headers_bytes_arc_1);
    });

    let block_headers_bytes_arc_2 = block_headers_bytes_arc.clone();
    let c = thread::spawn(move || {
        client::start(block_headers_bytes_arc_2, &config);
    });

    let _ = c.join();
}
