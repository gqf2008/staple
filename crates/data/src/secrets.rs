//! Local encrypted secret storage (`local_encrypted` provider).
//!
//! Values are encrypted with AES-256-GCM under a per-instance master key
//! (default `data/secrets/master.key`), mirroring the upstream local provider
//! semantics: the database alone is not enough to recover values.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use thiserror::Error;

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// Secret encryption errors.
#[derive(Debug, Error)]
pub enum CipherError {
    /// The master key could not be read or created.
    #[error("master key error: {0}")]
    Key(#[source] io::Error),
    /// Encryption failed.
    #[error("encryption failed")]
    Encrypt,
    /// Decryption failed (bad key, tampered ciphertext, or wrong nonce).
    #[error("decryption failed")]
    Decrypt,
    /// Stored value is not valid base64.
    #[error("invalid encoded value")]
    InvalidEncoding,
}

/// AES-256-GCM cipher bound to a master key file.
#[derive(Debug)]
pub struct SecretCipher {
    key: [u8; KEY_LEN],
}

impl SecretCipher {
    /// Loads the master key from `path`, creating a random one when missing.
    ///
    /// # Errors
    ///
    /// Returns [`CipherError`] when the key cannot be read or written.
    pub fn load_or_create(path: impl AsRef<Path>) -> Result<Self, CipherError> {
        let path = path.as_ref();
        let key = match fs::read(path) {
            Ok(bytes) if bytes.len() == KEY_LEN => {
                let mut key = [0u8; KEY_LEN];
                key.copy_from_slice(&bytes);
                key
            }
            Ok(_) => {
                return Err(CipherError::Key(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "master key has an invalid length",
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let mut key = [0u8; KEY_LEN];
                rand::rngs::OsRng.fill_bytes(&mut key);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(CipherError::Key)?;
                }
                fs::write(path, key).map_err(CipherError::Key)?;
                key
            }
            Err(error) => return Err(CipherError::Key(error)),
        };
        Ok(Self { key })
    }

    /// Encrypts `plaintext`, returning base64(nonce || ciphertext).
    ///
    /// # Errors
    ///
    /// Returns [`CipherError::Encrypt`] on failure.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String, CipherError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CipherError::Encrypt)?;
        let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        Ok(base64_encode(&combined))
    }

    /// Decrypts base64(nonce || ciphertext) produced by [`Self::encrypt`].
    ///
    /// # Errors
    ///
    /// Returns [`CipherError`] on malformed input or authentication failure.
    pub fn decrypt(&self, encoded: &str) -> Result<Vec<u8>, CipherError> {
        let combined = base64_decode(encoded).ok_or(CipherError::InvalidEncoding)?;
        if combined.len() < NONCE_LEN {
            return Err(CipherError::InvalidEncoding);
        }
        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        cipher
            .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
            .map_err(|_| CipherError::Decrypt)
    }
}

/// Redacts secret values from arbitrary text (logs, transcripts, outputs).
#[must_use]
pub fn redact(text: &str, secrets: &[String]) -> String {
    let mut result = text.to_owned();
    for secret in secrets {
        if secret.len() >= 4 {
            result = result.replace(secret, "***");
        }
    }
    result
}

/// Default master key path.
#[must_use]
pub fn default_key_path() -> PathBuf {
    PathBuf::from("data/secrets/master.key")
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(n >> 6) as usize & 63] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[n as usize & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for ch in input.chars().filter(|ch| *ch != '=') {
        let value = match ch {
            'A'..='Z' => ch as u32 - 'A' as u32,
            'a'..='z' => ch as u32 - 'a' as u32 + 26,
            '0'..='9' => ch as u32 - '0' as u32 + 52,
            '+' => 62,
            '/' => 63,
            _ => return None,
        };
        buffer = (buffer << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
        }
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cipher = SecretCipher::load_or_create(dir.path().join("key")).unwrap();
        let encoded = cipher.encrypt(b"sk-ant-secret-value").unwrap();
        assert_ne!(encoded, "c2stYW50LXNlY3JldC12YWx1ZQ==");
        let plain = cipher.decrypt(&encoded).unwrap();
        assert_eq!(plain, b"sk-ant-secret-value");
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let cipher_a = SecretCipher::load_or_create(dir.path().join("key")).unwrap();
        let encoded = cipher_a.encrypt(b"value").unwrap();
        let cipher_b = SecretCipher::load_or_create(dir.path().join("other")).unwrap();
        assert!(cipher_b.decrypt(&encoded).is_err());
    }

    #[test]
    fn redact_hides_values() {
        let text = "run output: sk-ant-secret-value and more";
        let redacted = redact(text, &["sk-ant-secret-value".to_owned()]);
        assert!(!redacted.contains("sk-ant-secret-value"));
        assert!(redacted.contains("***"));
    }
}
