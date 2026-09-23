//! Read-only, offline assistance. This module never launches a process or
//! changes an Execution, its artifacts, or their evidence state.

use crate::{ArtifactRole, Manifest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_LOCAL_DOCUMENT_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssistanceLevel {
    Generic,
    Documented,
    Adapted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssistanceStatus {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactKind {
    ObservedFact,
    DocumentedFact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssistanceFact {
    pub kind: FactKind,
    pub label: String,
    pub value: String,
    /// Manifest field or local-document identity; never an evidence promotion.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolAssistance {
    pub level: AssistanceLevel,
    pub documentation: AssistanceStatus,
    pub adapter: AssistanceStatus,
    pub facts: Vec<AssistanceFact>,
}

/// A provider must be registered explicitly for a known tool. It must not
/// launch the tool, use a network, or parse engagement artifacts to answer.
pub trait LocalDocumentationProvider {
    fn documentation_for(&self, executable: &str) -> Option<LocalDocumentation>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDocumentation {
    pub executable: String,
    pub title: String,
    pub source_path: PathBuf,
}

/// Declaration only. Probes/parsers are deliberately not run by this
/// foundation; an integration must separately establish their safety.
pub trait SpecializedAdapter {
    fn matches(&self, executable: &str) -> bool;
    fn identifier(&self) -> &'static str;
}

pub fn assist_execution(
    manifest: &Manifest,
    provider: Option<&dyn LocalDocumentationProvider>,
    adapter: Option<&dyn SpecializedAdapter>,
) -> ToolAssistance {
    let execution = &manifest.execution;
    let origin = format!("manifest.execution:{}", execution.id);
    let mut facts = vec![
        observed(
            "Executable",
            &execution.invocation.executable,
            &format!("{origin}.invocation.executable"),
        ),
        observed(
            "Arguments",
            &format!("{:?}", execution.invocation.argv),
            &format!("{origin}.invocation.argv"),
        ),
        observed(
            "Shell",
            &execution.invocation.shell.to_string(),
            &format!("{origin}.invocation.shell"),
        ),
        observed(
            "Capture status",
            &format!("{:?}", execution.capture_status),
            &format!("{origin}.capture_status"),
        ),
        observed(
            "Exit code",
            &format!("{:?}", execution.exit_code),
            &format!("{origin}.exit_code"),
        ),
    ];
    for artifact in &manifest.artifacts {
        let role = match artifact.role {
            ArtifactRole::Stdout => "stdout",
            ArtifactRole::Stderr => "stderr",
        };
        facts.push(observed(
            &format!("{role} artifact"),
            &format!(
                "{} bytes; {}:{}",
                artifact.size_bytes, artifact.digest.algorithm, artifact.digest.value
            ),
            &format!("manifest.artifacts:{}", artifact.id),
        ));
    }
    let documentation = provider
        .and_then(|registered| registered.documentation_for(&execution.invocation.executable))
        .filter(|doc| {
            doc.executable == execution.invocation.executable && !doc.title.trim().is_empty()
        })
        .and_then(|doc| {
            read_documentation(&doc.source_path).map(|(content, digest)| (doc, content, digest))
        });
    let documentation_status = if let Some((doc, content, digest)) = documentation {
        facts.push(AssistanceFact {
            kind: FactKind::DocumentedFact,
            label: doc.title,
            value: content,
            source: format!("{}#sha256={digest}", doc.source_path.display()),
        });
        AssistanceStatus::Available
    } else {
        AssistanceStatus::Unavailable
    };
    let adapter_status = if adapter.is_some_and(|candidate| {
        candidate.matches(&execution.invocation.executable) && !candidate.identifier().is_empty()
    }) {
        AssistanceStatus::Available
    } else {
        AssistanceStatus::Unavailable
    };
    let level = if adapter_status == AssistanceStatus::Available {
        AssistanceLevel::Adapted
    } else if documentation_status == AssistanceStatus::Available {
        AssistanceLevel::Documented
    } else {
        AssistanceLevel::Generic
    };
    ToolAssistance {
        level,
        documentation: documentation_status,
        adapter: adapter_status,
        facts,
    }
}

fn read_documentation(path: &Path) -> Option<(String, String)> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_LOCAL_DOCUMENT_BYTES
    {
        return None;
    }
    let mut bytes = Vec::new();
    File::open(path)
        .ok()?
        .take(MAX_LOCAL_DOCUMENT_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_LOCAL_DOCUMENT_BYTES {
        return None;
    }
    let content = String::from_utf8(bytes).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    let digest = Sha256::digest(content.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Some((content, digest))
}

fn observed(label: &str, value: &str, source: &str) -> AssistanceFact {
    AssistanceFact {
        kind: FactKind::ObservedFact,
        label: label.to_owned(),
        value: value.to_owned(),
        source: source.to_owned(),
    }
}
