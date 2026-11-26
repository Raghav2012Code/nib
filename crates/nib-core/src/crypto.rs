//! Per-paste sealing: Argon2id for the key, XChaCha20-Poly1305 for the text.

use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};

use crate::error::{Error, Result};

const PREFIX: &str = "nibenc1:";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;

pub fn seal(id: &str, passphrase: &str, plaintext: &[u8]) -> Result<String> {
    let salt: [u8; SALT_LEN] = rand::random();
    let nonce_bytes: [u8; NONCE_LEN] = rand::random();
    let key = derive(id, passphrase, &salt)?;

    let cipher = XChaCha20Poly1305::new(&Key::from(key));
    let sealed = cipher
        .encrypt(
            &XNonce::from(nonce_bytes),
            Payload { msg: plaintext, aad: id.as_bytes() },
        )
        .map_err(|_| Error::Message("sealing failed".into()))?;

    let mut envelope = Vec::with_capacity(SALT_LEN + NONCE_LEN + sealed.len());
    envelope.extend_from_slice(&salt);
    envelope.extend_from_slice(&nonce_bytes);
    envelope.extend_from_slice(&sealed);
    Ok(format!("{PREFIX}{}", STANDARD.encode(envelope)))
}

pub fn open(id: &str, passphrase: &str, envelope: &str) -> Result<String> {
    let body = envelope
        .strip_prefix(PREFIX)
        .ok_or_else(|| Error::Unlock(id.to_string()))?;
    let raw = STANDARD
        .decode(body)
        .map_err(|_| Error::Unlock(id.to_string()))?;
    if raw.len() < SALT_LEN + NONCE_LEN + 16 {
        return Err(Error::Unlock(id.to_string()));
    }

    let (salt, rest) = raw.split_at(SALT_LEN);
    let (nonce, ciphertext) = rest.split_at(NONCE_LEN);
    let key = derive(id, passphrase, salt)?;
    let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| Error::Unlock(id.to_string()))?;

    let cipher = XChaCha20Poly1305::new(&Key::from(key));
    let plain = cipher
        .decrypt(
            &XNonce::from(nonce),
            Payload { msg: ciphertext, aad: id.as_bytes() },
        )
        .map_err(|_| Error::Unlock(id.to_string()))?;

    String::from_utf8(plain).map_err(|_| Error::Unlock(id.to_string()))
}

// 64 MiB, 3 passes, one lane: slow enough to hurt, quick enough for a paste.
fn derive(id: &str, passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    let params = Params::new(64 * 1024, 3, 1, None).map_err(|e| Error::Message(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| Error::Unlock(id.to_string()))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let sealed = seal("k3x9", "hunter2", b"token=abc").unwrap();
        assert!(sealed.starts_with("nibenc1:"));
        assert!(!sealed.contains("token"));
        assert_eq!(open("k3x9", "hunter2", &sealed).unwrap(), "token=abc");
    }

    #[test]
    fn wrong_passphrase_fails() {
        let sealed = seal("k3x9", "hunter2", b"x").unwrap();
        assert!(open("k3x9", "hunter3", &sealed).is_err());
    }

    #[test]
    fn id_is_bound() {
        let sealed = seal("k3x9", "hunter2", b"x").unwrap();
        assert!(open("k3xa", "hunter2", &sealed).is_err());
    }
}
