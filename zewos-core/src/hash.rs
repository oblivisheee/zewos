pub use hex::{FromHex, ToHex};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Sha256(pub [u8; 32]);

impl Sha256 {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let result = hasher.finalize();

        Self(result.into())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl ToHex for Sha256 {
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T {
        self.0.encode_hex()
    }
    fn encode_hex_upper<T: std::iter::FromIterator<char>>(&self) -> T {
        self.0.encode_hex_upper()
    }
}

impl FromHex for Sha256 {
    type Error = hex::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let bytes = Vec::from_hex(hex)?;
        Ok(Self(
            bytes
                .try_into()
                .map_err(|_| hex::FromHexError::InvalidStringLength)?,
        ))
    }
}

pub struct Blake3(blake3::Hash);

impl Blake3 {
    pub fn new(data: &[u8]) -> Self {
        Self(blake3::hash(data))
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(blake3::Hash::from_bytes(bytes))
    }
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }
    pub fn from_hex(hex: &str) -> Self {
        Self(blake3::Hash::from_hex(hex).unwrap())
    }
}
impl ToHex for Blake3 {
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T {
        self.0.as_bytes().encode_hex()
    }
    fn encode_hex_upper<T: std::iter::FromIterator<char>>(&self) -> T {
        self.0.as_bytes().encode_hex_upper()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_new() {
        let data = b"hello world";
        let sha256 = Sha256::new(data);
        assert_eq!(sha256.0.len(), 32);
    }

    #[test]
    fn test_sha256_to_hex() {
        let data = b"test data";
        let sha256 = Sha256::new(data);
        let hex: String = sha256.encode_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_from_hex_valid() {
        let hex = "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447";
        let result = Sha256::from_hex(hex);
        assert!(result.is_ok());
        let sha256 = result.unwrap();
        assert_eq!(sha256.0.len(), 32);
    }

    #[test]
    fn test_sha256_roundtrip() {
        let original_data = b"roundtrip test";
        let sha256 = Sha256::new(original_data);
        let hex: String = sha256.encode_hex();
        let roundtrip_sha256 = Sha256::from_hex(&hex).unwrap();
        assert_eq!(sha256.0, roundtrip_sha256.0);
    }

    #[test]
    fn test_sha256_serde() {
        let data = b"serde test";
        let sha256 = Sha256::new(data);

        // Serialize
        let serialized = serde_json::to_string(&sha256).unwrap();

        // Deserialize
        let deserialized: Sha256 = serde_json::from_str(&serialized).unwrap();

        assert_eq!(sha256, deserialized);
    }
}
