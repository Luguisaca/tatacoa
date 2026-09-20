use crate::{Error, Result, SecurityProfile};
use aead_stream::aead::{Aead, KeyInit, Payload};
use aead_stream::{DecryptorBE32, EncryptorBE32};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use std::io::{Read, Write};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub(crate) const ARGON2_MEMORY_KIB: u32 = 65_536;
pub(crate) const ARGON2_ITERATIONS: u32 = 3;
pub(crate) const ARGON2_LANES: u32 = 4;
pub(crate) const ARGON2_SALT_BYTES: usize = 16;
pub(crate) const KEY_BYTES: usize = 32;
pub(crate) const WRAP_NONCE_BYTES: usize = 12;
pub(crate) const STREAM_NONCE_BYTES: usize = 7;
pub const MAX_PASSWORD_BYTES: usize = 1024;
pub const MIN_EXPORT_PASSWORD_CHARACTERS: usize = 12;
pub const PROFESSIONAL_MIN_PASSWORD_CHARACTERS: usize = 14;
pub const HIGH_SENSITIVITY_MIN_PASSWORD_CHARACTERS: usize = 16;

const DEK_DOMAIN: &[u8] = b"TATACOA\0encrypted\0v1\0dek\0";

#[derive(Zeroize, ZeroizeOnDrop)]
pub(crate) struct SecretKey([u8; KEY_BYTES]);

pub struct SecretPassword(Zeroizing<Vec<u8>>);

impl SecretPassword {
    pub fn for_export(mut password: String) -> Result<Self> {
        if password.chars().count() < MIN_EXPORT_PASSWORD_CHARACTERS
            || password.len() > MAX_PASSWORD_BYTES
        {
            password.zeroize();
            return Err(Error::Conflict(format!(
                "ENCRYPTED export password must contain at least {MIN_EXPORT_PASSWORD_CHARACTERS} characters and at most {MAX_PASSWORD_BYTES} UTF-8 bytes"
            )));
        }
        Ok(Self(Zeroizing::new(password.into_bytes())))
    }

    pub fn for_export_profile(profile: SecurityProfile, mut password: String) -> Result<Self> {
        let character_count = password.chars().count();
        let minimum = match profile {
            SecurityProfile::LabLearning => MIN_EXPORT_PASSWORD_CHARACTERS,
            SecurityProfile::Professional => PROFESSIONAL_MIN_PASSWORD_CHARACTERS,
            SecurityProfile::HighSensitivity => HIGH_SENSITIVITY_MIN_PASSWORD_CHARACTERS,
            SecurityProfile::Custom => {
                password.zeroize();
                return Err(Error::Conflict(
                    "CUSTOM has no approved password policy; ENCRYPTED export is denied".to_owned(),
                ));
            }
        };
        if character_count < minimum || password.len() > MAX_PASSWORD_BYTES {
            password.zeroize();
            return Err(Error::Conflict(format!(
                "{profile:?} ENCRYPTED export password must contain at least {minimum} characters and at most {MAX_PASSWORD_BYTES} UTF-8 bytes"
            )));
        }
        if profile != SecurityProfile::LabLearning && is_obviously_weak_password(&password) {
            password.zeroize();
            return Err(Error::Conflict(format!(
                "{profile:?} ENCRYPTED export rejects predictable passwords; use a longer, non-repetitive passphrase"
            )));
        }
        Ok(Self(Zeroizing::new(password.into_bytes())))
    }

