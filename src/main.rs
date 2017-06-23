
extern crate serde;
extern crate hyper;
extern crate serde_json;
extern crate bitcoin;
extern crate crypto;
extern crate futures;
extern crate tokio_core;

#[macro_use]
extern crate serde_derive;

use std::env;
use std::thread;
use std::convert::AsMut;
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde_json::{Value, Error};
use hyper::Method;
use hyper::Body;
use hyper::HttpVersion;
use hyper::header::{Headers, Authorization, Basic, ContentLength};
use hyper::server::{Http, Request, Response, Service};
use hyper::client::FutureResponse;
use tokio_core::reactor::Core;
use futures::{Future, Stream};
use hyper::Client;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeaderRpcResponse {
    result : BlockHeaderRpc,
    id: String,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeaderRpc {
    hash: String,
    height: u32,
    version: u32,
    nonce: u32,
    versionHex: String,
    merkleroot: String,
    time: u32,
    mediantime: u32,
    bits: String,
    difficulty: f64,
    chainwork: String,
    nextblockhash: Option<String>,
    previousblockhash: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub struct BlockHeader {
    pub version: [u8; 4], // The protocol version. Should always be 1.
    pub prev_blockhash: [u8; 32], // Reference to the previous block in the chain
    pub merkle_root: [u8; 32], /// The root hash of the merkle tree of transactions in the block
    pub time: [u8; 4], // The timestamp of the block, as claimed by the mainer
    pub bits: [u8; 4], // The target value below which the blockhash must lie, encoded as a a float (with well-defined rounding, of course)
    pub nonce: [u8; 4], // The nonce, selected to obtain a low enough blockhash
}

struct HelloWorld;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(HelloWorld)).unwrap();
    server.run().unwrap();

    let now = Instant::now();

    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let mut headers = Headers::new();

    let username = env::args().nth(1).unwrap();
    let password = Some(env::args().nth(2).unwrap());
    headers.set(Authorization(Basic {
        username: username,
        password: password
    }));
    let mut block_hash : String = String::from("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");  //genesis hash
    //let mut block_hash : String = String::from("0000000000000000006c539c722e280a0769abd510af0073430159d71e6d7589");  // 470 000
    let mut block_headers  = Vec::with_capacity(1000000) ;

    loop {

        let request_body_string: String = format!("{{\"jsonrpc\":\"1.0\",\"id\":\"{}\",\"method\":\"{}\",\"params\":[\"{}\"]}}", 0, "getblockheader", block_hash);

        let mut r : hyper::Request = Request {
            method: hyper::Method::Post,
            uri: "http://localhost:8332".parse().unwrap(),
            version: HttpVersion::default(),
            headers: headers.clone(),
            body: None,
            is_proxy: false,
            remote_addr: None,
        };
        r.set_body(Body::from(request_body_string));

        let mut future : FutureResponse = client.request(r);
        future.and_then(|res| {
            println!("Response: {}", res.status());

            /*res.body().for_each(|chunk| {
                io::stdout()
                    .write_all(&chunk)
                    .map(|_| ())
                    .map_err(From::from)
            })*/
        });

        //assert_eq!(res.status, hyper::Ok);
        let mut buffer = String::new();  // {"result":{"hash":"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f","confirmations":467930,"height":0,"version":1,"versionHex":"00000001","merkleroot":"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b","time":1231006505,"mediantime":1231006505,"nonce":2083236893,"bits":"1d00ffff","difficulty":1,"chainwork":"0000000000000000000000000000000000000000000000000000000100010001","nextblockhash":"00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048"},"error":null,"id":"curltext"}
        //res.read_to_string(&mut buffer);
        //println!("buffer {}", buffer);

        let block_header_rpc_response : BlockHeaderRpcResponse = serde_json::from_str(&buffer).unwrap();
        //println!("block_header_rpc_response {:?}", block_header_rpc_response);

        let block_header_rpc : BlockHeaderRpc = block_header_rpc_response.result;
        let height = block_header_rpc.height.clone() as usize;
        if height%1000==0 {
            println!("Reached block {} with hash {}", height, block_hash);
        }

        let block_hash_option = block_header_rpc.nextblockhash.clone();
        match block_hash_option {
            Some(val) => {
                block_hash = val;

                let block_header = BlockHeader::from_block_header_rpc(block_header_rpc);

                //println!("block_header {:?}",block_header);
                //println!("block_header_hex {:x}", ByteBuf(&block_header.as_bytes()));
                //println!("block_header_compressed_hex {:x}", ByteBuf(&block_header.as_compressed_bytes()));

                /*
                let mut sha2 = Sha256::new();
                sha2.input(&block_header.as_bytes());
                let mut first : [u8;32] = [0;32];
                sha2.result(&mut first);
                let mut sha2b = Sha256::new();
                sha2b.input(&first);
                println!("sha256 double hash of block_header bytes {}", sha2b.result_str());
                */

                //println!("sha256 double hash of block_header bytes {}", block_header.hash() );

                while block_headers.len() < height + 1 {
                    block_headers.push(None);
                }

                block_headers[height]=Some(block_header);
            },
            None => {
                println!("Last block height {} hash {}, going to sleep for a while", height, block_hash);

                thread::sleep(std::time::Duration::from_secs(10));
                block_hash = block_headers[height-10].unwrap().hash();

                println!("restarting from hash {} (10 blocks ago)", block_hash);
            }
        }



        println!();

    }

    println!("{}", now.elapsed().as_secs());

    //Server::http("127.0.0.1:8888").unwrap().handle(hello).unwrap();
}


