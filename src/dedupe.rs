use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use crate::rolling::RollingHash;
use blake3;
use nohash_hasher::BuildNoHashHasher;

pub struct BlockIndex {
    crc_map: HashMap<u64, Vec<usize>, BuildNoHashHasher<u64>>,
    block_hashes: Vec<blake3::Hash>,
}

pub fn hash_blocks(
    reader: &mut BufReader<File>,
    block_size_log2: u8,
) -> Result<BlockIndex, std::io::Error> {
    let block_size = 1 << block_size_log2;

    let num_bytes = reader.stream_len()? as usize;
    let num_blocks = num_bytes >> block_size_log2;

    let mut crc_map = HashMap::with_capacity_and_hasher(num_blocks, BuildNoHashHasher::default());
    let mut block_hashes = Vec::with_capacity(num_blocks);

    let mut buffer = vec![0; block_size];
    let mut rolling_hasher = RollingHash::new(block_size);

    reader.seek(SeekFrom::Start(0))?;

    for index in 0..num_blocks {
        reader.read_exact(buffer.as_mut_slice())?;

        rolling_hasher.reset();
        let rolling_hash = rolling_hasher.feed(buffer.as_slice());
        let secure_hash = blake3::hash(buffer.as_slice());

        block_hashes.push(secure_hash);
        crc_map
            .entry(rolling_hash)
            .or_insert_with(|| vec![])
            .push(index);
    }

    Ok(BlockIndex {
        block_hashes,
        crc_map,
    })
}

pub fn dedupe(
    reader: &mut BufReader<File>,
    block_size_log2: u8,
    block_index: BlockIndex,
) -> Result<Vec<usize>, std::io::Error> {
    let block_size = 1 << block_size_log2;

    let mut rolling_hasher = RollingHash::new(block_size);
    let mut remap = vec![0; block_index.block_hashes.len()];

    let num_bytes = reader.stream_len()? as usize;
    let mut window_end = 0;
    let mut b = [0; 1];

    reader.seek(SeekFrom::Start(0))?;

    while window_end < num_bytes {
        if remap[window_end >> block_size_log2] > 0 {
            // incoming data overlaps a block that's already been deduped
            window_end = reader.seek(SeekFrom::Current(block_size as i64))? as usize;
            rolling_hasher.reset();
            continue;
        }

        reader.read_exact(&mut b)?;
        window_end += 1;

        let rolling_hash = rolling_hasher.update(b[0]);
        if !rolling_hasher.valid() || rolling_hash == 0 {
            // hash of zero probably means the data is all zeroes
            continue;
        }

        let window_start = window_end - block_size;

        if let Some(indices) = block_index.crc_map.get(&rolling_hash) {
            let mut secure_hash = None;

            for &index in indices {
                let block_pos = index * block_size;
                if remap[index] > 0 || window_end > block_pos {
                    continue;
                }

                if secure_hash.is_none() {
                    let buf_slices = rolling_hasher.buf().as_slices();
                    let mut secure_hasher = blake3::Hasher::new();
                    secure_hasher.update(buf_slices.0);
                    secure_hasher.update(buf_slices.1);
                    secure_hash = Some(secure_hasher.finalize());
                }

                if block_index.block_hashes[index] == secure_hash.unwrap() {
                    remap[index] = block_pos - window_start;
                }
            }
        }
    }

    Ok(remap)
}
