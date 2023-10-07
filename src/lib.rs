use q_compress::{Compressor, CompressorConfig};

pub fn u48tou64(bytes: &[u8]) -> Vec<u64> {
    let mut data = vec![];

    for chunk in bytes.chunks_exact(6) {
        data.push(u64::from_le_bytes([
            chunk[5], chunk[4], chunk[3], chunk[2], chunk[1], chunk[0], 0x00, 0x00,
        ]));
    }

    data
}

pub fn u64tou48(data: &[u64]) -> Vec<u8> {
    let mut bytes = vec![];

    for chunk in data.iter() {
        let chunk_bytes = chunk.to_le_bytes();
        bytes.push(chunk_bytes[0]);
        bytes.push(chunk_bytes[1]);
        bytes.push(chunk_bytes[2]);
        bytes.push(chunk_bytes[3]);
        bytes.push(chunk_bytes[4]);
        bytes.push(chunk_bytes[5]);
    }

    bytes
}

pub fn compress(data: &[u32]) -> Vec<u8> {
    let mut compressor = Compressor::<u32>::from_config(
        CompressorConfig::default()
            .with_compression_level(8)
            .with_delta_encoding_order(1)
            .with_use_gcds(false),
    );

    compressor.simple_compress(data)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use q_compress::{auto_compress, auto_decompress};

    use super::*;

    #[test]
    fn q_compressor() -> Result<()> {
        let file = fs::read("sealList.bin")?;
        println!("orig bytes: {}", file.len());

        let data = u48tou64(&file);

        let compressed = auto_compress(&data, 8);

        println!("compressed headers: {}", compressed.len());
        // println!("compressed indices: {}", compressed_indices.len());
        println!("total: {}", compressed.len());

        let decompressed = auto_decompress::<u64>(&compressed)?;
        // let decompressed_indices = auto_decompress::<u32>(&compressed_indices)?;

        let bytes = u64tou48(&decompressed);

        assert_eq!(file[0..10], bytes[0..10]);
        assert_eq!(file.len(), bytes.len());

        Ok(())
    }
}
