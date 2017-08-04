use bitcoin;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use util::hex::{ToHex, FromHex};

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
            version:  to_reversed_array_of_4(version_hex.from_hex().unwrap() ),
            prev_blockhash:  to_reversed_array_of_32(previous_block_hash.from_hex().unwrap() ),
            merkle_root: to_reversed_array_of_32(merkle_root.from_hex().unwrap()),
            time: transform_u32_to_reversed_array_of_u8(time),
            bits: to_reversed_array_of_4(bits.from_hex().unwrap()),
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

        let mut bytes : Vec<u8>= (&sha2b.result_str()).from_hex().unwrap();
        bytes.reverse();

        bytes.to_hex()
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

#[cfg(test)]
mod tests {

    use bitcoin::header::BlockHeader;
    use util::hex::{ToHex,FromHex};

    #[test]
    pub fn test_block_header_from_hex_to_hex() {
        let genesis_raw = "0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c";
        let mut genesis_raw_bytes : [u8;80] = [0;80] ;
        let genesis_raw_vec = genesis_raw.from_hex().unwrap();
        genesis_raw_bytes.clone_from_slice(&genesis_raw_vec);

        let b = BlockHeader::from_bytes(genesis_raw_bytes);

        assert_eq!(genesis_raw,
        b.as_bytes().to_hex());
    }

}
