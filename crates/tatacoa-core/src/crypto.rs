use crate::{Error, Result};
use aead_stream::aead::{Aead, KeyInit, Payload};
use aead_stream::{DecryptorBE32, EncryptorBE32};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub(crate) const ARGON2_MEMORY_KIB: u32 = 65_536;
pub(crate) const ARGON2_ITERATIONS: u32 = 3;
pub(crate) const ARGON2_LANES: u32 = 4;
pub(crate) const ARGON2_SALT_BYTES: usize = 16;
pub(crate) const KEY_BYTES: usize = 32;
pub(crate) const WRAP_NONCE_BYTES: usize = 12;
pub(crate) const STREAM_NONCE_BYTES: usize = 7;

const DEK_DOMAIN: &[u8] = b"TATACOA\0encrypted\0v1\0dek\0";

#[derive(Zeroize, ZeroizeOnDrop)]
pub(crate) struct SecretKey([u8; KEY_BYTES]);

pub(crate) struct SecretPassword(Zeroizing<Vec<u8>>);

impl SecretPassword {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl SecretKey {
    fn random() -> Result<Self> {
        let mut bytes = [0_u8; KEY_BYTES];
        getrandom::fill(&mut bytes)
            .map_err(|_| Error::Cryptography("system random generation unavailable"))?;
        Ok(Self(bytes))
    }

