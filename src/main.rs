
extern crate serde;
extern crate hyper;
extern crate serde_json;
extern crate bitcoin;
extern crate crypto;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use serde_json::{Value, Error};
use hyper::client::Client;
use hyper::header::{Headers, Authorization, Basic, ContentLength};
use std::env;
use hyper::server::{Server, Request, Response};


fn main() {

    let string_block = "{\"result\":{\"hash\":\"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f\",\"confirmations\":467930,\"height\":0,\"version\":1,\"versionHex\":\"00000001\",\"merkleroot\":\"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b\",\"time\":1231006505,\"mediantime\":1231006505,\"nonce\":2083236893,\"bits\":\"1d00ffff\",\"difficulty\":1,\"chainwork\":\"0000000000000000000000000000000000000000000000000000000100010001\",\"nextblockhash\":\"00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048\"},\"error\":null,\"id\":\"curltext\"}";
    //let b : BlockHeader = parse_block(string_block).unwrap();
    //println!("{:?}",b);
    let now = Instant::now();
    let client = Client::new();
    let mut headers = Headers::new();
    let username = env::args().nth(1).unwrap();
    println!("username {}", username);
    let password = Some(env::args().nth(2).unwrap());
    println!("password {:?}", password);

    headers.set(Authorization(Basic {
        username: username,
        password: password
    }));

    let mut block_hash : String = String::from("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");  //genesis hash

    for x in 0..2 {
        //println!("block_hash {:?}",block_hash);
        let request: String = format!("{{\"jsonrpc\":\"1.0\",\"id\":\"curltext\",\"method\":\"getblockheader\",\"params\":[\"{}\"]}}", block_hash);
        //println!("request {}", request);
        let mut res = client.post("http://localhost:8332")
            .headers(headers.clone())
            .body(request.as_str())
            .send()
            .unwrap();

        assert_eq!(res.status, hyper::Ok);

        let mut buffer = String::new();  // {"result":{"hash":"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f","confirmations":467930,"height":0,"version":1,"versionHex":"00000001","merkleroot":"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b","time":1231006505,"mediantime":1231006505,"nonce":2083236893,"bits":"1d00ffff","difficulty":1,"chainwork":"0000000000000000000000000000000000000000000000000000000100010001","nextblockhash":"00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048"},"error":null,"id":"curltext"}
        res.read_to_string(&mut buffer);
        println!("buffer {} {}", x, buffer);


        let r = serde_json::from_str(&buffer);
        let v : Value = r.unwrap();

        let ref q = v["result"];
        let nextblockhash = q["nextblockhash"].as_str().unwrap();

        let versionHex = q["versionHex"].as_str().unwrap();
        let previousblockhash = match q["previousblockhash"].as_str() {
            Some(r) => r,
            _ => "0000000000000000000000000000000000000000000000000000000000000000"
        };
        let merkleroot = q["merkleroot"].as_str().unwrap();
        let time = q["time"].as_u64().unwrap() as u32;
        let bits = q["bits"].as_str().unwrap();
        let nonce = q["nonce"].as_u64().unwrap() as u32;


        //let result = extract_next(buffer.as_str()).unwrap();
        let result = String::from(nextblockhash);

        println!("result {:?}", result);

        block_hash = result;

        let b : BlockHeader = BlockHeader {
            version:  to_reversed_array_of_4(from_hex(versionHex).unwrap() ),
            prev_blockhash:  to_reversed_array_of_32(from_hex(previousblockhash).unwrap() ),
            merkle_root: to_reversed_array_of_32(from_hex(merkleroot).unwrap()),
            time: transform_u32_to_reversed_array_of_u8(time),
            bits: to_reversed_array_of_4(from_hex(bits).unwrap()),
            nonce: transform_u32_to_reversed_array_of_u8(nonce)
        };

        println!("{:?}",b);
        println!("blockheader {:x}", ByteBuf(&b.as_bytes()));

        let mut sha2 = Sha256::new();
        sha2.input(&b.as_bytes());
        println!("sha2 {}", sha2.result_str());
        let mut first : [u8;32] = [0;32];
        sha2.result(&mut first);
        let mut sha2b = Sha256::new();
        sha2b.input(&first);
        println!("sha2b {}", sha2b.result_str());


        //println!("---");
    }

    println!("{}", now.elapsed().as_secs());

    //Server::http("127.0.0.1:8888").unwrap().handle(hello).unwrap();
}

#[derive(Debug)]
pub struct BlockHeader {
    /// The protocol version. Should always be 1.
    pub version: [u8; 4],
    /// Reference to the previous block in the chain
    pub prev_blockhash: [u8; 32],
    /// The root hash of the merkle tree of transactions in the block
    pub merkle_root: [u8; 32],
    /// The timestamp of the block, as claimed by the mainer
    pub time: [u8; 4],
    /// The target value below which the blockhash must lie, encoded as a
    /// a float (with well-defined rounding, of course)
    pub bits: [u8; 4],
    /// The nonce, selected to obtain a low enough blockhash
    pub nonce: [u8; 4],
}

fn transform_u32_to_reversed_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b4, b3, b2, b1]
}

//01000000 0000000000000000000000000000000000000000000000000000000000000000 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a495fab29 01000000 7c2bac1d
//01000000 0000000000000000000000000000000000000000000000000000000000000000 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49 01000000 1dac2b7c
//01000000 0000000000000000000000000000000000000000000000000000000000000000 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49 ffff001d 1dac2b7c
//00000001 0000000000000000000000000000000000000000000000000000000000000000 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b495fab29 00000001 7c2bac1d
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

fn parse_block(getblockheader: &str  ) -> Result<BlockHeader,Error>{

    let r : Value = serde_json::from_str(getblockheader).unwrap();
    println!("{:?}",r);

    let result : BlockHeader = BlockHeader::new();
    println!("{:?}",result );

    Ok(result)
}


fn extract_next(json_result: &str) -> Result<String,Error> {

    let r = serde_json::from_str(json_result);
    let v : Value = r.unwrap();
    let x = v["result"]["nextblockhash"].as_str().unwrap();

    Ok(String::from(x))
}

fn hello(req: Request, mut res: Response) {
    // handle things here


    println!("req.uri {:?}", req.uri);
    let body = b"Hello World!";
    res.headers_mut().set(ContentLength(body.len() as u64));
    let mut res = res.start().unwrap();
    res.write_all(body).unwrap();

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
