#[inline]
pub fn compress_json(bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(bytes, 0)
}

pub fn decompress_json(compressed: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    zstd::decode_all(compressed)
}

#[cfg(test)]
mod test {

    use serde::{Deserialize, Serialize};

    use crate::compression_utils::{compress_json, decompress_json};

    #[derive(Serialize, Deserialize, Default)]
    struct JsonObject {
        company_name: String,
        employee_count: u32,
        field1: String,
        field2: u32,
        field3: String,
        field4: u32,
        field5: String,
    }

    #[test]
    fn test_compress_json() {
        let json_object = JsonObject {
            company_name: "momento".to_string(),
            employee_count: 50,
            field1: "field1".to_string(),
            field2: 100,
            field3: "field3".to_string(),
            field4: 100,
            field5: "field5".to_string(),
        };
        let json_str = serde_json::to_string(&json_object).expect("Serialization failed");
        let compressed = compress_json(json_str.as_bytes()).expect("compression failed");

        // assert we are in fact compressed
        assert!(compressed.len() < json_str.len());
        assert_eq!(
            vec![
                40, 181, 47, 253, 0, 88, 133, 2, 0, 66, 4, 15, 21, 160, 87, 7, 214, 33, 23, 145, 2,
                168, 248, 92, 10, 64, 100, 88, 219, 183, 152, 254, 55, 33, 135, 17, 10, 206, 41, 6,
                13, 16, 180, 17, 216, 137, 193, 164, 32, 180, 211, 98, 87, 250, 223, 178, 150, 55,
                241, 58, 125, 110, 26, 253, 84, 154, 158, 170, 150, 187, 108, 4, 7, 0, 128, 16,
                192, 15, 16, 6, 136, 16, 138, 40, 24, 236, 72, 148, 61
            ],
            compressed
        );

        let decompressed = decompress_json(compressed.as_slice()).expect("decompression failed");
        assert_eq!(json_str.as_bytes(), decompressed);
        assert_eq!(
            json_str,
            String::from_utf8(decompressed).expect("failed to convert from bytes to utf8")
        )
    }
}
