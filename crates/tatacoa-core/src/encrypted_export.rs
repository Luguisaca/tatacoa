use crate::bundle::{artifact_source_path, prepare_export_manifest, reject_symlink};
use crate::crypto::{
    derive_kek, derive_object_dek, encrypt_stream_io, generate_bundle_key,
    protect_bundle_key_with_nonce, random_salt, random_stream_nonce, random_wrap_nonce,
};
use crate::encrypted_format::{
    CHUNK_BYTES, EncryptedHeader, HEADER_AAD_BYTES, HEADER_BYTES, MAX_ENVELOPE_BYTES,
    ObjectDescriptor, ObjectType, directory_aad, encode_directory, object_aad,
    stream_ciphertext_length,
};
use crate::{ExportMode, Manifest, Result, SecretPassword, authorize_encrypted_export};
use sha2::{Digest as ShaDigest, Sha256};
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::Path;

pub fn export_encrypted_bundle(
    workspace: &Path,
    manifest: &Manifest,
    destination: &Path,
    password: &SecretPassword,
) -> Result<()> {
    authorize_encrypted_export(manifest.engagement.security_profile)?;
    if destination.exists() {
        return Err(crate::Error::Conflict(format!(
            "bundle destination already exists: {}",
            destination.display()
        )));
    }
    let mut exported = prepare_export_manifest(workspace, manifest, ExportMode::Encrypted)?;
    exported.export_mode = ExportMode::Encrypted;
    let mut manifest_bytes = serde_json::to_vec_pretty(&exported)?;
    manifest_bytes.push(b'\n');

    let mut descriptors = Vec::with_capacity(exported.artifacts.len() + 1);
    descriptors.push(ObjectDescriptor {
        object_type: ObjectType::Manifest,
        object_id: "manifest".to_owned(),
        plaintext_length: manifest_bytes.len() as u64,
        ciphertext_length: stream_ciphertext_length(manifest_bytes.len() as u64)?,
        stream_nonce: random_stream_nonce()?,
    });
    for artifact in &exported.artifacts {
        descriptors.push(ObjectDescriptor {
            object_type: ObjectType::Artifact,
            object_id: artifact.id.to_string(),
            plaintext_length: artifact.size_bytes,
            ciphertext_length: stream_ciphertext_length(artifact.size_bytes)?,
            stream_nonce: random_stream_nonce()?,
        });
    }
    let directory = encode_directory(&descriptors)?;
    let directory_ciphertext_length = stream_ciphertext_length(directory.len() as u64)?;
    let total = (HEADER_BYTES as u64)
        .checked_add(directory_ciphertext_length)
        .and_then(|value| {
            descriptors
                .iter()
                .try_fold(value, |sum, item| sum.checked_add(item.ciphertext_length))
        })
        .ok_or(crate::Error::Cryptography("invalid encrypted envelope"))?;
    if total > MAX_ENVELOPE_BYTES {
        return Err(crate::Error::Cryptography("invalid encrypted envelope"));
    }

    let salt = random_salt()?;
    let wrap_nonce = random_wrap_nonce()?;
    let directory_nonce = random_stream_nonce()?;
    let bundle_key = generate_bundle_key()?;
    let kek = derive_kek(password, &salt)?;
    let mut header = EncryptedHeader {
        salt,
        wrap_nonce,
        directory_nonce,
        directory_ciphertext_length,
        wrapped_bundle_key: [0; 48],
    };
    let provisional = header.encode()?;
    let protected = protect_bundle_key_with_nonce(
        &kek,
        &bundle_key,
        wrap_nonce,
        &provisional[..HEADER_AAD_BYTES],
    )?;
    header.wrapped_bundle_key = protected
        .ciphertext
        .try_into()
        .map_err(|_| crate::Error::Cryptography("bundle key protection failed"))?;
    let header_bytes = header.encode()?;

    let parent = destination.parent().ok_or_else(|| {
        crate::Error::InvalidPath("bundle destination has no parent directory".to_owned())
    })?;
    fs::create_dir_all(parent)
        .map_err(|source| crate::Error::io("create bundle parent", source))?;
    let temporary = parent.join(format!(
        ".tatacoa-encrypted-{}.partial",
        uuid::Uuid::new_v4()
    ));
    let plan = EnvelopeWritePlan {
        manifest: &exported,
        header: &header_bytes,
        directory: &directory,
        descriptors: &descriptors,
        bundle_key: &bundle_key,
        manifest_bytes: &manifest_bytes,
        directory_nonce: &directory_nonce,
    };
    let result = plan.write(workspace, &temporary);
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if let Err(source) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(crate::Error::io("commit encrypted bundle", source));
    }
    Ok(())
}

