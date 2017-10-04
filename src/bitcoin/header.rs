use bitcoin;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use util::hex::{ToHex,FromHex};
use std::fmt;

static GENESIS_RAW_HEX: &'static str = "0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c";

#[derive(Copy, Clone, Debug)]
pub struct BlockHeader {
    pub version: [u8; 4], // The protocol version. Should always be 1.
    pub prev_blockhash: [u8; 32], // Reference to the previous block in the chain
    pub merkle_root: [u8; 32], /// The root hash of the merkle tree of transactions in the block
    pub time: [u8; 4], // The timestamp of the block, as claimed by the mainer
    pub bits: [u8; 4], // The target value below which the blockhash must lie, encoded as a a float (with well-defined rounding, of course)
    pub nonce: [u8; 4], // The nonce, selected to obtain a low enough blockhash
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}{}{}{}{}{}",
               self.version.to_hex(),
               self.prev_blockhash.to_hex(),
               self.merkle_root.to_hex(),
               self.time.to_hex(),
               self.bits.to_hex(),
               self.nonce.to_hex()
        )
    }
}


impl BlockHeader {

    pub fn new() -> BlockHeader {
        BlockHeader { version: [0;4], prev_blockhash: [0;32], merkle_root: [0;32], time: [0;4], bits: [0;4], nonce: [0;4]  }
    }

