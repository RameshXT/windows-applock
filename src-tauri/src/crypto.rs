use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};
use rand::{Rng, thread_rng};
use std::fmt;

pub enum CryptoError {
    KeyDerivationFailed(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    IntegrityCheckFailed,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyDerivationFailed(e) => write!(f, "Key derivation failed: {}", e),
            CryptoError::EncryptionFailed(e) => write!(f, "Encryption failed: {}", e),
            CryptoError::DecryptionFailed(e) => write!(f, "Decryption failed: {}", e),
            CryptoError::IntegrityCheckFailed => write!(f, "File integrity check failed (Tampered)"),
        }
    }
}
pub fn derive_encryption_key() -> Result<[u8; 32], CryptoError> {
    let id = machine_uid::get().map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    Ok(key)
}
pub fn calculate_checksum(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut checksum = [0u8; 32];
    checksum.copy_from_slice(&result);
    checksum
}
pub fn encrypt_with_integrity(plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let key_bytes = derive_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let checksum = calculate_checksum(plaintext);
    let mut payload = Vec::with_capacity(32 + plaintext.len());
    payload.extend_from_slice(&checksum);
    payload.extend_from_slice(plaintext);
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, payload.as_ref())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}
pub fn decrypt_with_integrity(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 12 + 32 {
        return Err(CryptoError::DecryptionFailed("Data too short".to_string()));
    }

    let key_bytes = derive_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let decrypted_payload = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    if decrypted_payload.len() < 32 {
        return Err(CryptoError::DecryptionFailed("Invalid decrypted payload".to_string()));
    }
    let (stored_checksum, plaintext) = decrypted_payload.split_at(32);
    let calculated_checksum = calculate_checksum(plaintext);

    if stored_checksum != calculated_checksum {
        return Err(CryptoError::IntegrityCheckFailed);
    }

    Ok(plaintext.to_vec())
}
