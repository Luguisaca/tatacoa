use crate::{Error, Result};
use std::path::{Component, Path, PathBuf};

pub fn ensure_safe_relative_path(path: &Path) -> Result<()> {
    let rendered = path.to_string_lossy();
    if rendered.is_empty() {
        return Err(Error::InvalidPath("path is empty".to_owned()));
    }
    if rendered.contains('\\') || rendered.contains(':') {
        return Err(Error::InvalidPath(format!(
            "platform-specific or ambiguous separator/prefix in {rendered}"
        )));
    }
    if path.is_absolute() {
        return Err(Error::InvalidPath(format!("absolute path {rendered}")));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(Error::InvalidPath(format!(
                    "non-normal component in {rendered}"
                )));
            }
        }
    }
    Ok(())
}

pub fn safe_join_existing(root: &Path, relative: &Path) -> Result<PathBuf> {
    ensure_safe_relative_path(relative)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|source| Error::io("canonicalize bundle root", source))?;
    let candidate = canonical_root.join(relative);

    let mut current = canonical_root.clone();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(Error::InvalidPath("non-normal path component".to_owned()));
        };
        current.push(name);
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|source| Error::io("inspect bundle path", source))?;
        if metadata.file_type().is_symlink() {
            return Err(Error::InvalidPath(format!(
                "symbolic link/reparse point is not allowed: {}",
                current.display()
            )));
        }
    }

    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|source| Error::io("canonicalize bundle object", source))?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(Error::InvalidPath(format!(
            "path escapes bundle root: {}",
            relative.display()
        )));
    }
    Ok(canonical_candidate)
}

#[cfg(test)]
mod tests {
    use super::ensure_safe_relative_path;
    use std::path::Path;

    #[test]
    fn accepts_portable_object_path() {
        assert!(ensure_safe_relative_path(Path::new("objects/art_123.bin")).is_ok());
    }

    #[test]
    fn rejects_traversal_and_platform_prefixes() {
        for path in [
            "../outside",
            "objects/../../outside",
            "/absolute",
            "C:/absolute",
            "C:relative",
            r"objects\mixed",
            r"\\server\share\file",
            "./objects/file",
        ] {
            assert!(
                ensure_safe_relative_path(Path::new(path)).is_err(),
                "accepted unsafe path: {path}"
            );
        }
    }
}
