use crate::{Error, Result, compute_sha256, read_bundle_manifest, safe_join_existing};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const PLAIN_ROOT_VERSION: &str = "tatacoa.plain-root.v1";
const DOMAIN: &[u8] = b"tatacoa.plain-root.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainRootDigest {
    pub algorithm: &'static str,
    pub value: String,
    pub entry_count: u32,
}

pub fn compute_plain_root(bundle_root: &Path) -> Result<PlainRootDigest> {
    let manifest = read_bundle_manifest(bundle_root)?;
    let mut entries = vec![("manifest.json".to_owned(), None)];
    entries.extend(
        manifest
            .artifacts
            .iter()
            .map(|a| (a.path.clone(), Some((a.size_bytes, a.digest.value.clone())))),
    );
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(Error::InvalidManifest(
            "duplicate Plain root path".to_owned(),
        ));
    }
    let entry_count = u32::try_from(entries.len())
        .map_err(|_| Error::InvalidManifest("too many Plain root entries".to_owned()))?;
    let mut root = Sha256::new();
    root.update(DOMAIN);
    root.update(entry_count.to_be_bytes());
    for (relative, expected) in entries {
        let path_bytes = relative.as_bytes();
        let path_len = u32::try_from(path_bytes.len())
            .map_err(|_| Error::InvalidPath("Plain root path is too long".to_owned()))?;
        let path = safe_join_existing(bundle_root, Path::new(&relative))?;
        let (size, digest) = compute_sha256(&path)?;
        if let Some((expected_size, expected_digest)) = expected.as_ref()
            && (size != *expected_size || digest != *expected_digest)
        {
            return Err(Error::InvalidManifest(format!(
                "Plain root artifact integrity failed: {relative}"
            )));
        }
        let digest = decode_sha256(&digest)?;
        root.update(path_len.to_be_bytes());
        root.update(path_bytes);
        root.update(size.to_be_bytes());
        root.update(digest);
    }
    let mut value = String::with_capacity(64);
    for byte in root.finalize() {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}")
            .map_err(|e| Error::Execution(format!("format Plain root digest: {e}")))?;
    }
    Ok(PlainRootDigest {
        algorithm: "SHA-256",
        value,
        entry_count,
    })
}

fn decode_sha256(value: &str) -> Result<[u8; 32]> {
    if value.len() != 64 {
        return Err(Error::InvalidManifest(
            "invalid SHA-256 length in Plain root".to_owned(),
        ));
    }
    let mut out = [0_u8; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let text = std::str::from_utf8(pair)
            .map_err(|_| Error::InvalidManifest("invalid SHA-256 encoding".to_owned()))?;
        out[index] = u8::from_str_radix(text, 16)
            .map_err(|_| Error::InvalidManifest("invalid SHA-256 encoding".to_owned()))?;
    }
    Ok(out)
}