struct EnvelopeWritePlan<'a> {
    manifest: &'a Manifest,
    header: &'a [u8; HEADER_BYTES],
    directory: &'a [u8],
    descriptors: &'a [ObjectDescriptor],
    bundle_key: &'a crate::crypto::SecretKey,
    manifest_bytes: &'a [u8],
    directory_nonce: &'a [u8; 7],
}

impl EnvelopeWritePlan<'_> {
    fn write(&self, workspace: &Path, temporary: &Path) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temporary)
            .map_err(|source| crate::Error::io("create encrypted bundle staging", source))?;
        let mut writer = BufWriter::new(file);
        writer
            .write_all(self.header)
            .map_err(|source| crate::Error::io("write encrypted header", source))?;
        let directory_dek = derive_object_dek(self.bundle_key, b"directory", b"root")?;
        let directory_written = encrypt_stream_io(
            &directory_dek,
            self.directory_nonce,
            self.directory.len() as u64,
            CHUNK_BYTES as usize,
            &directory_aad(self.header),
            &mut Cursor::new(self.directory),
            &mut writer,
        )?;
        if directory_written != stream_ciphertext_length(self.directory.len() as u64)? {
            return Err(crate::Error::Cryptography("encrypted length mismatch"));
        }
        for (index, descriptor) in self.descriptors.iter().enumerate() {
            let dek = derive_object_dek(
                self.bundle_key,
                descriptor.object_type.domain(),
                descriptor.object_id.as_bytes(),
            )?;
            let aad = object_aad(self.header, descriptor)?;
            if index == 0 {
                let written = encrypt_stream_io(
                    &dek,
                    &descriptor.stream_nonce,
                    descriptor.plaintext_length,
                    CHUNK_BYTES as usize,
                    &aad,
                    &mut Cursor::new(self.manifest_bytes),
                    &mut writer,
                )?;
                if written != descriptor.ciphertext_length {
                    return Err(crate::Error::Cryptography("encrypted length mismatch"));
                }
            } else {
                let artifact = &self.manifest.artifacts[index - 1];
                let path = artifact_source_path(workspace, &self.manifest.engagement.id, artifact)?;
                reject_symlink(&path, "workspace artifact")?;
                let mut source = File::open(path)
                    .map_err(|source| crate::Error::io("open workspace artifact", source))?;
                let metadata = source
                    .metadata()
                    .map_err(|source| crate::Error::io("inspect workspace artifact", source))?;
                if !metadata.is_file() || metadata.len() != descriptor.plaintext_length {
                    return Err(crate::Error::Conflict(
                        "workspace artifact changed during encrypted export".to_owned(),
                    ));
                }
                let mut hashing_source = HashingReader::new(&mut source);
                let written = encrypt_stream_io(
                    &dek,
                    &descriptor.stream_nonce,
                    descriptor.plaintext_length,
                    CHUNK_BYTES as usize,
                    &aad,
                    &mut hashing_source,
                    &mut writer,
                )?;
                let mut trailing = [0_u8; 1];
                if hashing_source
                    .read(&mut trailing)
                    .map_err(|source| crate::Error::io("finish workspace artifact", source))?
                    != 0
                {
                    return Err(crate::Error::Conflict(
                        "workspace artifact changed during encrypted export".to_owned(),
                    ));
                }
                let (actual_size, actual_digest) = hashing_source.finish();
                if actual_size != artifact.size_bytes
                    || artifact.digest.algorithm != "SHA-256"
                    || !actual_digest.eq_ignore_ascii_case(&artifact.digest.value)
                {
                    return Err(crate::Error::Conflict(
                        "workspace artifact changed during encrypted export".to_owned(),
                    ));
                }
                if written != descriptor.ciphertext_length {
                    return Err(crate::Error::Cryptography("encrypted length mismatch"));
                }
            }
        }
        writer
            .flush()
            .map_err(|source| crate::Error::io("flush encrypted bundle", source))?;
        writer
            .get_ref()
            .sync_all()
            .map_err(|source| crate::Error::io("sync encrypted bundle", source))?;
        Ok(())
    }
}

struct HashingReader<R> {
    inner: R,
    digest: Sha256,
    size: u64,
}

