use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use util::hex::ToHex;
use bitcoin::header::BlockHeader;
use bitcoin;


pub fn start(block_headers : Arc<Mutex<Vec<Option<BlockHeader>>>>, host : String, username : String, password : Option<String>) {
    let start = Instant::now();

    let genesis_block_hash = String::from("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");  //genesis hash
    let mut block_hash : String = genesis_block_hash.clone();
    let mut last_block : usize = 0;
    let mut min_block_hash : String = genesis_block_hash;

    loop {
        let r = bitcoin::rpc::get_block_header(block_hash.clone(), host.clone(), username.clone(), password.clone());
        match r {
            Ok(block_header_rpc_response) => {
                //let block_header_rpc_response : BlockHeaderRpcResponse = r.unwrap();
                //println!("{:?}", block_header_rpc_response);
                let block_header_rpc : bitcoin::rpc::BlockHeaderRpc = block_header_rpc_response.result;
                let height = block_header_rpc.height.clone() as usize;
                if last_block==0 && height%1000==0 {
                    println!("Block #{} with hash {} elapsed {} seconds", height, block_hash, start.elapsed().as_secs());
                }

                let block_hash_option = block_header_rpc.nextblockhash.clone();
                let sleep;

                {
                    let mut block_headers = block_headers.lock().unwrap();
                    sleep = match block_hash_option {
                        Some(val) => {
                            block_hash = val;
                            let block_header = BlockHeader::from_block_header_rpc(block_header_rpc);
                            while block_headers.len() < height + 1 {
                                block_headers.push(None);
                            }
                            block_headers[height] = Some(block_header);
                            let mut hash = block_header.hash();
                            hash.reverse();
                            if min_block_hash> hash.to_hex() {
                                min_block_hash = hash.to_hex();
                                println!("Block #{} with hash {} is the min!", height, min_block_hash );
                            }

                            false
                        },
                        None => {
                            if height != last_block {
                                println!("Block #{} with hash {}", height, block_hash );
                            }
                            last_block = height;
                            block_hash = block_headers[height - 144].unwrap().hash().to_hex();   //going back 144 blocks to support reorgs one day long

                            true
                        }
                    };
                }  //releasing lock

                if sleep {
                    thread::sleep(Duration::from_secs(60));
                }
            },
            Err(e) =>{
                println!("{:?}",e);
                thread::sleep(Duration::from_secs(30));
            }
        }


    }
}