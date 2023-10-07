use bitpacking::{BitPacker, BitPacker8x};

pub fn split(input: &[u8]) -> (Vec<u32>, Vec<u32>) {
    let mut headers = vec![];
    let mut indices = vec![];

    let mut header = true;
    for chunk in input.chunks(3) {
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

pub fn compress(data: &[u32]) -> (Vec<u8>, Vec<u8>) {
    let bitpacker = BitPacker8x::new();
    let mut initial_value = 0u32;
    let mut result = vec![];
    let mut compressed_lens = vec![];
    let mut lengths: Vec<u8> = vec![];

    for chunk in data.chunks_exact(BitPacker8x::BLOCK_LEN) {
        let mut compressed = vec![0u8; 4 * BitPacker8x::BLOCK_LEN];

        let num_bits: u8 = bitpacker.num_bits(chunk);

        println!("num bits: {num_bits}");

        let compressed_len =
            bitpacker.compress_sorted(initial_value, chunk, &mut compressed[..], num_bits);

        println!("compressed len: {compressed_len}");

        result.extend_from_slice(&compressed[..compressed_len]);

        compressed_lens.push(compressed_len as u32);
        initial_value = chunk[BitPacker8x::BLOCK_LEN - 1];
    }
    // TODO: handle remainder

    let mut initial_value = 0u32;

    for chunk in compressed_lens.chunks_exact(BitPacker8x::BLOCK_LEN) {
        let mut compressed = vec![0u8; 4 * BitPacker8x::BLOCK_LEN];

        let num_bits: u8 = bitpacker.num_bits(chunk);

        println!("num bits: {num_bits}");

        let compressed_len =
            bitpacker.compress_sorted(initial_value, chunk, &mut compressed[..], num_bits);

        println!("compressed len: {compressed_len}");

        lengths.extend_from_slice(&compressed[..compressed_len]);

        initial_value = chunk[BitPacker8x::BLOCK_LEN - 1];
    }
    // TODO: handle remainder

    (result, lengths)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn bitpacker() -> Result<()> {
        let file = fs::read("sealList.bin")?;
        println!("orig bytes: {}", file.len() * 4);

        let (headers, indices) = split(&file);

        let (compressed_headers, hlens) = compress(&headers);
        let (compressed_indices, ilens) = compress(&indices);

        println!("compressed headers len: {}", compressed_headers.len());
        println!("compressed indices len: {}", compressed_indices.len());

        println!("compressed headers len: {}", hlens.len());
        println!("compressed indices len: {}", ilens.len());

        assert_eq!(
            compressed_headers.len() + compressed_indices.len() + hlens.len() + ilens.len(),
            file.len()
        );

        // assert_eq!(
        //     (num_bits as usize) * BitPacker4x::BLOCK_LEN / 8,
        //     compressed_len
        // );

        // // Decompressing
        // let mut decompressed = vec![0u32; BitPacker4x::BLOCK_LEN];

        // // The initial value must be the same as the one passed
        // // when compressing the block.
        // bitpacker.decompress_sorted(
        //     initial_value,
        //     &compressed[..compressed_len],
        //     &mut decompressed[..],
        //     num_bits,
        // );

        // assert_eq!(&data, &decompressed);

        // assert_eq!(compressed.len(), 0);

        Ok(())
    }
}