impl<R> HashingReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            digest: Sha256::new(),
            size: 0,
        }
    }

    fn finish(self) -> (u64, String) {
        let bytes = self.digest.finalize();
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            let _ = write!(encoded, "{byte:02x}");
        }
        (self.size, encoded)
    }
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.digest.update(&buffer[..read]);
        self.size = self
            .size
            .checked_add(read as u64)
            .ok_or_else(|| std::io::Error::other("artifact size overflow"))?;
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SecurityProfile, create_engagement, create_environment, create_scope, create_session,
        create_target, execute, verify_encrypted_bundle,
    };
    use std::io::{Seek, SeekFrom};

    #[test]
    fn encrypted_export_round_trip_rejects_wrong_password_and_tampering() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("tatacoa-encrypted-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root)
            .map_err(|source| crate::Error::io("create test workspace", source))?;
        let result = run_round_trip(&root);
        let _ = fs::remove_dir_all(&root);
        result
    }

    fn run_round_trip(root: &Path) -> Result<()> {
        let engagement = create_engagement(
            root,
            "encrypted test".to_owned(),
            SecurityProfile::LabLearning,
        )?;
        let scope = create_scope(
            root,
            &engagement.id,
            "scope".to_owned(),
            "authorized test process".to_owned(),
        )?;
        let environment = create_environment(root, &engagement.id, &scope.id, "local".to_owned())?;
        let target = create_target(
            root,
            &engagement.id,
            &scope.id,
            &environment.id,
            "test binary".to_owned(),
            "local".to_owned(),
        )?;
        let session = create_session(
            root,
            &engagement.id,
            &scope.id,
            &environment.id,
            &target.id,
            "test".to_owned(),
        )?;
        let executable = std::env::current_exe()
            .map_err(|source| crate::Error::io("find test executable", source))?;
        let manifest = execute(
            root,
            &engagement.id,
            &session.id,
            executable.to_string_lossy().into_owned(),
            vec!["--list".to_owned()],
            None,
        )?;
        let password = SecretPassword::for_export("correct horse battery staple".to_owned())?;
        let bundle = root.join("bundle.tatacoa-encrypted");
        export_encrypted_bundle(root, &manifest, &bundle, &password)?;
        let verified = verify_encrypted_bundle(&bundle, &password)?;
        assert_eq!(verified.execution.id, manifest.execution.id);

        let wrong = SecretPassword::for_verification("wrong password".to_owned())?;
        assert!(verify_encrypted_bundle(&bundle, &wrong).is_err());

        let original = fs::read(&bundle)
            .map_err(|source| crate::Error::io("read test encrypted bundle", source))?;
        let truncated = root.join("truncated.tatacoa-encrypted");
        fs::write(&truncated, &original[..original.len() - 1])
            .map_err(|source| crate::Error::io("write truncated test bundle", source))?;
        assert!(verify_encrypted_bundle(&truncated, &password).is_err());
        let appended = root.join("appended.tatacoa-encrypted");
        let mut appended_bytes = original.clone();
        appended_bytes.push(0);
        fs::write(&appended, appended_bytes)
            .map_err(|source| crate::Error::io("write appended test bundle", source))?;
        assert!(verify_encrypted_bundle(&appended, &password).is_err());

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&bundle)
            .map_err(|source| crate::Error::io("open test bundle", source))?;
        file.seek(SeekFrom::End(-1))
            .map_err(|source| crate::Error::io("seek test bundle", source))?;
        let mut byte = [0_u8; 1];
        file.read_exact(&mut byte)
            .map_err(|source| crate::Error::io("read test bundle", source))?;
        byte[0] ^= 1;
        file.seek(SeekFrom::End(-1))
            .map_err(|source| crate::Error::io("seek test bundle", source))?;
        file.write_all(&byte)
            .map_err(|source| crate::Error::io("tamper test bundle", source))?;
        drop(file);
        assert!(verify_encrypted_bundle(&bundle, &password).is_err());

        let artifact = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.size_bytes > 0)
            .ok_or_else(|| crate::Error::Execution("test produced no artifact bytes".to_owned()))?;
        let artifact_path = artifact_source_path(root, &engagement.id, artifact)?;
        let mut artifact_bytes = fs::read(&artifact_path)
            .map_err(|source| crate::Error::io("read test artifact", source))?;
        artifact_bytes[0] ^= 1;
        fs::write(&artifact_path, artifact_bytes)
            .map_err(|source| crate::Error::io("tamper test artifact", source))?;
        let changed_destination = root.join("changed-source.tatacoa-encrypted");
        assert!(export_encrypted_bundle(root, &manifest, &changed_destination, &password).is_err());
        assert!(!changed_destination.exists());
        Ok(())
    }
}