    pub fn genesis() -> BlockHeader {
        let mut genesis_raw_bytes : [u8;80] = [0;80] ;
        let genesis_raw_vec = GENESIS_RAW_HEX.from_hex().unwrap();
        genesis_raw_bytes.clone_from_slice(&genesis_raw_vec);
        let b = BlockHeader::from_bytes(genesis_raw_bytes);
        return b;
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

    pub fn as_compressed_bytes(&self) -> [u8;44] {
        let mut result : [u8;44] = [0;44];
        let current = &self.as_bytes();
        result[0..4].clone_from_slice(&current[0..4]);
        result[4..40].clone_from_slice(&current[36..72]);
        result[40..44].clone_from_slice(&current[76..80]);
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

    pub fn from_compressed_bytes(compressed_bytes : [u8;44], prev_blockhash : [u8;32], difficulty : [u8;4]) -> BlockHeader {
        let mut result : [u8;80] = [0;80];
        result[0..4].clone_from_slice(&compressed_bytes[0..4]);
        result[4..36].clone_from_slice(&prev_blockhash);
        result[36..72].clone_from_slice(&compressed_bytes[4..40]);
        result[72..76].clone_from_slice(&difficulty);
        result[76..80].clone_from_slice(&compressed_bytes[40..44]);

        BlockHeader::from_bytes(result)
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

    pub fn hash_be(&self) -> [u8;32] {
        let mut hash = self.hash();
        hash.reverse();
        hash
    }
    pub fn hash(&self) -> [u8;32] {
        let mut sha2 = Sha256::new();
        sha2.input(&self.as_bytes());
        let mut first : [u8;32] = [0;32];
        sha2.result(&mut first);
        let mut sha2b = Sha256::new();
        sha2b.input(&first);


        let bytes : Vec<u8>= (&sha2b.result_str()).from_hex().unwrap();  //TODO nonsense passing from string
        //bytes.reverse();

        let mut result: [u8;32] = [0;32];
        result.clone_from_slice(&bytes);

        result
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
    result.clone_from_slice(&vec);
    result
}

fn to_reversed_array_of_32(mut vec : Vec<u8> ) -> [u8;32] {
    let mut result : [u8;32] = [0;32];
    vec.reverse();
    result.clone_from_slice(&vec);
    result
}

#[cfg(test)]
mod tests {

    use bitcoin::header::BlockHeader;
    use util::hex::{ToHex,FromHex};
    use bitcoin::header::GENESIS_RAW_HEX;

    #[test]
    pub fn test_genesis() {
        let g = BlockHeader::genesis();
        let h = g.hash_be().to_hex();
        assert_eq!(h,"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");
    }

    #[test]
    pub fn test_as_compressed_bytes() {
        let g = BlockHeader::genesis();
        let b = g.as_compressed_bytes();
        assert_eq!(b.to_hex(),"010000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f491dac2b7c");
    }

    #[test]
    pub fn test_block_header_from_hex_to_hex() {
        let mut genesis_raw_bytes : [u8;80] = [0;80] ;
        let genesis_raw_vec = GENESIS_RAW_HEX.from_hex().unwrap();
        genesis_raw_bytes.clone_from_slice(&genesis_raw_vec);
        let b = BlockHeader::from_bytes(genesis_raw_bytes);
        assert_eq!(GENESIS_RAW_HEX,b.as_bytes().to_hex());
        assert_eq!("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",b.hash_be().to_hex());
    }

    #[test]
    pub fn test_block_headers_reconstruct() {
        let test_data_144 = include_bytes!("../../examples/144/0").to_vec();
        let zeroes = "0000000000000000000000000000000000000000000000000000000000000000";
        let genesis_block = "6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000";
        let block_42335 = "e709fcacfe11464204e4cc1daf4a7b63df72a742a59f4f3eef96843000000000";

        test_block_header_reconstruct(test_data_144,
                                      144,
                                      String::from(zeroes),
                                      String::from(genesis_block), //genesis block
                                      String::from("61188712afd4785d18ef15db57fb52dd150b56c8b547fc6bbf23ec4900000000"), //block #143
        );

        let test_data_2016 = include_bytes!("../../examples/2016/0").to_vec();
        test_block_header_reconstruct(test_data_2016,
                                      2016,
                                      String::from(zeroes),
                                      String::from(genesis_block), //genesis block
                                      String::from("6397bb6abd4fc521c0d3f6071b5650389f0b4551bc40b4e6b067306900000000"), //block #2015
        );

        let test_data_2016_20 = include_bytes!("../../examples/2016/20").to_vec();
        test_block_header_reconstruct(test_data_2016_20,
                                      2016,
                                      String::from("1a231097b6ab6279c80f24674a2c8ee5b9a848e1d45715ad89b6358100000000"), //block #40319
                                      String::from("45720d24eae33ade0d10397a2e02989edef834701b965a9b161e864500000000"), //block #40320
                                      String::from(block_42335), //block #42335
        );

        let test_data_2016_21 = include_bytes!("../../examples/2016/21").to_vec();
        test_block_header_reconstruct(test_data_2016_21,
                                      2016,
                                      String::from(block_42335), //block #42335
                                      String::from("1296ba2f0a66e421d7f51c4596c2ce0820903f3d81a953173778000b00000000"), //block #42336
                                      String::from("d55e1b468c22798971272037d6cc04fdac73913c0012d0d7630c2e1a00000000"), //block #44351
        );

        //TODO add test continuity between chunk (prev first hash equal last previous chunk)
    }

    pub fn test_block_header_reconstruct(test_data : Vec<u8>, chunk_size : u32, first_prev_hash : String, first_hash_verify : String, last_hash_verify : String) {
        let mut first : [u8;80] = [0;80];
        first.clone_from_slice(&test_data[0..80]);
        let first_as_block : BlockHeader = BlockHeader::from_bytes(first);
        let first_hash = first_as_block.hash();
        assert_eq!(first_hash.to_hex(),first_hash_verify);
        assert_eq!(first_as_block.prev_blockhash.to_hex(),first_prev_hash);
        let mut prev_hash = first_hash;
        let prev_diff = first_as_block.bits;
        for i in 0..chunk_size-1 {
            let mut compressed_block_bytes : [u8;44] = [0;44];
            let start = (i * 44 + 80) as usize;
            let end   = (start + 44) as usize;
            compressed_block_bytes.clone_from_slice(&test_data[start..end]);
            let current_as_block : BlockHeader = BlockHeader::from_compressed_bytes(compressed_block_bytes, prev_hash, prev_diff);
            let current_hash = current_as_block.hash();
            prev_hash = current_hash;
            if i==chunk_size-2 {
                assert_eq!(current_hash.to_hex(),last_hash_verify);
            }
        }
    }

}
