use crate::errors::{KeypairError, SignatureError};

use ecdsa::{
    signature::Keypair as EcdsaKeypair, signature::Verifier, RecoveryId,
    Signature as EcdsaSignature, SigningKey, VerifyingKey as EcdsaVerifyingKey,
};
use sha3::{Digest, Sha3_256};

pub struct VerifyingKey {
    key: EcdsaVerifyingKey<p256::NistP256>,
}

impl VerifyingKey {
    pub fn new(key: EcdsaVerifyingKey<p256::NistP256>) -> Self {
        Self { key }
    }

    pub fn verify(&self, hash: &[u8], signature: &[u8]) -> Result<bool, SignatureError> {
        let signature = EcdsaSignature::from_slice(signature).map_err(|_| {
            SignatureError::InvalidKeyFormat("Invalid signature format".to_string())
        })?;

        Ok(self.key.verify(hash, &signature).is_ok())
    }

    pub fn from_recovery_id(
        recovery_id: RecoveryId,
        signature: &[u8],
        message: &[u8],
    ) -> Result<Self, SignatureError> {
        let verifying_key = EcdsaVerifyingKey::recover_from_prehash(
            message,
            &EcdsaSignature::from_slice(signature).map_err(|_| {
                SignatureError::InvalidKeyFormat("Invalid signature format".to_string())
            })?,
            recovery_id,
        )
        .map_err(|_| SignatureError::InvalidSignature)?;
        Ok(Self::new(verifying_key))
    }
    pub fn key(&self) -> &EcdsaVerifyingKey<p256::NistP256> {
        &self.key
    }
}
pub struct Keypair {
    signing_key: SigningKey<p256::NistP256>,
    verifying_key: VerifyingKey,
}

impl Keypair {
    pub fn new() -> Result<Self, KeypairError> {
        let signing_key = SigningKey::<p256::NistP256>::random(&mut rand::thread_rng());
        let verifying_key = *signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key: VerifyingKey::new(verifying_key),
        })
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SignatureError> {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let (signature, _recovery_id) = self
            .signing_key
            .sign_recoverable(&hash)
            .map_err(|e| SignatureError::InvalidKeyFormat(e.to_string()))?;
        Ok(signature.to_vec())
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SignatureError> {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        self.verifying_key
            .verify(&hash, signature)
            .map_err(|e| SignatureError::InvalidKeyFormat(e.to_string()))
    }
    pub fn signing_key(&self) -> &SigningKey<p256::NistP256> {
        &self.signing_key
    }
}
impl EcdsaKeypair for Keypair {
    type VerifyingKey = EcdsaVerifyingKey<p256::NistP256>;

    fn verifying_key(&self) -> Self::VerifyingKey {
        *self.verifying_key.key()
    }
}
pub struct SignatureBuilder {
    keypair: Keypair,
    data: Option<Vec<u8>>,
}

impl SignatureBuilder {
    pub fn init(keypair: Keypair) -> Self {
        Self {
            keypair,
            data: None,
        }
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }
    pub fn set_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn build(self) -> Result<Signature, SignatureError> {
        let data = self.data.ok_or(SignatureError::MissingData)?;
        let signature = self.keypair.sign(&data)?;
        Ok(Signature {
            data,
            signature: Some(signature),
            verifying_key: self.keypair.verifying_key(),
            recovery_id: None,
        })
    }
}
pub struct Signature {
    data: Vec<u8>,
    signature: Option<Vec<u8>>,
    verifying_key: EcdsaVerifyingKey<p256::NistP256>,
    recovery_id: Option<RecoveryId>,
}

impl Signature {
    pub fn builder(keypair: Keypair) -> SignatureBuilder {
        SignatureBuilder::init(keypair)
    }

    pub fn verify(&self) -> Result<bool, SignatureError> {
        match &self.signature {
            Some(sig) => {
                let mut hasher = Sha3_256::new();
                hasher.update(&self.data);
                let hash = hasher.finalize();
                let signature = EcdsaSignature::from_slice(sig).map_err(|_| {
                    SignatureError::InvalidKeyFormat("Invalid signature format".to_string())
                })?;
                Ok(self.verifying_key.verify(&hash, &signature).is_ok())
            }
            None => Ok(false),
        }
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_signature(&self) -> Option<&Vec<u8>> {
        self.signature.as_ref()
    }

    pub fn get_verifying_key(&self) -> &EcdsaVerifyingKey<p256::NistP256> {
        &self.verifying_key
    }

    pub fn get_recovery_id(&self) -> Option<RecoveryId> {
        self.recovery_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_new() {
        let keypair = Keypair::new();
        assert!(keypair.is_ok());
    }

    #[test]
    fn test_keypair_sign_and_verify() {
        let keypair = Keypair::new().unwrap();
        let data = b"test data";
        let signature = keypair.sign(data);
        assert!(signature.is_ok());

        let verify_result = keypair.verify(data, &signature.unwrap());
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
    }

    #[test]
    fn test_signature_builder() {
        let keypair = Keypair::new().unwrap();
        let data = b"test data".to_vec();
        let signature = Signature::builder(keypair).data(data).build();

        assert!(signature.is_ok());
    }

    #[test]
    fn test_signature_verify() {
        let keypair = Keypair::new().unwrap();
        let data = b"test data".to_vec();
        let signature = Signature::builder(keypair).data(data).build().unwrap();

        let verify_result = signature.verify();
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
    }

    #[test]
    fn test_verifying_key_from_recovery_id() {
        let keypair = Keypair::new().unwrap();
        let data = b"test data";
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        let (signature, recovery_id) = keypair.signing_key().sign_recoverable(&hash).unwrap();

        let verifying_key = VerifyingKey::from_recovery_id(recovery_id, &signature.to_vec(), &hash);
        assert!(verifying_key.is_ok());
    }
}
