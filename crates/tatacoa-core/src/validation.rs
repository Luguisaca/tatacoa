use crate::{
    Artifact, ArtifactClassification, EngagementId, Error, EvidenceState, ExecutionId,
    KNOWLEDGE_SCHEMA_VERSION, KnowledgeCard, KnowledgeReviewStatus, ProvenanceKind,
    REPLAY_SCHEMA_VERSION, ReplayRecipe, Result, SourceClassification,
};
use std::collections::{HashMap, HashSet, VecDeque};

pub(crate) fn validate_knowledge_card(
    card: &KnowledgeCard,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
) -> Result<()> {
    if card.schema_version != KNOWLEDGE_SCHEMA_VERSION {
        return Err(Error::InvalidManifest(format!(
            "unsupported knowledge schema: {}",
            card.schema_version
        )));
    }
    if &card.engagement_id != engagement_id || &card.execution_id != execution_id {
        return Err(Error::InvalidManifest(format!(
            "knowledge card {} has a cross-context reference",
            card.id
        )));
    }
    for (name, value) in [
        ("WHAT", &card.what),
        ("WHY", &card.why),
        ("OBJECTIVE", &card.objective),
        ("HOW", &card.how),
        ("OBSERVE", &card.observe),
        ("PROVES", &card.proves),
        ("DOES_NOT_PROVE", &card.does_not_prove),
        ("ERRORS", &card.errors),
        ("VALIDATION", &card.validation),
        ("DEFENSIVE_CONTEXT", &card.defensive_context),
    ] {
        if value.trim().is_empty() {
            return Err(Error::InvalidManifest(format!(
                "knowledge card {} has empty {name}",
                card.id
            )));
        }
    }
    if card.references.is_empty() {
        return Err(Error::InvalidManifest(format!(
            "knowledge card {} has no classified references",
            card.id
        )));
    }
    for reference in &card.references {
        if reference.locator.trim().is_empty() || reference.title.trim().is_empty() {
            return Err(Error::InvalidManifest(format!(
                "knowledge card {} contains an incomplete reference",
                card.id
            )));
        }
    }
    if card.review_status == KnowledgeReviewStatus::SourceReviewed
        && card
            .references
            .iter()
            .any(|reference| reference.classification == SourceClassification::AiDraft)
    {
        return Err(Error::InvalidManifest(format!(
            "knowledge card {} cannot be source-reviewed while citing AI_DRAFT",
            card.id
        )));
    }
    Ok(())
}

pub(crate) fn validate_replay_recipe(
    recipe: &ReplayRecipe,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
) -> Result<()> {
    if recipe.schema_version != REPLAY_SCHEMA_VERSION {
        return Err(Error::InvalidManifest(format!(
            "unsupported replay schema: {}",
            recipe.schema_version
        )));
    }
    if &recipe.engagement_id != engagement_id || &recipe.source_execution_id != execution_id {
        return Err(Error::InvalidManifest(format!(
            "replay recipe {} has a cross-context origin",
            recipe.id
        )));
    }
    if recipe.executable.trim().is_empty() {
        return Err(Error::InvalidManifest(format!(
            "replay recipe {} has an empty executable",
            recipe.id
        )));
    }
    if recipe.authorization_limits.is_empty()
        || recipe
            .authorization_limits
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(Error::InvalidManifest(format!(
            "replay recipe {} requires explicit authorization limits",
            recipe.id
        )));
    }
    let mut names = HashSet::new();
    for placeholder in &recipe.placeholders {
        if !valid_placeholder_name(&placeholder.name)
            || placeholder.description.trim().is_empty()
            || !names.insert(placeholder.name.clone())
        {
            return Err(Error::InvalidManifest(format!(
                "replay recipe {} has an invalid or duplicate placeholder",
                recipe.id
            )));
        }
    }
    let referenced = template_placeholders(&recipe.argv_template)?;
    if referenced != names {
        return Err(Error::InvalidManifest(format!(
            "replay recipe {} placeholder declarations do not match argv_template",
            recipe.id
        )));
    }
    Ok(())
}

