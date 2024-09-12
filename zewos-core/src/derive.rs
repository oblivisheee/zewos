use hkdf::Hkdf;

pub struct Deriver {
    salt: Option<Vec<u8>>,
    ikm: Vec<u8>,
}

impl Deriver {
    pub fn new(salt: Option<Vec<u8>>, ikm: Vec<u8>) -> Self {
        Self { salt, ikm }
    }

    pub fn derive_key(&self, info: &[u8]) -> Vec<u8> {
        let hk = Hkdf::<sha3::Sha3_256>::new(self.salt.as_deref(), &self.ikm);
        let mut okm = vec![0u8; 32];
        hk.expand(info, &mut okm).expect("HKDF expansion failed");
        okm
    }
}
