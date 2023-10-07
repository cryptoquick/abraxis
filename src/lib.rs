use q_compress::{Compressor, CompressorConfig};

pub fn split(input: &[u8]) -> (Vec<u32>, Vec<u32>) {
    let mut headers = vec![];
    let mut indices = vec![];

    let mut header = true;
    for chunk in input.chunks_exact(3) {
        if header {
            headers.push(u32::from_le_bytes([chunk[2], chunk[1], chunk[0], 0x00]));
            header = false;
        } else {
            indices.push(u32::from_le_bytes([chunk[2], chunk[1], chunk[0], 0x00]));
            header = true;
        }
    }

    (headers, indices)
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
    use q_compress::auto_decompress;

    use super::*;

    #[test]
    fn q_compressor() -> Result<()> {
        let file = fs::read("sealList.bin")?;
        println!("orig bytes: {}", file.len());

        let (headers, indices) = split(&file);

        let compressed_headers = compress(&headers);
        let compressed_indices = compress(&indices);

        println!("compressed headers: {}", compressed_headers.len());
        println!("compressed indices: {}", compressed_indices.len());
        println!(
            "total: {}",
            compressed_headers.len() + compressed_indices.len()
        );

        let decompressed_headers = auto_decompress::<u32>(&compressed_headers)?;
        let decompressed_indices = auto_decompress::<u32>(&compressed_indices)?;

        let bytes = join(&decompressed_headers, &decompressed_indices);

        assert_eq!(file[0..10], bytes[0..10]);
        assert_eq!(file.len(), bytes.len());

        Ok(())
    }
}
