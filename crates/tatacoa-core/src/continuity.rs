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
    let path = paths
        .pop()
        .ok_or_else(|| Error::InvalidManifest("continuity history is empty".to_owned()))?;
    let bytes = fs::read(path).map_err(|source| Error::io("read continuity state", source))?;
    let state: ContinuityState = serde_json::from_slice(&bytes)?;
    if state.schema_version != CONTINUITY_SCHEMA_VERSION || &state.engagement_id != engagement_id {
        return Err(Error::InvalidManifest(
            "invalid continuity state identity/version".to_owned(),
        ));
    }
    Ok(state)
}
fn store(workspace: &Path, state: &ContinuityState) -> Result<()> {
    let root = engagement_root(workspace, &state.engagement_id)?.join("continuity");
    fs::create_dir_all(&root).map_err(|source| Error::io("create continuity history", source))?;
    let name = format!(
        "{:032}-{}.json",
        state.updated_unix_ms_observed,
        uuid::Uuid::new_v4().simple()
    );
    write_json_new_atomic(&root.join(name), state)
}
