use crate::bundle::{engagement_root, unix_ms_observed, write_json_new_atomic};
use crate::{EngagementId, Error, Result, SessionId};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONTINUITY_SCHEMA_VERSION: &str = "tatacoa.continuity.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContinuityStatus {
    Active,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContinuityState {
    pub schema_version: String,
    pub engagement_id: EngagementId,
    pub status: ContinuityStatus,
    pub current_session_id: Option<SessionId>,
    pub pending: Vec<String>,
    #[serde(default)]
    pub revision: u64,
    pub updated_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContinuityInspection {
    pub state: ContinuityState,
    pub recovery_required: bool,
    pub incomplete_capture_count: usize,
}

pub(crate) fn initialize(workspace: &Path, engagement_id: &EngagementId) -> Result<()> {
    store(
        workspace,
        &ContinuityState {
            schema_version: CONTINUITY_SCHEMA_VERSION.to_owned(),
            engagement_id: engagement_id.clone(),
            status: ContinuityStatus::Active,
            current_session_id: None,
            pending: Vec::new(),
            revision: 1,
            updated_unix_ms_observed: unix_ms_observed()?,
        },
    )
}

pub fn pause_work(
    workspace: &Path,
    engagement_id: &EngagementId,
    current_session_id: Option<SessionId>,
    pending: Vec<String>,
) -> Result<ContinuityState> {
    let mut state = load(workspace, engagement_id)?;
    if pending.iter().any(|item| item.trim().is_empty()) {
        return Err(Error::InvalidManifest(
            "continuity pending items must not be empty".to_owned(),
        ));
    }
    state.status = ContinuityStatus::Paused;
    state.current_session_id = current_session_id;
    state.pending = pending;
    state.revision = state
        .revision
        .checked_add(1)
        .ok_or_else(|| Error::InvalidManifest("continuity revision overflow".to_owned()))?;
    state.updated_unix_ms_observed = unix_ms_observed()?;
    store(workspace, &state)?;
    Ok(state)
}

pub fn resume_work(
    workspace: &Path,
    engagement_id: &EngagementId,
    authorization_revalidated: bool,
) -> Result<ContinuityState> {
    if !authorization_revalidated {
        return Err(Error::InvalidManifest(
            "current authorization must be revalidated before resuming".to_owned(),
        ));
    }
    let mut state = load(workspace, engagement_id)?;
    state.status = ContinuityStatus::Active;
    state.revision = state
        .revision
        .checked_add(1)
        .ok_or_else(|| Error::InvalidManifest("continuity revision overflow".to_owned()))?;
    state.updated_unix_ms_observed = unix_ms_observed()?;
    store(workspace, &state)?;
    Ok(state)
}

pub fn inspect_continuity(
    workspace: &Path,
    engagement_id: &EngagementId,
) -> Result<ContinuityInspection> {
    let state = load(workspace, engagement_id)?;
    let objects = engagement_root(workspace, engagement_id)?.join("objects");
    let mut count = 0;
    for entry in
        fs::read_dir(objects).map_err(|source| Error::io("inspect continuity objects", source))?
    {
        let path = entry
            .map_err(|source| Error::io("inspect continuity entry", source))?
            .path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.') && n.ends_with(".partial"))
        {
            count += 1;
        }
    }
    Ok(ContinuityInspection {
        state,
        recovery_required: count > 0,
        incomplete_capture_count: count,
    })
}

fn load(workspace: &Path, engagement_id: &EngagementId) -> Result<ContinuityState> {
    let root = engagement_root(workspace, engagement_id)?.join("continuity");
    if !root.exists() {
        let engagement = crate::load_engagement(workspace, engagement_id)?;
        return Ok(ContinuityState {
            schema_version: CONTINUITY_SCHEMA_VERSION.to_owned(),
            engagement_id: engagement_id.clone(),
            status: ContinuityStatus::Active,
            current_session_id: None,
            pending: Vec::new(),
            revision: 0,
            updated_unix_ms_observed: engagement.created_unix_ms_observed,
        });
    }
    let mut paths = fs::read_dir(root)
        .map_err(|source| Error::io("read continuity history", source))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|source| Error::io("read continuity entry", source))
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    let mut states = Vec::with_capacity(paths.len());
    for path in paths {
        let bytes = fs::read(&path).map_err(|source| Error::io("read continuity state", source))?;
        let state: ContinuityState = serde_json::from_slice(&bytes)?;
        if state.schema_version != CONTINUITY_SCHEMA_VERSION
            || &state.engagement_id != engagement_id
        {
            return Err(Error::InvalidManifest(
                "invalid continuity state identity/version".to_owned(),
            ));
        }
        states.push((path, state));
    }
    if states.is_empty() {
        return Err(Error::InvalidManifest(
            "continuity history is empty".to_owned(),
        ));
    }
    let mut revisions = std::collections::HashSet::new();
    for (_, state) in &states {
        if state.revision > 0 && !revisions.insert(state.revision) {
            return Err(Error::InvalidManifest(format!(
                "duplicate continuity revision: {}",
                state.revision
            )));
        }
    }
    if let Some((_, state)) = states
        .iter()
        .filter(|(_, state)| state.revision > 0)
        .max_by_key(|(_, state)| state.revision)
    {
        return Ok(state.clone());
    }
    // Legacy SP3 snapshots had no revision. Their former filename order is used once
    // for compatibility; every subsequent write receives revision 1 and no longer
    // relies on observed wall-clock time.
    states
        .pop()
        .map(|(_, state)| state)
        .ok_or_else(|| Error::InvalidManifest("continuity history is empty".to_owned()))
}
fn store(workspace: &Path, state: &ContinuityState) -> Result<()> {
    let root = engagement_root(workspace, &state.engagement_id)?.join("continuity");
    fs::create_dir_all(&root).map_err(|source| Error::io("create continuity history", source))?;
    let name = format!(
        "{:020}-{}.json",
        state.revision,
        uuid::Uuid::new_v4().simple()
    );
    write_json_new_atomic(&root.join(name), state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SecurityProfile, create_engagement};

    fn root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "tatacoa-continuity-{label}-{}",
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn revision_not_wall_clock_selects_latest_state() -> Result<()> {
        let root = root("revision");
        let engagement = create_engagement(&root, "test".to_owned(), SecurityProfile::LabLearning)?;
        let mut state = load(&root, &engagement.id)?;
        state.revision = 2;
        state.status = ContinuityStatus::Paused;
        state.updated_unix_ms_observed = u128::MAX;
        store(&root, &state)?;
        state.revision = 7;
        state.status = ContinuityStatus::Active;
        state.updated_unix_ms_observed = 0;
        store(&root, &state)?;
        assert_eq!(load(&root, &engagement.id)?.revision, 7);
        assert_eq!(
            load(&root, &engagement.id)?.status,
            ContinuityStatus::Active
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn legacy_snapshot_is_read_then_upgraded_on_write() -> Result<()> {
        let root = root("legacy");
        let engagement = create_engagement(&root, "test".to_owned(), SecurityProfile::LabLearning)?;
        let history = engagement_root(&root, &engagement.id)?.join("continuity");
        fs::remove_dir_all(&history)
            .map_err(|source| Error::io("replace test continuity", source))?;
        fs::create_dir(&history).map_err(|source| Error::io("create legacy history", source))?;
        let legacy = ContinuityState {
            schema_version: CONTINUITY_SCHEMA_VERSION.to_owned(),
            engagement_id: engagement.id.clone(),
            status: ContinuityStatus::Paused,
            current_session_id: None,
            pending: vec!["legacy".to_owned()],
            revision: 0,
            updated_unix_ms_observed: 99,
        };
        let mut legacy_value = serde_json::to_value(&legacy)?;
        legacy_value
            .as_object_mut()
            .ok_or_else(|| Error::InvalidManifest("test legacy state is not an object".to_owned()))?
            .remove("revision");
        fs::write(
            history.join("00099-legacy.json"),
            serde_json::to_vec(&legacy_value)?,
        )
        .map_err(|source| Error::io("write legacy snapshot", source))?;
        let mut later = legacy.clone();
        later.pending = vec!["legacy-latest".to_owned()];
        let mut later_value = serde_json::to_value(&later)?;
        later_value
            .as_object_mut()
            .ok_or_else(|| Error::InvalidManifest("test legacy state is not an object".to_owned()))?
            .remove("revision");
        fs::write(
            history.join("00100-legacy.json"),
            serde_json::to_vec(&later_value)?,
        )
        .map_err(|source| Error::io("write later legacy snapshot", source))?;
        assert_eq!(load(&root, &engagement.id)?.revision, 0);
        assert_eq!(load(&root, &engagement.id)?.pending, later.pending);
        let resumed = resume_work(&root, &engagement.id, true)?;
        assert_eq!(resumed.revision, 1);
        assert_eq!(load(&root, &engagement.id)?.revision, 1);
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn duplicate_and_overflow_revisions_fail_closed() -> Result<()> {
        let duplicate_root = root("duplicates");
        let engagement = create_engagement(
            &duplicate_root,
            "test".to_owned(),
            SecurityProfile::LabLearning,
        )?;
        let mut state = load(&duplicate_root, &engagement.id)?;
        state.revision = 2;
        store(&duplicate_root, &state)?;
        store(&duplicate_root, &state)?;
        assert!(load(&duplicate_root, &engagement.id).is_err());

        let overflow_root = root("overflow");
        let engagement = create_engagement(
            &overflow_root,
            "test".to_owned(),
            SecurityProfile::LabLearning,
        )?;
        let mut state = load(&overflow_root, &engagement.id)?;
        state.revision = u64::MAX;
        store(&overflow_root, &state)?;
        assert!(pause_work(&overflow_root, &engagement.id, None, Vec::new()).is_err());
        let _ = fs::remove_dir_all(duplicate_root);
        let _ = fs::remove_dir_all(overflow_root);
        Ok(())
    }
}
