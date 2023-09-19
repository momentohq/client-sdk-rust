#[inline]
pub fn compress_json_str(input: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(input.as_bytes(), 0)
}

pub fn decompress_json_bytes(compressed: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    zstd::decode_all(compressed)
}
