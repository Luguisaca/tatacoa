use crate::bundle::{reject_symlink, validate_manifest_links};
use crate::crypto::{
    ProtectedBundleKey, decrypt_stream_io, derive_kek, derive_object_dek, recover_bundle_key,
};
use crate::encrypted_format::{
    CHUNK_BYTES, EncryptedHeader, HEADER_AAD_BYTES, HEADER_BYTES, MAX_DIRECTORY_CIPHERTEXT_BYTES,
    ObjectDescriptor, ObjectType, decode_directory, directory_aad, object_aad,
};
use crate::{Error, ExportMode, Manifest, Result, SecretPassword};
use sha2::{Digest as ShaDigest, Sha256};
use std::fmt::Write as FmtWrite;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use zeroize::Zeroizing;

/// Authenticates every encrypted object and verifies artifact sizes and digests.
/// Plaintext artifacts are streamed into a digest sink and are never persisted.
pub fn verify_encrypted_bundle(path: &Path, password: &SecretPassword) -> Result<Manifest> {
    read_encrypted_bundle(path, password, None)
}

/// Writes authenticated artifact plaintext only into caller-managed staging.
/// The caller must keep staging invisible and remove it on any failure.
pub(crate) fn materialize_encrypted_bundle(
    path: &Path,
    password: &SecretPassword,
    objects: &Path,
) -> Result<Manifest> {
    read_encrypted_bundle(path, password, Some(objects))
}

fn read_encrypted_bundle(
    path: &Path,
    password: &SecretPassword,
    objects: Option<&Path>,
) -> Result<Manifest> {
    reject_symlink(path, "encrypted bundle")?;
    let mut file = File::open(path).map_err(|source| Error::io("open encrypted bundle", source))?;
    let metadata = file
        .metadata()
        .map_err(|source| Error::io("inspect encrypted bundle", source))?;
    if !metadata.is_file() {
        return Err(Error::InvalidPath(
            "encrypted bundle is not a regular file".to_owned(),
        ));
    }

    let mut header_bytes = [0_u8; HEADER_BYTES];
    file.read_exact(&mut header_bytes)
        .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
    let header = EncryptedHeader::decode(&header_bytes, metadata.len())?;
    let kek = derive_kek(password, &header.salt)?;
    let protected = ProtectedBundleKey {
        nonce: header.wrap_nonce,
        ciphertext: header.wrapped_bundle_key.to_vec(),
    };
    let bundle_key = recover_bundle_key(&kek, &protected, &header_bytes[..HEADER_AAD_BYTES])?;

    let directory_plaintext_length = plaintext_length_from_ciphertext(
        header.directory_ciphertext_length,
        MAX_DIRECTORY_CIPHERTEXT_BYTES,
    )?;
    let directory_capacity = usize::try_from(directory_plaintext_length)
        .map_err(|_| Error::Cryptography("invalid encrypted envelope"))?;
    let mut directory = Zeroizing::new(Vec::with_capacity(directory_capacity));
    let directory_dek = derive_object_dek(&bundle_key, b"directory", b"root")?;
    decrypt_stream_io(
        &directory_dek,
        &header.directory_nonce,
        directory_plaintext_length,
        CHUNK_BYTES as usize,
        &directory_aad(&header_bytes),
        &mut file,
        &mut *directory,
    )?;
    let descriptors = decode_directory(&directory)?;

    let manifest_descriptor = descriptors
        .first()
        .ok_or(Error::Cryptography("invalid encrypted envelope"))?;
    let mut manifest_bytes = Zeroizing::new(Vec::with_capacity(
        usize::try_from(manifest_descriptor.plaintext_length)
            .map_err(|_| Error::Cryptography("invalid encrypted envelope"))?,
    ));
    decrypt_object(
        &bundle_key,
        &header_bytes,
        manifest_descriptor,
        &mut file,
        &mut *manifest_bytes,
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| Error::Cryptography("bundle authentication failed"))?;
    validate_manifest_links(&manifest)?;
    if manifest.export_mode != ExportMode::Encrypted
        || descriptors.len() != manifest.artifacts.len() + 1
    {
        return Err(Error::Cryptography("invalid encrypted envelope"));
    }

    for (descriptor, artifact) in descriptors.iter().skip(1).zip(&manifest.artifacts) {
        if descriptor.object_type != ObjectType::Artifact
            || descriptor.object_id != artifact.id.to_string()
            || descriptor.plaintext_length != artifact.size_bytes
            || artifact.digest.algorithm != "SHA-256"
        {
            return Err(Error::Cryptography("invalid encrypted envelope"));
        }
        let output = objects
            .map(|root| {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join(format!("{}.bin", artifact.id)))
                    .map_err(|source| Error::io("create imported artifact", source))
            })
            .transpose()?;
        let mut digest_writer = DigestWriter {
            output,
            ..DigestWriter::default()
        };
        decrypt_object(
            &bundle_key,
            &header_bytes,
            descriptor,
            &mut file,
            &mut digest_writer,
        )?;
        let (size, digest) = digest_writer.finish()?;
        if size != artifact.size_bytes || !digest.eq_ignore_ascii_case(&artifact.digest.value) {
            return Err(Error::Cryptography("bundle authentication failed"));
        }
    }

    let mut trailing = [0_u8; 1];
    if file
        .read(&mut trailing)
        .map_err(|source| Error::io("finish encrypted bundle", source))?
        != 0
    {
        return Err(Error::Cryptography("invalid encrypted envelope"));
    }
    Ok(manifest)
}

