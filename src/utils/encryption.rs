use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::Engine;
use std::sync::Arc;

#[derive(Clone)]
pub struct Encryption {
    cipher: Arc<Aes256Gcm>,
}

impl Encryption {
    pub fn new(key: &str) -> Result<Self> {
        // Преобразуем hex строку в 32 байта
        let key_bytes = hex::decode(key)?;
        if key_bytes.len() != 32 {
            anyhow::bail!("Encryption key must be 32 bytes (64 hex characters)");
        }

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid encryption key: {}", e))?;
        Ok(Self {
            cipher: Arc::new(cipher),
        })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Объединяем nonce и ciphertext в одну строку (base64)
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(base64::engine::general_purpose::STANDARD.encode(combined))
    }

    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        let combined = base64::engine::general_purpose::STANDARD
            .decode(ciphertext)
            .context("Base64 decode failed")?;

        if combined.len() < 12 {
            anyhow::bail!("Invalid ciphertext length");
        }

        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext_bytes = &combined[12..];

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext_bytes)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        Ok(String::from_utf8(plaintext)?)
    }
}
