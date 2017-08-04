
use std;
use serde_json::Error;
use bitcoin;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Copy, Clone, Debug)]
pub struct BlockHeader {
    pub version: [u8; 4], // The protocol version. Should always be 1.
    pub prev_blockhash: [u8; 32], // Reference to the previous block in the chain
    pub merkle_root: [u8; 32], /// The root hash of the merkle tree of transactions in the block
    pub time: [u8; 4], // The timestamp of the block, as claimed by the mainer
    pub bits: [u8; 4], // The target value below which the blockhash must lie, encoded as a a float (with well-defined rounding, of course)
    pub nonce: [u8; 4], // The nonce, selected to obtain a low enough blockhash
}


impl BlockHeader {

    pub fn new() -> BlockHeader {
        BlockHeader { version: [0;4], prev_blockhash: [0;32], merkle_root: [0;32], time: [0;4], bits: [0;4], nonce: [0;4]  }
    }

    pub fn as_bytes(&self) -> [u8;80] {
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

    pub fn as_compressed_bytes(&self) -> [u8;48] {
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

    pub fn from_bytes(bytes : [u8;80]) -> BlockHeader {
        BlockHeader {
            version:        clone_into_array(&bytes[0 .. 4]),
            prev_blockhash: clone_into_array(&bytes[4 .. 36]),
            merkle_root:    clone_into_array(&bytes[36 .. 68]),
            time:           clone_into_array(&bytes[68 .. 72]),
            bits:           clone_into_array(&bytes[72 .. 76]),
            nonce:          clone_into_array(&bytes[76 .. 80])
        }
    }

    pub fn from_compressed_bytes(bytes : [u8;48], prev_blockhash : [u8;32]) -> BlockHeader {
        BlockHeader {
            version:        clone_into_array(&bytes[0 .. 4]),
            prev_blockhash: clone_into_array(&prev_blockhash),
            merkle_root:    clone_into_array(&bytes[4 .. 36]),
            time:           clone_into_array(&bytes[36 .. 40]),
            bits:           clone_into_array(&bytes[40 .. 44]),
            nonce:          clone_into_array(&bytes[44 .. 48])
        }
    }

    pub fn from_block_header_rpc(block_header_rpc : bitcoin::rpc::BlockHeaderRpc) -> BlockHeader {
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

    pub fn hash(&self) -> String {
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


fn transform_u32_to_reversed_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b4, b3, b2, b1]
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

    for byte in hex_str.bytes() {
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
                //let ch = hex_str[idx..].chars().next().unwrap();
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