pub(crate) fn validate_artifact_provenance(artifacts: &[Artifact]) -> Result<()> {
    let by_id: HashMap<_, _> = artifacts
        .iter()
        .map(|artifact| (artifact.id.as_str(), artifact))
        .collect();
    for artifact in artifacts {
        if artifact.evidence_state == EvidenceState::Validated {
            return Err(Error::InvalidManifest(format!(
                "artifact {} claims VALIDATED without a human validation record",
                artifact.id
            )));
        }
        match artifact.classification {
            ArtifactClassification::Raw => {
                if artifact.provenance.kind != ProvenanceKind::Capture
                    || !artifact.provenance.source_artifact_ids.is_empty()
                {
                    return Err(Error::InvalidManifest(format!(
                        "RAW artifact {} must have CAPTURE provenance and no artifact sources",
                        artifact.id
                    )));
                }
            }
            ArtifactClassification::Derived => {
                if artifact.provenance.kind == ProvenanceKind::Capture
                    || artifact.provenance.source_artifact_ids.is_empty()
                {
                    return Err(Error::InvalidManifest(format!(
                        "derived artifact {} requires a transformation and source artifacts",
                        artifact.id
                    )));
                }
            }
        }
        for source in &artifact.provenance.source_artifact_ids {
            if !by_id.contains_key(source.as_str()) {
                return Err(Error::InvalidManifest(format!(
                    "artifact {} references missing provenance source {}",
                    artifact.id, source
                )));
            }
        }
    }

    let mut dependency_count: HashMap<&str, usize> = artifacts
        .iter()
        .map(|artifact| {
            (
                artifact.id.as_str(),
                artifact.provenance.source_artifact_ids.len(),
            )
        })
        .collect();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for artifact in artifacts {
        for source in &artifact.provenance.source_artifact_ids {
            dependents
                .entry(source.as_str())
                .or_default()
                .push(artifact.id.as_str());
        }
    }
    let mut ready: VecDeque<&str> = dependency_count
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect();
    let mut visited = 0_usize;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        if let Some(children) = dependents.get(id) {
            for child in children {
                let count = dependency_count.get_mut(child).ok_or_else(|| {
                    Error::InvalidManifest(format!("missing provenance node {child}"))
                })?;
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.push_back(child);
                }
            }
        }
    }
    if visited != artifacts.len() {
        return Err(Error::InvalidManifest(
            "provenance graph contains a cycle".to_owned(),
        ));
    }
    Ok(())
}

fn valid_placeholder_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn template_placeholders(arguments: &[String]) -> Result<HashSet<String>> {
    let mut names = HashSet::new();
    for argument in arguments {
        let mut remainder = argument.as_str();
        loop {
            let Some(start) = remainder.find("{{") else {
                if remainder.contains("}}") {
                    return Err(Error::InvalidManifest(
                        "replay argv_template contains an unmatched closing placeholder".to_owned(),
                    ));
                }
                break;
            };
            if remainder[..start].contains("}}") {
                return Err(Error::InvalidManifest(
                    "replay argv_template contains an unmatched closing placeholder".to_owned(),
                ));
            }
            let after_open = &remainder[start + 2..];
            let end = after_open.find("}}").ok_or_else(|| {
                Error::InvalidManifest(
                    "replay argv_template contains an unclosed placeholder".to_owned(),
                )
            })?;
            let name = &after_open[..end];
            if !valid_placeholder_name(name) || name.contains("{{") {
                return Err(Error::InvalidManifest(format!(
                    "replay argv_template contains invalid placeholder: {name}"
                )));
            }
            names.insert(name.to_owned());
            remainder = &after_open[end + 2..];
        }
    }
    Ok(names)
}
