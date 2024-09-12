use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Error, Key, Nonce,
};

pub use aes_gcm::{Aes128Gcm, Aes256Gcm};
#[derive(Clone)]
pub struct AES<T: AeadCore + Aead + KeyInit> {
    cipher: T,
}

impl<T: AeadCore + Aead + KeyInit> AES<T> {
    pub fn new<K: AsRef<[u8]>>(key: K) -> Self {
        Self {
            cipher: T::new(Key::<T>::from_slice(key.as_ref())),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: Option<&[u8]>) -> Result<Vec<u8>, Error> {
        let nonce = match nonce {
            Some(n) => Nonce::from_slice(n).to_owned(),
            None => T::generate_nonce(&mut OsRng),
        };
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        if ciphertext.len() < 12 {
            return Err(Error);
        }

        let (nonce, encrypted_data) = ciphertext.split_at(12);
        self.cipher
            .decrypt(Nonce::from_slice(nonce), encrypted_data)
    }
}
