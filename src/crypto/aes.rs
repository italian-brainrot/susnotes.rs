use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use pbkdf2::pbkdf2;
use sha2::Sha256;
use std::error::Error;

type HmacSha256 = hmac::Hmac<Sha256>;

/// Derive encryption key from password using PBKDF2
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let _ = pbkdf2::<HmacSha256>(password.as_bytes(), salt, 600_000, &mut key);
    key
}

/// Generate a random salt
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).expect("Failed to generate random bytes");
    salt
}

/// Encrypt plaintext with a password
pub fn encrypt(plaintext: &[u8], password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let salt = generate_salt();
    let key = derive_key(password, &salt);

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let mut nonce = [0u8; 12];
    getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher.encrypt(nonce, plaintext)?;

    // Format: salt(16) + nonce(12) + ciphertext
    let mut result = Vec::with_capacity(16 + 12 + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt ciphertext with a password
pub fn decrypt(encrypted: &[u8], password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted.len() < 28 {
        return Err("Encrypted data too short".into());
    }

    let salt = &encrypted[..16];
    let nonce_bytes = &encrypted[16..28];
    let ciphertext = &encrypted[28..];

    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

/// Hash a password for verification
pub fn hash_password(password: &str) -> (String, String) {
    let salt = generate_salt();
    let hash = derive_key(password, &salt);
    (
        hex::encode(&hash),
        hex::encode(&salt),
    )
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str, salt: &str) -> bool {
    let salt_bytes = match hex::decode(salt) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let expected_hash = match hex::decode(hash) {
        Ok(h) => h,
        Err(e) => {
            log::warn!("Failed to decode password hash: {}", e);
            return false;
        }
    };
    let actual_hash = derive_key(password, &salt_bytes);
    expected_hash == actual_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = b"Hello, encrypted world!";
        let password = "test-password-123";

        let encrypted = encrypt(plaintext, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(decrypted, plaintext);
        assert_ne!(encrypted, plaintext);
    }

    #[test]
    fn test_wrong_password() {
        let plaintext = b"Secret message";
        let password = "correct-password";

        let encrypted = encrypt(plaintext, password).unwrap();
        let result = decrypt(&encrypted, "wrong-password");

        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash() {
        let password = "my-strong-password";
        let (hash, salt) = hash_password(password);

        assert!(verify_password(password, &hash, &salt));
        assert!(!verify_password("wrong-password", &hash, &salt));
    }
}
