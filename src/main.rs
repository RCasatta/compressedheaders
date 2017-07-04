pub mod bitcoin;
pub mod server;

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
use std::time::{Duration, Instant};
use std::sync::{Mutex, Arc};

use bitcoin::header::BlockHeader;
use bitcoin::rpc::BlockHeaderRpcResponse;


fn main() {
    let start = Instant::now();

    let username = env::args().nth(1).unwrap();
    let password = Some(env::args().nth(2).unwrap());

    let genesis_block_hash = String::from("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");  //genesis hash
    let mut block_hash : String = genesis_block_hash;

    let mut boxed_block_headers  = Vec::with_capacity(1000000) ;
    let counter = Arc::new(Mutex::new(boxed_block_headers));

    let cloned_counter = counter.clone();
    let second_cloned_counter = counter.clone();
    thread::spawn(move || {
        server::start(cloned_counter);
    });



    loop {
        let block_header_rpc_response : BlockHeaderRpcResponse = bitcoin::rpc::get_block_header(block_hash.clone(), username.clone(), password.clone()).unwrap();
        //println!("{:?}", block_header_rpc_response);
        let block_header_rpc : bitcoin::rpc::BlockHeaderRpc = block_header_rpc_response.result;
        let height = block_header_rpc.height.clone() as usize;
        if height%1000==0 {
            println!("Reached block {} with hash {} elapsed {}", height, block_hash, start.elapsed().as_secs());
        }

        let block_hash_option = block_header_rpc.nextblockhash.clone();


        let mut block_headers = second_cloned_counter.lock().unwrap();
        match block_hash_option {
            Some(val) => {
                block_hash = val;

                let block_header = BlockHeader::from_block_header_rpc(block_header_rpc);

                while block_headers.len() < height + 1 {
                    block_headers.push(None);
                }

                block_headers[height]=Some(block_header);
            },
            None => {
                println!("Last block height {} hash {}, going to sleep for a while", height, block_hash);
                thread::sleep(Duration::from_secs(10));
                block_hash = block_headers[height-10].unwrap().hash();
                println!("restarting from hash {} (10 blocks ago)", block_hash);
            }
        }
        //second_cloned_counter.unlock();
        //println!();
    }

}

