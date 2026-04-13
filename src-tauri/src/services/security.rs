use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use base64::{engine::general_purpose::STANDARD, Engine as _};

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let hash = argon2.hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    
    println!("[Security] New password hashed successfully");
    hash
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    if hash.is_empty() {
        println!("[Security] Verification failed: Hash is empty");
        return false;
    }

    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(e) => {
            println!("[Security] Verification failed: Invalid hash format ({:?})", e);
            return false;
        }
    };

    let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok();
    
    if result {
        println!("[Security] Verification SUCCESS");
    } else {
        println!("[Security] Verification FAILED: Password mismatch");
    }
    
    result
}

pub fn encrypt(data: &[u8], secret: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let key_bytes = hasher.finalize();

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data).expect("Encryption failed");
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    STANDARD.encode(combined)
}

pub fn decrypt(encrypted_base64: &str, secret: &str) -> Option<Vec<u8>> {
    let combined = STANDARD.decode(encrypted_base64).ok()?;
    if combined.len() < 12 { return None; }

    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let key_bytes = hasher.finalize();

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ciphertext).ok()
}