    pub fn for_verification(mut password: String) -> Result<Self> {
        if password.is_empty() || password.len() > MAX_PASSWORD_BYTES {
            password.zeroize();
            return Err(Error::Cryptography("bundle authentication failed"));
        }
        Ok(Self(Zeroizing::new(password.into_bytes())))
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

fn is_obviously_weak_password(password: &str) -> bool {
    let normalized = password.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return true;
    }
    let chars: Vec<char> = normalized.chars().collect();
    if chars.iter().all(|character| *character == chars[0]) {
        return true;
    }
    if is_repeated_pattern(&chars) || is_monotonic_ascii_sequence(&chars) {
        return true;
    }
    const COMMON: &[&str] = &[
        "password",
        "password123",
        "password1234",
        "qwerty",
        "qwerty123",
        "qwerty123456",
        "123456789012",
        "123456789123",
        "letmein",
        "administrator",
        "changeme",
    ];
    COMMON.iter().any(|candidate| normalized == *candidate)
}

fn is_repeated_pattern(chars: &[char]) -> bool {
    (1..=chars.len() / 2).any(|period| {
        chars
            .iter()
            .enumerate()
            .all(|(index, character)| *character == chars[index % period])
    })
}

fn is_monotonic_ascii_sequence(chars: &[char]) -> bool {
    if chars.len() < 4
        || !chars
            .iter()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return false;
    }
    let bytes: Vec<u8> = chars.iter().map(|character| *character as u8).collect();
    let ascending = bytes
        .windows(2)
        .all(|pair| pair[1] == pair[0].wrapping_add(1));
    let descending = bytes
        .windows(2)
        .all(|pair| pair[0] == pair[1].wrapping_add(1));
    ascending || descending
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

pub(crate) fn random_stream_nonce() -> Result<[u8; STREAM_NONCE_BYTES]> {
    let mut nonce = [0_u8; STREAM_NONCE_BYTES];
    getrandom::fill(&mut nonce)
        .map_err(|_| Error::Cryptography("system random generation unavailable"))?;
    Ok(nonce)
}

pub(crate) fn random_wrap_nonce() -> Result<[u8; WRAP_NONCE_BYTES]> {
    let mut nonce = [0_u8; WRAP_NONCE_BYTES];
    getrandom::fill(&mut nonce)
        .map_err(|_| Error::Cryptography("system random generation unavailable"))?;
    Ok(nonce)
}

pub(crate) fn protect_bundle_key(
    kek: &SecretKey,
    bundle_key: &SecretKey,
    authenticated_header: &[u8],
) -> Result<ProtectedBundleKey> {
    let nonce = random_wrap_nonce()?;
    protect_bundle_key_with_nonce(kek, bundle_key, nonce, authenticated_header)
}

pub(crate) fn protect_bundle_key_with_nonce(
    kek: &SecretKey,
    bundle_key: &SecretKey,
    nonce: [u8; WRAP_NONCE_BYTES],
    authenticated_header: &[u8],
) -> Result<ProtectedBundleKey> {
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

pub(crate) fn encrypt_stream_io(
    dek: &SecretKey,
    nonce: &[u8; STREAM_NONCE_BYTES],
    plaintext_length: u64,
    chunk_bytes: usize,
    associated_data: &[u8],
    reader: &mut dyn Read,
    writer: &mut dyn Write,
) -> Result<u64> {
    if chunk_bytes == 0 {
        return Err(Error::Cryptography("stream encryption failed"));
    }
    let mut key = Key::<Aes256Gcm>::from(*dek.as_bytes());
    let stream_nonce =
        aead_stream::Nonce::<Aes256Gcm, aead_stream::StreamBE32<Aes256Gcm>>::from(*nonce);
    let mut encryptor = EncryptorBE32::<Aes256Gcm>::new(&key, &stream_nonce);
    key.zeroize();
    let mut remaining = plaintext_length;
    let mut written = 0_u64;

    if remaining == 0 {
        let ciphertext = encryptor
            .encrypt_last(Payload {
                msg: &[],
                aad: associated_data,
            })
            .map_err(|_| Error::Cryptography("stream encryption failed"))?;
        writer
            .write_all(&ciphertext)
            .map_err(|source| Error::io("write encrypted stream", source))?;
        return u64::try_from(ciphertext.len())
            .map_err(|_| Error::Cryptography("stream encryption failed"));
    }

    while remaining > chunk_bytes as u64 {
        let read_length = chunk_bytes;
        let mut plaintext = Zeroizing::new(vec![0_u8; read_length]);
        reader
            .read_exact(&mut plaintext)
            .map_err(|source| Error::io("read plaintext stream", source))?;
        remaining -= read_length as u64;
        let ciphertext = encryptor
            .encrypt_next(Payload {
                msg: plaintext.as_slice(),
                aad: associated_data,
            })
            .map_err(|_| Error::Cryptography("stream encryption failed"))?;
        writer
            .write_all(&ciphertext)
            .map_err(|source| Error::io("write encrypted stream", source))?;
        written = written
            .checked_add(
                u64::try_from(ciphertext.len())
                    .map_err(|_| Error::Cryptography("stream encryption failed"))?,
            )
            .ok_or(Error::Cryptography("stream encryption failed"))?;
    }
    let final_length =
        usize::try_from(remaining).map_err(|_| Error::Cryptography("stream encryption failed"))?;
    let mut final_plaintext = Zeroizing::new(vec![0_u8; final_length]);
    reader
        .read_exact(&mut final_plaintext)
        .map_err(|source| Error::io("read plaintext stream", source))?;
    let final_ciphertext = encryptor
        .encrypt_last(Payload {
            msg: final_plaintext.as_slice(),
            aad: associated_data,
        })
        .map_err(|_| Error::Cryptography("stream encryption failed"))?;
    writer
        .write_all(&final_ciphertext)
        .map_err(|source| Error::io("write encrypted stream", source))?;
    written = written
        .checked_add(
            u64::try_from(final_ciphertext.len())
                .map_err(|_| Error::Cryptography("stream encryption failed"))?,
        )
        .ok_or(Error::Cryptography("stream encryption failed"))?;
    Ok(written)
}

pub(crate) fn decrypt_stream_io(
    dek: &SecretKey,
    nonce: &[u8; STREAM_NONCE_BYTES],
    plaintext_length: u64,
    chunk_bytes: usize,
    associated_data: &[u8],
    reader: &mut dyn Read,
    writer: &mut dyn Write,
) -> Result<u64> {
    if chunk_bytes == 0 {
        return Err(Error::Cryptography("bundle authentication failed"));
    }
    let mut key = Key::<Aes256Gcm>::from(*dek.as_bytes());
    let stream_nonce =
        aead_stream::Nonce::<Aes256Gcm, aead_stream::StreamBE32<Aes256Gcm>>::from(*nonce);
    let mut decryptor = DecryptorBE32::<Aes256Gcm>::new(&key, &stream_nonce);
    key.zeroize();
    let mut remaining = plaintext_length;
    let mut written = 0_u64;

    if remaining == 0 {
        let mut ciphertext = vec![0_u8; 16];
        reader
            .read_exact(&mut ciphertext)
            .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
        let plaintext = decryptor
            .decrypt_last(Payload {
                msg: &ciphertext,
                aad: associated_data,
            })
            .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
        if !plaintext.is_empty() {
            return Err(Error::Cryptography("bundle authentication failed"));
        }
        return Ok(0);
    }

    while remaining > chunk_bytes as u64 {
        let plaintext_chunk = chunk_bytes;
        let ciphertext_length = plaintext_chunk
            .checked_add(16)
            .ok_or(Error::Cryptography("bundle authentication failed"))?;
        let mut ciphertext = vec![0_u8; ciphertext_length];
        reader
            .read_exact(&mut ciphertext)
            .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
        remaining -= plaintext_chunk as u64;
        let mut plaintext = decryptor
            .decrypt_next(Payload {
                msg: &ciphertext,
                aad: associated_data,
            })
            .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
        if plaintext.len() != plaintext_chunk {
            plaintext.zeroize();
            return Err(Error::Cryptography("bundle authentication failed"));
        }
        writer
            .write_all(&plaintext)
            .map_err(|source| Error::io("write authenticated plaintext staging", source))?;
        written = written
            .checked_add(plaintext_chunk as u64)
            .ok_or(Error::Cryptography("bundle authentication failed"))?;
        plaintext.zeroize();
    }
    let final_plaintext_length = usize::try_from(remaining)
        .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
    let final_ciphertext_length = final_plaintext_length
        .checked_add(16)
        .ok_or(Error::Cryptography("bundle authentication failed"))?;
    let mut final_ciphertext = vec![0_u8; final_ciphertext_length];
    reader
        .read_exact(&mut final_ciphertext)
        .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
    let mut final_plaintext = decryptor
        .decrypt_last(Payload {
            msg: &final_ciphertext,
            aad: associated_data,
        })
        .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
    if final_plaintext.len() != final_plaintext_length {
        final_plaintext.zeroize();
        return Err(Error::Cryptography("bundle authentication failed"));
    }
    writer
        .write_all(&final_plaintext)
        .map_err(|source| Error::io("write authenticated plaintext staging", source))?;
    written = written
        .checked_add(remaining)
        .ok_or(Error::Cryptography("bundle authentication failed"))?;
    final_plaintext.zeroize();
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    const HEADER: &[u8] = b"authenticated public header prototype";

    #[test]
    fn fixed_argon2id_protects_and_recovers_bundle_key() -> Result<()> {
        let salt = random_salt()?;
        let password = SecretPassword::for_export("correct horse battery staple".to_owned())?;
        let kek = derive_kek(&password, &salt)?;
        let bundle_key = generate_bundle_key()?;
        let protected = protect_bundle_key(&kek, &bundle_key, HEADER)?;
        let recovered = recover_bundle_key(&kek, &protected, HEADER)?;
        assert_eq!(bundle_key.as_bytes(), recovered.as_bytes());
        assert_ne!(protected.ciphertext.as_slice(), bundle_key.as_bytes());
        Ok(())
    }

    #[test]
    fn profile_password_policy_scales_with_security_profile() {
        assert!(
            SecretPassword::for_export_profile(
                SecurityProfile::LabLearning,
                "000000000000".to_owned()
            )
            .is_ok()
        );

        for weak in ["00000000000000", "11111111111111", "12345678912345"] {
            assert!(
                SecretPassword::for_export_profile(SecurityProfile::Professional, weak.to_owned())
                    .is_err()
            );
        }
        assert!(
            SecretPassword::for_export_profile(
                SecurityProfile::Professional,
                "correct horse battery staple".to_owned()
            )
            .is_ok()
        );

        for weak in ["0000000000000000", "1111111111111111", "1234123412341234"] {
            assert!(
                SecretPassword::for_export_profile(
                    SecurityProfile::HighSensitivity,
                    weak.to_owned()
                )
                .is_err()
            );
        }
        assert!(
            SecretPassword::for_export_profile(
                SecurityProfile::HighSensitivity,
                "evidence glacier orbit lantern".to_owned()
            )
            .is_ok()
        );
    }

    #[test]
    fn wrong_password_and_header_tampering_fail_closed() -> Result<()> {
        let salt = random_salt()?;
        let password = SecretPassword::for_export("correct password".to_owned())?;
        let wrong_password = SecretPassword::for_verification("wrong password".to_owned())?;
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
    fn caller_managed_wrap_nonce_is_authenticated() -> Result<()> {
        let salt = random_salt()?;
        let password = SecretPassword::for_export("authenticated nonce password".to_owned())?;
        let kek = derive_kek(&password, &salt)?;
        let bundle_key = generate_bundle_key()?;
        let nonce = random_wrap_nonce()?;
        let protected = protect_bundle_key_with_nonce(&kek, &bundle_key, nonce, HEADER)?;
        assert_eq!(protected.nonce, nonce);
        let mut changed_nonce = protected.nonce;
        changed_nonce[0] ^= 1;
        let altered = ProtectedBundleKey {
            nonce: changed_nonce,
            ciphertext: protected.ciphertext,
        };
        assert!(recover_bundle_key(&kek, &altered, HEADER).is_err());
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
    fn password_policy_applies_only_when_creating_encrypted_bundles() {
        assert!(SecretPassword::for_export("short".to_owned()).is_err());
        assert!(SecretPassword::for_export("docecarácter".to_owned()).is_ok());
        assert!(SecretPassword::for_verification("short".to_owned()).is_ok());
        assert!(SecretPassword::for_verification(String::new()).is_err());
        assert!(SecretPassword::for_verification("x".repeat(MAX_PASSWORD_BYTES + 1)).is_err());
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
    fn streaming_io_is_bounded_and_fails_on_truncation() -> Result<()> {
        let bundle_key = generate_bundle_key()?;
        let dek = derive_object_dek(&bundle_key, b"artifact", b"art_io")?;
        let nonce = random_stream_nonce()?;
        let plaintext = vec![0x42; 3 * 1024 + 17];
        let mut encrypted = Vec::new();
        let encrypted_length = encrypt_stream_io(
            &dek,
            &nonce,
            plaintext.len() as u64,
            1024,
            HEADER,
            &mut plaintext.as_slice(),
            &mut encrypted,
        )?;
        assert_eq!(encrypted_length, encrypted.len() as u64);
        let mut recovered = Vec::new();
        assert_eq!(
            decrypt_stream_io(
                &dek,
                &nonce,
                plaintext.len() as u64,
                1024,
                HEADER,
                &mut encrypted.as_slice(),
                &mut recovered,
            )?,
            plaintext.len() as u64
        );
        assert_eq!(recovered, plaintext);
        encrypted.pop();
        assert!(
            decrypt_stream_io(
                &dek,
                &nonce,
                plaintext.len() as u64,
                1024,
                HEADER,
                &mut encrypted.as_slice(),
                &mut Vec::new(),
            )
            .is_err()
        );
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