    fn as_bytes(&self) -> &[u8; KEY_BYTES] {
        &self.0
    }
}

pub(crate) struct ProtectedBundleKey {
    pub(crate) nonce: [u8; WRAP_NONCE_BYTES],
    pub(crate) ciphertext: Vec<u8>,
}

pub(crate) struct EncryptedStream {
    pub(crate) nonce: [u8; STREAM_NONCE_BYTES],
    pub(crate) chunks: Vec<Vec<u8>>,
}

pub(crate) fn random_salt() -> Result<[u8; ARGON2_SALT_BYTES]> {
    let mut salt = [0_u8; ARGON2_SALT_BYTES];
    getrandom::fill(&mut salt)
        .map_err(|_| Error::Cryptography("system random generation unavailable"))?;
    Ok(salt)
}

pub(crate) fn derive_kek(
    password: &SecretPassword,
    salt: &[u8; ARGON2_SALT_BYTES],
) -> Result<SecretKey> {
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_LANES,
        Some(KEY_BYTES),
    )
    .map_err(|_| Error::Cryptography("invalid fixed Argon2id parameters"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = SecretKey([0_u8; KEY_BYTES]);
    if argon2
        .hash_password_into(password.as_bytes(), salt, &mut key.0)
        .is_err()
    {
        key.zeroize();
        return Err(Error::Cryptography("password key derivation failed"));
    }
    Ok(key)
}

pub(crate) fn generate_bundle_key() -> Result<SecretKey> {
    SecretKey::random()
}

pub(crate) fn protect_bundle_key(
    kek: &SecretKey,
    bundle_key: &SecretKey,
    authenticated_header: &[u8],
) -> Result<ProtectedBundleKey> {
    let mut nonce = [0_u8; WRAP_NONCE_BYTES];
    getrandom::fill(&mut nonce)
        .map_err(|_| Error::Cryptography("system random generation unavailable"))?;
    let mut key = Key::<Aes256Gcm>::from(*kek.as_bytes());
    let nonce_array = Nonce::from(nonce);
    let cipher = Aes256Gcm::new(&key);
    key.zeroize();
    let ciphertext = cipher
        .encrypt(
            &nonce_array,
            Payload {
                msg: bundle_key.as_bytes(),
                aad: authenticated_header,
            },
        )
        .map_err(|_| Error::Cryptography("bundle key protection failed"))?;
    Ok(ProtectedBundleKey { nonce, ciphertext })
}

pub(crate) fn recover_bundle_key(
    kek: &SecretKey,
    protected: &ProtectedBundleKey,
    authenticated_header: &[u8],
) -> Result<SecretKey> {
    let mut key = Key::<Aes256Gcm>::from(*kek.as_bytes());
    let nonce = Nonce::from(protected.nonce);
    let cipher = Aes256Gcm::new(&key);
    key.zeroize();
    let mut plaintext = cipher
        .decrypt(
            &nonce,
            Payload {
                msg: &protected.ciphertext,
                aad: authenticated_header,
            },
        )
        .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
    if plaintext.len() != KEY_BYTES {
        plaintext.zeroize();
        return Err(Error::Cryptography("bundle authentication failed"));
    }
    let mut bytes = [0_u8; KEY_BYTES];
    bytes.copy_from_slice(&plaintext);
    plaintext.zeroize();
    Ok(SecretKey(bytes))
}

pub(crate) fn derive_object_dek(
    bundle_key: &SecretKey,
    object_type: &[u8],
    object_id: &[u8],
) -> Result<SecretKey> {
    if object_type.is_empty() || object_id.is_empty() {
        return Err(Error::Cryptography("object key domain is incomplete"));
    }
    let type_len = u32::try_from(object_type.len())
        .map_err(|_| Error::Cryptography("object key domain is too large"))?;
    let id_len = u32::try_from(object_id.len())
        .map_err(|_| Error::Cryptography("object key domain is too large"))?;
    let mut info = Vec::with_capacity(DEK_DOMAIN.len() + 8 + object_type.len() + object_id.len());
    info.extend_from_slice(DEK_DOMAIN);
    info.extend_from_slice(&type_len.to_be_bytes());
    info.extend_from_slice(object_type);
    info.extend_from_slice(&id_len.to_be_bytes());
    info.extend_from_slice(object_id);

    let hkdf = Hkdf::<Sha256>::new(None, bundle_key.as_bytes());
    let mut dek = SecretKey([0_u8; KEY_BYTES]);
    if hkdf.expand(&info, &mut dek.0).is_err() {
        info.zeroize();
        dek.zeroize();
        return Err(Error::Cryptography("object key derivation failed"));
    }
    info.zeroize();
    Ok(dek)
}

pub(crate) fn encrypt_stream(
    dek: &SecretKey,
    plaintext_chunks: &[&[u8]],
    associated_data: &[u8],
) -> Result<EncryptedStream> {
    if plaintext_chunks.is_empty() {
        return Err(Error::Cryptography(
            "encrypted stream requires a final chunk",
        ));
    }
    let mut nonce = [0_u8; STREAM_NONCE_BYTES];
    getrandom::fill(&mut nonce)
        .map_err(|_| Error::Cryptography("system random generation unavailable"))?;
    let mut key = Key::<Aes256Gcm>::from(*dek.as_bytes());
    let stream_nonce =
        aead_stream::Nonce::<Aes256Gcm, aead_stream::StreamBE32<Aes256Gcm>>::from(nonce);
    let mut encryptor = EncryptorBE32::<Aes256Gcm>::new(&key, &stream_nonce);
    key.zeroize();
    let mut chunks = Vec::with_capacity(plaintext_chunks.len());
    let (final_chunk, preceding_chunks) = plaintext_chunks.split_last().ok_or(
        Error::Cryptography("encrypted stream requires a final chunk"),
    )?;
    for plaintext in preceding_chunks {
        chunks.push(
            encryptor
                .encrypt_next(Payload {
                    msg: plaintext,
                    aad: associated_data,
                })
                .map_err(|_| Error::Cryptography("stream encryption failed"))?,
        );
    }
    chunks.push(
        encryptor
            .encrypt_last(Payload {
                msg: final_chunk,
                aad: associated_data,
            })
            .map_err(|_| Error::Cryptography("stream encryption failed"))?,
    );
    Ok(EncryptedStream { nonce, chunks })
}

pub(crate) fn decrypt_stream(
    dek: &SecretKey,
    encrypted: &EncryptedStream,
    associated_data: &[u8],
) -> Result<Vec<Vec<u8>>> {
    if encrypted.chunks.is_empty() {
        return Err(Error::Cryptography("bundle authentication failed"));
    }
    let mut key = Key::<Aes256Gcm>::from(*dek.as_bytes());
    let stream_nonce =
        aead_stream::Nonce::<Aes256Gcm, aead_stream::StreamBE32<Aes256Gcm>>::from(encrypted.nonce);
    let mut decryptor = DecryptorBE32::<Aes256Gcm>::new(&key, &stream_nonce);
    key.zeroize();
    let mut plaintext = Vec::with_capacity(encrypted.chunks.len());
    let (final_chunk, preceding_chunks) = encrypted
        .chunks
        .split_last()
        .ok_or(Error::Cryptography("bundle authentication failed"))?;
    for ciphertext in preceding_chunks {
        plaintext.push(
            decryptor
                .decrypt_next(Payload {
                    msg: ciphertext,
                    aad: associated_data,
                })
                .map_err(|_| Error::Cryptography("bundle authentication failed"))?,
        );
    }
    plaintext.push(
        decryptor
            .decrypt_last(Payload {
                msg: final_chunk,
                aad: associated_data,
            })
            .map_err(|_| Error::Cryptography("bundle authentication failed"))?,
    );
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    const HEADER: &[u8] = b"authenticated public header prototype";

    #[test]
    fn fixed_argon2id_protects_and_recovers_bundle_key() -> Result<()> {
        let salt = random_salt()?;
        let password = SecretPassword::new(b"correct horse battery staple".to_vec());
        let kek = derive_kek(&password, &salt)?;
        let bundle_key = generate_bundle_key()?;
        let protected = protect_bundle_key(&kek, &bundle_key, HEADER)?;
        let recovered = recover_bundle_key(&kek, &protected, HEADER)?;
        assert_eq!(bundle_key.as_bytes(), recovered.as_bytes());
        assert_ne!(protected.ciphertext.as_slice(), bundle_key.as_bytes());
        Ok(())
    }

    #[test]
    fn wrong_password_and_header_tampering_fail_closed() -> Result<()> {
        let salt = random_salt()?;
        let password = SecretPassword::new(b"correct password".to_vec());
        let wrong_password = SecretPassword::new(b"wrong password".to_vec());
        let kek = derive_kek(&password, &salt)?;
        let wrong_kek = derive_kek(&wrong_password, &salt)?;
        let bundle_key = generate_bundle_key()?;
        let mut protected = protect_bundle_key(&kek, &bundle_key, HEADER)?;
        let wrong_password_error = recover_bundle_key(&wrong_kek, &protected, HEADER)
            .err()
            .ok_or(Error::Cryptography(
                "wrong password was unexpectedly accepted",
            ))?;
        assert!(!wrong_password_error.to_string().contains("wrong password"));
        assert!(recover_bundle_key(&kek, &protected, b"tampered header").is_err());
        protected.ciphertext[0] ^= 1;
        assert!(recover_bundle_key(&kek, &protected, HEADER).is_err());
        Ok(())
    }

    #[test]
    fn random_material_is_fresh_per_generation() -> Result<()> {
        assert_ne!(random_salt()?, random_salt()?);
        let first_bundle_key = generate_bundle_key()?;
        let second_bundle_key = generate_bundle_key()?;
        assert_ne!(first_bundle_key.as_bytes(), second_bundle_key.as_bytes());
        Ok(())
    }

    #[test]
    fn hkdf_separates_object_type_and_id() -> Result<()> {
        let bundle_key = generate_bundle_key()?;
        let first = derive_object_dek(&bundle_key, b"artifact", b"art_one")?;
        let second = derive_object_dek(&bundle_key, b"artifact", b"art_two")?;
        let manifest = derive_object_dek(&bundle_key, b"manifest", b"manifest")?;
        assert_ne!(first.as_bytes(), second.as_bytes());
        assert_ne!(first.as_bytes(), manifest.as_bytes());
        Ok(())
    }

    #[test]
    fn stream_round_trip_and_negative_cases() -> Result<()> {
        let bundle_key = generate_bundle_key()?;
        let dek = derive_object_dek(&bundle_key, b"artifact", b"art_stream")?;
        let chunks: Vec<&[u8]> = vec![b"first", b"second", b"final"];
        let encrypted = encrypt_stream(&dek, &chunks, HEADER)?;
        assert_eq!(decrypt_stream(&dek, &encrypted, HEADER)?, chunks);

        let mut tampered = EncryptedStream {
            nonce: encrypted.nonce,
            chunks: encrypted.chunks.clone(),
        };
        tampered.chunks[0][0] ^= 1;
        assert!(decrypt_stream(&dek, &tampered, HEADER).is_err());
        assert!(decrypt_stream(&dek, &encrypted, b"tampered aad").is_err());

        let truncated = EncryptedStream {
            nonce: encrypted.nonce,
            chunks: encrypted.chunks[..2].to_vec(),
        };
        assert!(decrypt_stream(&dek, &truncated, HEADER).is_err());

        let reordered = EncryptedStream {
            nonce: encrypted.nonce,
            chunks: vec![
                encrypted.chunks[1].clone(),
                encrypted.chunks[0].clone(),
                encrypted.chunks[2].clone(),
            ],
        };
        assert!(decrypt_stream(&dek, &reordered, HEADER).is_err());
        Ok(())
    }

    #[test]
    #[ignore = "manual development benchmark for selecting the v1 chunk size"]
    fn benchmark_stream_chunk_sizes() -> Result<()> {
        const PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
        let plaintext = vec![0x5a; PAYLOAD_BYTES];
        let bundle_key = generate_bundle_key()?;
        let dek = derive_object_dek(&bundle_key, b"benchmark", b"chunk-size")?;

        for chunk_size in [64 * 1024, 256 * 1024, 1024 * 1024, 4 * 1024 * 1024] {
            let chunks: Vec<&[u8]> = plaintext.chunks(chunk_size).collect();
            let encrypt_started = Instant::now();
            let encrypted = encrypt_stream(&dek, &chunks, HEADER)?;
            let encrypt_elapsed = encrypt_started.elapsed();
            let decrypt_started = Instant::now();
            let decrypted = decrypt_stream(&dek, &encrypted, HEADER)?;
            let decrypt_elapsed = decrypt_started.elapsed();
            assert_eq!(decrypted.concat(), plaintext);
            println!(
                "chunk_bytes={chunk_size} chunks={} encrypt_ms={} decrypt_ms={}",
                chunks.len(),
                encrypt_elapsed.as_millis(),
                decrypt_elapsed.as_millis()
            );
        }
        Ok(())
    }
}
