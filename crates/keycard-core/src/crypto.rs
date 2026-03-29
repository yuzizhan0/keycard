//! Argon2id → HKDF-SHA256 → XChaCha20-Poly1305 per design spec §2.
//!
//! Parameters (document in README): Argon2id **m = 65536 KiB (64 MiB)**, **t = 3**, **p = 4**.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use sha2::Sha256;

/// HKDF info label for the data encryption key (32 bytes).
pub const HKDF_INFO_DEK: &[u8] = b"keycard-v1-dek";

const ARGON2_M_KIB: u32 = 65536;
const ARGON2_T: u32 = 3;
const ARGON2_P: u32 = 4;
const MASTER_KEY_LEN: usize = 32;
const XNONCE_LEN: usize = 24;

#[derive(Debug)]
pub enum CryptoError {
    Argon2(argon2::password_hash::Error),
    Hkdf,
    Aead(chacha20poly1305::aead::Error),
    InvalidKeyLength,
    InvalidNonceLength,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Argon2(e) => write!(f, "argon2: {e}"),
            CryptoError::Hkdf => write!(f, "hkdf expand failed"),
            CryptoError::Aead(e) => write!(f, "aead: {e}"),
            CryptoError::InvalidKeyLength => write!(f, "invalid key length"),
            CryptoError::InvalidNonceLength => write!(f, "invalid nonce length"),
        }
    }
}

impl std::error::Error for CryptoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CryptoError::Argon2(e) => Some(e),
            CryptoError::Aead(e) => Some(e),
            _ => None,
        }
    }
}

/// Derive a 32-byte master key from password and salt (salt stored in DB in production).
pub fn derive_master_key(password: &[u8], salt: &[u8]) -> Result<[u8; MASTER_KEY_LEN], CryptoError> {
    let params = Params::new(ARGON2_M_KIB, ARGON2_T, ARGON2_P, None).map_err(CryptoError::Argon2)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; MASTER_KEY_LEN];
    argon2
        .hash_password_into(password, salt, &mut out)
        .map_err(CryptoError::Argon2)?;
    Ok(out)
}

/// Derive the 32-byte DEK used for entry secrets.
pub fn derive_dek(master_key: &[u8; MASTER_KEY_LEN]) -> Result<[u8; MASTER_KEY_LEN], CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, master_key.as_slice());
    let mut dek = [0u8; MASTER_KEY_LEN];
    hk.expand(HKDF_INFO_DEK, &mut dek)
        .map_err(|_| CryptoError::Hkdf)?;
    Ok(dek)
}

fn cipher_from_dek(dek: &[u8; MASTER_KEY_LEN]) -> Result<XChaCha20Poly1305, CryptoError> {
    XChaCha20Poly1305::new_from_slice(dek.as_slice()).map_err(|_| CryptoError::InvalidKeyLength)
}

/// Encrypt `plaintext`; returns `(nonce, ciphertext)` (nonce is 24 bytes).
pub fn seal_secret(dek: &[u8; MASTER_KEY_LEN], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    let cipher = cipher_from_dek(dek)?;
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(CryptoError::Aead)?;
    Ok((nonce.to_vec(), ciphertext))
}

/// Decrypt using DEK and nonce + ciphertext from [`seal_secret`].
pub fn open_secret(
    dek: &[u8; MASTER_KEY_LEN],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if nonce.len() != XNONCE_LEN {
        return Err(CryptoError::InvalidNonceLength);
    }
    let cipher = cipher_from_dek(dek)?;
    let nonce = XNonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(CryptoError::Aead)
}

#[cfg(test)]
mod crypto_tests {
    use super::{derive_dek, derive_master_key, open_secret, seal_secret};

    #[test]
    fn seal_and_open_roundtrip() {
        let master = derive_master_key(b"password", b"salt123456789012").unwrap();
        let dek = derive_dek(&master).unwrap();
        let secret = b"sk-test-abc";
        let (nonce, ct) = seal_secret(&dek, secret).unwrap();
        let out = open_secret(&dek, &nonce, &ct).unwrap();
        assert_eq!(out, secret);
    }
}