fn decrypt_object(
    bundle_key: &crate::crypto::SecretKey,
    header: &[u8; HEADER_BYTES],
    descriptor: &ObjectDescriptor,
    reader: &mut dyn Read,
    writer: &mut dyn Write,
) -> Result<()> {
    let dek = derive_object_dek(
        bundle_key,
        descriptor.object_type.domain(),
        descriptor.object_id.as_bytes(),
    )?;
    let written = decrypt_stream_io(
        &dek,
        &descriptor.stream_nonce,
        descriptor.plaintext_length,
        CHUNK_BYTES as usize,
        &object_aad(header, descriptor)?,
        reader,
        writer,
    )?;
    if written != descriptor.plaintext_length {
        return Err(Error::Cryptography("bundle authentication failed"));
    }
    Ok(())
}

fn plaintext_length_from_ciphertext(ciphertext_length: u64, maximum: u64) -> Result<u64> {
    if ciphertext_length < 16 || ciphertext_length > maximum {
        return Err(Error::Cryptography("invalid encrypted envelope"));
    }
    // The directory is bounded to 16 MiB, so at most 16 chunks exist. Find the
    // unique plaintext length whose deterministic STREAM framing matches.
    for chunks in 1_u64..=16 {
        let tag_bytes = chunks
            .checked_mul(16)
            .ok_or(Error::Cryptography("invalid encrypted envelope"))?;
        let Some(plaintext) = ciphertext_length.checked_sub(tag_bytes) else {
            continue;
        };
        let expected_chunks = if plaintext == 0 {
            1
        } else {
            plaintext.div_ceil(u64::from(CHUNK_BYTES))
        };
        if expected_chunks == chunks {
            return Ok(plaintext);
        }
    }
    Err(Error::Cryptography("invalid encrypted envelope"))
}

#[derive(Default)]
struct DigestWriter {
    digest: Sha256,
    size: u64,
    output: Option<File>,
}

impl DigestWriter {
    fn finish(mut self) -> Result<(u64, String)> {
        if let Some(file) = self.output.take() {
            file.sync_all()
                .map_err(|source| Error::io("sync imported artifact", source))?;
        }
        let bytes = self.digest.finalize();
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            let _ = write!(encoded, "{byte:02x}");
        }
        Ok((self.size, encoded))
    }
}

impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Some(output) = &mut self.output {
            output.write_all(bytes)?;
        }
        self.digest.update(bytes);
        self.size = self
            .size
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("digest size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
