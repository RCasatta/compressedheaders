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
use std::fs::File;
use std::io::Read;

fn main() {
    let (host, username, password) = read_config().unwrap();

    let block_headers_bytes  = Vec::new() ;
    let block_headers_bytes_arc = Arc::new(Mutex::new(block_headers_bytes));

    let block_headers_bytes_arc_1 = block_headers_bytes_arc.clone();
    thread::spawn(move || {
        server::start(block_headers_bytes_arc_1);
    });

    let block_headers_bytes_arc_2 = block_headers_bytes_arc.clone();
    let c = thread::spawn(move || {
        client::start(block_headers_bytes_arc_2, host, username, password);
    });

    let _ = c.join();
}

fn read_config() -> Result<(String , String, Option<String>), &'static str> {
    let mut host : Option<String> = Some(String::from("http://localhost:8332"));
    let mut username : Option<String> = None;
    let mut password : Option<String> = None;

    match env::home_dir() {
        Some(path) => {
            let paths = vec!["/Library/Application Support/Bitcoin/bitcoin.conf", "/.bitcoin/bitcoin.conf", "\\AppData\\Roaming\\Bitcoin\\bitcoin.conf"];
            for filename in paths {
                let full_path = format!("{}{}",path.display(), filename);
                let f = File::open(full_path.clone());
                match f {
                    Ok(mut f) => {
                        let mut contents = String::new();
                        f.read_to_string(&mut contents)
                            .expect("something went wrong reading the file");
                        println!("Found config file at {}", full_path );
                        let x = contents.split("\n");
                        for el in x {
                            let x = el.replace(" ", "");
                            if x.starts_with("rpcuser=") {
                                username=Some(String::from(&x[8..]));
                            }
                            if x.starts_with("rpcpassword=") {
                                password=Some(String::from(&x[12..]));
                            }
                            if x.starts_with("rpchost=") {
                                host=Some(String::from(&x[8..]));
                            }
                        }

                    },
                    Err(_) => (),
                }
            }
        },
        None => println!("Impossible to get your home dir!"),
    }

    match host.is_some() && username.is_some() {
        true => Ok((host.unwrap(),username.unwrap(),password)),
        false => Err("Cannot find rpcuser and rpcpassword"),
    }

}