const PHRASE: &'static str = "Hello, World!";

impl Service for HelloWorld {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _req: Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_body(PHRASE)
        )
    }
}


fn transform_u32_to_reversed_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b4, b3, b2, b1]
}


impl BlockHeader {
    // `Self` is the implementor type: `Sheep`.
    fn new() -> BlockHeader {
        BlockHeader { version: [0;4], prev_blockhash: [0;32], merkle_root: [0;32], time: [0;4], bits: [0;4], nonce: [0;4]  }
    }

    fn as_bytes(&self) -> [u8;80] {
        let mut result : [u8;80] = [0;80];
        let mut vec : Vec<u8> = Vec::new();
        vec.extend_from_slice(&self.version);
        vec.extend_from_slice(&self.prev_blockhash);
        vec.extend_from_slice(&self.merkle_root);
        vec.extend_from_slice(&self.time);
        vec.extend_from_slice(&self.bits);
        vec.extend_from_slice(&self.nonce);
        for (idx, el) in vec.into_iter().enumerate() {
            result[idx]=el;
        }
        result
    }

    fn as_compressed_bytes(&self) -> [u8;48] {
        let mut result : [u8;48] = [0;48];
        let all = &self.as_bytes();
        for i in 0..48 {
            if i < 4 {
                result[i] = all[i];
            } else {
                result[i] = all[32+i];
            }
        }
        result
    }

    fn from_bytes(bytes : [u8;80]) -> BlockHeader {
        BlockHeader {
            version:        clone_into_array(&bytes[0 .. 4]),
            prev_blockhash: clone_into_array(&bytes[4 .. 36]),
            merkle_root:    clone_into_array(&bytes[36 .. 68]),
            time:           clone_into_array(&bytes[68 .. 72]),
            bits:           clone_into_array(&bytes[72 .. 76]),
            nonce:          clone_into_array(&bytes[76 .. 80])
        }
    }

    fn from_compressed_bytes(bytes : [u8;48], prev_blockhash : [u8;32]) -> BlockHeader {
        BlockHeader {
            version:        clone_into_array(&bytes[0 .. 4]),
            prev_blockhash: clone_into_array(&prev_blockhash),
            merkle_root:    clone_into_array(&bytes[4 .. 36]),
            time:           clone_into_array(&bytes[36 .. 40]),
            bits:           clone_into_array(&bytes[40 .. 44]),
            nonce:          clone_into_array(&bytes[44 .. 48])
        }
    }

