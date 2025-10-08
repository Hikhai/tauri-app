use anyhow::Result;

// Dummy crypto context using base64 (NOT secure). Replace with real AEAD + KDF later.
pub struct CryptoCtx;

impl CryptoCtx {
    pub fn new_dummy() -> Self { CryptoCtx }
    pub fn encrypt(&self, plain: &[u8]) -> Result<Vec<u8>> {
        Ok(base64::encode(plain).into_bytes())
    }
    pub fn decrypt(&self, enc: &[u8]) -> Result<Vec<u8>> {
        let s = String::from_utf8_lossy(enc);
        let decoded = base64::decode(&*s)?;
        Ok(decoded)
    }
}
