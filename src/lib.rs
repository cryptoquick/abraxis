use std::collections::BTreeMap;

use anyhow::Result;
use pco::{standalone::simple_compress, ChunkConfig};

pub fn decode(input: &[u8]) -> BTreeMap<u32, Vec<u32>> {
    let mut headers: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    let mut header_num = 0;
    let mut is_header = true;

    for chunk in input.chunks_exact(3) {
        if is_header {
            header_num = u32::from_le_bytes([chunk[2], chunk[1], chunk[0], 0x00]);
            is_header = false;
        } else {
            let seal_num = u32::from_le_bytes([chunk[2], chunk[1], chunk[0], 0x00]);
            let header = headers.entry(header_num).or_default();
            header.push(seal_num);
            is_header = true;
        }
    }

    headers
}

pub fn compress(chain: &BTreeMap<u32, Vec<u32>>) -> Result<Vec<u8>> {
    let mut bytes = vec![];
    let mut lens: Vec<u32> = vec![];

    let mut headers = chain.keys().map(|k| k.to_owned()).collect::<Vec<u32>>();
    headers.sort();

    let compressed_headers = simple_compress(
        headers.as_slice(),
        &ChunkConfig::default()
            .with_compression_level(12)
            .with_delta_encoding_order(Some(1)),
    )?;

    bytes.extend(compressed_headers.len().to_le_bytes());
    bytes.extend(compressed_headers);

    let mut indices = vec![];

    for vals in chain.values() {
        let mut vals = vals.to_owned();
        vals.sort();
        lens.push(vals.len() as u32);
        indices.extend(vals);
    }

    let compressed_indices = simple_compress(
        &indices,
        &ChunkConfig::default()
            .with_compression_level(12)
            .with_delta_encoding_order(Some(1)),
    )?;

    bytes.extend(compressed_indices.len().to_le_bytes());
    bytes.extend(compressed_indices);

    let compressed_lens = simple_compress(
        &lens,
        &ChunkConfig::default()
            .with_compression_level(12)
            .with_delta_encoding_order(Some(0)),
    )?;

    bytes.extend(compressed_lens.len().to_le_bytes());
    bytes.extend(compressed_lens);

    Ok(bytes)
}

pub fn join(headers: &[u32], indices: &[u32]) -> Vec<u8> {
    let mut bytes = vec![];

    for (i, header) in headers.iter().enumerate() {
        let header_bytes = header.to_le_bytes();
        bytes.push(header_bytes[2]);
        bytes.push(header_bytes[1]);
        bytes.push(header_bytes[0]);
        let index_bytes = indices[i].to_le_bytes();
        bytes.push(index_bytes[2]);
        bytes.push(index_bytes[1]);
        bytes.push(index_bytes[0]);
    }

    bytes
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn q_compressor() -> Result<()> {
        let file = fs::read("sealList.bin")?;
        println!("orig bytes: {}", file.len());

        let chain = decode(&file);
        let compressed = compress(&chain).unwrap();

        println!("compressed: {}", compressed.len());

        assert_eq!(compressed.len(), 0);

        // let decompressed_headers = auto_decompress::<u32>(&compressed_headers)?;
        // let decompressed_indices = auto_decompress::<u32>(&compressed_indices)?;

        // let bytes = join(&decompressed_headers, &decompressed_indices);

        // assert_eq!(file[0..10], bytes[0..10]);
        // assert_eq!(file.len(), bytes.len());

        Ok(())
    }
}