    fn from_block_header_rpc(block_header_rpc : BlockHeaderRpc) -> BlockHeader {
        //let nextblockhash = q["nextblockhash"].as_str().unwrap();
        let version_hex = &block_header_rpc.versionHex;
        let previous_block_hash = match block_header_rpc.previousblockhash {
            Some(r) => r,
            _ => String::from("0000000000000000000000000000000000000000000000000000000000000000")
        };
        let merkle_root = &block_header_rpc.merkleroot;
        let time = block_header_rpc.time;
        let bits = &block_header_rpc.bits;
        let nonce = block_header_rpc.nonce;

        BlockHeader {
            version:  to_reversed_array_of_4(from_hex(version_hex).unwrap() ),
            prev_blockhash:  to_reversed_array_of_32(from_hex(&previous_block_hash).unwrap() ),
            merkle_root: to_reversed_array_of_32(from_hex(merkle_root).unwrap()),
            time: transform_u32_to_reversed_array_of_u8(time),
            bits: to_reversed_array_of_4(from_hex(bits).unwrap()),
            nonce: transform_u32_to_reversed_array_of_u8(nonce)
        }
    }

    fn hash(&self) -> String {
        let mut sha2 = Sha256::new();
        sha2.input(&self.as_bytes());
        let mut first : [u8;32] = [0;32];
        sha2.result(&mut first);
        let mut sha2b = Sha256::new();
        sha2b.input(&first);

        let mut bytes : Vec<u8>= from_hex(&sha2b.result_str()).unwrap();
        bytes.reverse();

        format!("{:x}",ByteBuf(bytes.as_ref()))

    }
}


fn clone_into_array<A, T>(slice: &[T]) -> A
    where A: Sized + Default + AsMut<[T]>,
          T: Clone
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

fn to_reversed_array_of_4(mut vec : Vec<u8> ) -> [u8;4] {
    let mut result : [u8;4]= [0;4];
    vec.reverse();
    for (idx, el) in vec.into_iter().enumerate() {
        result[idx]=el;
    }

    result
}

fn to_reversed_array_of_32(mut vec : Vec<u8> ) -> [u8;32] {
    let mut result : [u8;32] = [0;32];
    vec.reverse();
    for (idx, el) in vec.into_iter().enumerate() {
        result[idx]=el;
    }
    result
}

fn from_hex<'a>(hex_str: &'a str) -> Result<Vec<u8>, Error> {
    // This may be an overestimate if there is any whitespace
    let mut b = Vec::with_capacity(hex_str.len() / 2);
    let mut modulus = 0;
    let mut buf = 0;

    for (idx, byte) in hex_str.bytes().enumerate() {
        buf <<= 4;

        match byte {
            b'A'...b'F' => buf |= byte - b'A' + 10,
            b'a'...b'f' => buf |= byte - b'a' + 10,
            b'0'...b'9' => buf |= byte - b'0',
            b' '|b'\r'|b'\n'|b'\t' => {
                buf >>= 4;
                continue
            }
            _ => {
                let ch = hex_str[idx..].chars().next().unwrap();
                panic!("woooow")  //FIX error
            }
        }

        modulus += 1;
        if modulus == 2 {
            modulus = 0;
            b.push(buf);
        }
    }

    match modulus {
        0 => Ok(b.into_iter().collect()),
        _ => panic!("woooow") //FIX error
    }
}

struct ByteBuf<'a>(&'a [u8]);

impl<'a> std::fmt::LowerHex for ByteBuf<'a> {
    fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for byte in self.0 {
            try!( fmtr.write_fmt(format_args!("{:02x}", byte)));
        }
        Ok(())
    }
}
