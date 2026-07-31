use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct DecryptOutput {
    pub plaintext: Zeroizing<Vec<u8>>,
}

impl DecryptOutput {
    pub fn new(plaintext: Vec<u8>) -> Self {
        Self {
            plaintext: Zeroizing::new(plaintext),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EncryptOutput {
    pub ciphertext: Zeroizing<Vec<u8>>,
}

impl EncryptOutput {
    pub fn new(ciphertext: Vec<u8>) -> Self {
        Self {
            ciphertext: Zeroizing::new(ciphertext),
        }
    }
}
