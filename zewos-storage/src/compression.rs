use std::io::{Error, Write};
use zstd::{decode_all, encode_all};

pub fn compress_bytes(input: &[u8], level: i32) -> Result<Vec<u8>, Error> {
    encode_all(input, level)
}

pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>, Error> {
    decode_all(input)
}

pub fn compress_bytes_with_dict(
    input: &[u8],
    level: i32,
    dictionary: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    let mut compressed = Vec::new();
    let mut encoder = zstd::Encoder::with_dictionary(&mut compressed, level, dictionary)?;
    encoder.write_all(input)?;
    encoder.finish()?;
    Ok(compressed)
}

pub fn decompress_bytes_with_dict(
    input: &[u8],
    dictionary: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = zstd::Decoder::with_dictionary(input, dictionary)?;
    let mut decompressed = Vec::new();
    std::io::copy(&mut decoder, &mut decompressed)?;
    Ok(decompressed)
}
