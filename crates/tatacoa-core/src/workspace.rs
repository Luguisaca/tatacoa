use crate::bundle::{
    engagement_existing_subdirectory, ensure_engagement_subdirectory, load_execution_manifest,
    read_json_limited, unix_ms_observed, write_json_new_atomic,
};
use crate::{
    EngagementId, Environment, EnvironmentId, Error, ExecutionContext, ExecutionId,
    KNOWLEDGE_SCHEMA_VERSION, KnowledgeCard, KnowledgeCardInput, KnowledgeId,
    REPLAY_SCHEMA_VERSION, ReplayId, ReplayRecipe, ReplayRecipeInput, Result, Scope, ScopeId,
    Session, SessionId, Target, TargetId,
};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};

pub fn create_scope(
    workspace: &Path,
    engagement_id: &EngagementId,
    name: String,
    authorization_boundary: String,
) -> Result<Scope> {
    require_text("scope name", &name)?;
    require_text("authorization boundary", &authorization_boundary)?;
    crate::load_engagement(workspace, engagement_id)?;
    let scope = Scope {
        id: ScopeId::new(),
        engagement_id: engagement_id.clone(),
        name,
        authorization_boundary,
        created_unix_ms_observed: unix_ms_observed()?,
    };
    store_context_record(
        workspace,
        engagement_id,
        "scopes",
        scope.id.as_str(),
        &scope,
    )?;
    Ok(scope)
}

pub fn create_environment(
    workspace: &Path,
    engagement_id: &EngagementId,
    scope_id: &ScopeId,
    name: String,
) -> Result<Environment> {
    require_text("environment name", &name)?;
    let scope = load_scope(workspace, engagement_id, scope_id)?;
    let environment = Environment {
        id: EnvironmentId::new(),
        engagement_id: engagement_id.clone(),
        scope_id: scope.id,
        name,
        created_unix_ms_observed: unix_ms_observed()?,
    };
    store_context_record(
        workspace,
        engagement_id,
        "environments",
        environment.id.as_str(),
        &environment,
    )?;
    Ok(environment)
}

pub fn create_target(
    workspace: &Path,
    engagement_id: &EngagementId,
    scope_id: &ScopeId,
    environment_id: &EnvironmentId,
    label: String,
    locator: String,
) -> Result<Target> {
    require_text("target label", &label)?;
    require_text("target locator", &locator)?;
    let scope = load_scope(workspace, engagement_id, scope_id)?;
    let environment = load_environment(workspace, engagement_id, environment_id)?;
    if environment.scope_id != scope.id {
        return Err(Error::InvalidManifest(
            "target environment belongs to another scope".to_owned(),
        ));
    }
    let target = Target {
        id: TargetId::new(),
        engagement_id: engagement_id.clone(),
        scope_id: scope.id,
        environment_id: environment.id,
        label,
        locator,
        created_unix_ms_observed: unix_ms_observed()?,
    };
    store_context_record(
        workspace,
        engagement_id,
        "targets",
        target.id.as_str(),
        &target,
    )?;
    Ok(target)
}

pub fn create_session(
    workspace: &Path,
    engagement_id: &EngagementId,
    scope_id: &ScopeId,
    environment_id: &EnvironmentId,
    target_id: &TargetId,
    name: String,
) -> Result<Session> {
    require_text("session name", &name)?;
    let scope = load_scope(workspace, engagement_id, scope_id)?;
    let environment = load_environment(workspace, engagement_id, environment_id)?;
    let target = load_target(workspace, engagement_id, target_id)?;
    if environment.scope_id != scope.id
        || target.scope_id != scope.id
        || target.environment_id != environment.id
    {
        return Err(Error::InvalidManifest(
            "session context contains cross-context relationships".to_owned(),
        ));
    }
    let session = Session {
        id: SessionId::new(),
        engagement_id: engagement_id.clone(),
        scope_id: scope.id,
        environment_id: environment.id,
        target_id: target.id,
        name,
        created_unix_ms_observed: unix_ms_observed()?,
    };
    store_context_record(
        workspace,
        engagement_id,
        "sessions",
        session.id.as_str(),
        &session,
    )?;
    Ok(session)
}

pub fn load_execution_context(
    workspace: &Path,
    engagement_id: &EngagementId,
    session_id: &SessionId,
) -> Result<ExecutionContext> {
    let session = load_session(workspace, engagement_id, session_id)?;
    let scope = load_scope(workspace, engagement_id, &session.scope_id)?;
    let environment = load_environment(workspace, engagement_id, &session.environment_id)?;
    let target = load_target(workspace, engagement_id, &session.target_id)?;
    let context = ExecutionContext {
        scope,
        environment,
        target,
        session,
    };
    context.validate(engagement_id)?;
    Ok(context)
}

pub fn list_sessions(workspace: &Path, engagement_id: &EngagementId) -> Result<Vec<Session>> {
    let sessions: Vec<Session> = load_records(workspace, engagement_id, "context/sessions")?;
    sessions
        .into_iter()
        .map(|session| {
            if &session.engagement_id != engagement_id {
                return Err(Error::InvalidManifest(
                    "session belongs to another engagement".to_owned(),
                ));
            }
            load_execution_context(workspace, engagement_id, &session.id)?;
            Ok(session)
        })
        .collect()
}

pub fn create_knowledge_card(
    workspace: &Path,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
    input: KnowledgeCardInput,
) -> Result<KnowledgeCard> {
    let manifest = load_execution_manifest(workspace, engagement_id, execution_id)?;
    require_text("knowledge WHAT", &input.what)?;
    require_text("knowledge WHY", &input.why)?;
    require_text("knowledge OBJECTIVE", &input.objective)?;
    require_text("knowledge HOW", &input.how)?;
    require_text("knowledge OBSERVE", &input.observe)?;
    require_text("knowledge PROVES", &input.proves)?;
    require_text("knowledge DOES_NOT_PROVE", &input.does_not_prove)?;
    require_text("knowledge ERRORS", &input.errors)?;
    require_text("knowledge VALIDATION", &input.validation)?;
    require_text("knowledge DEFENSIVE_CONTEXT", &input.defensive_context)?;
    if input.references.is_empty() {
        return Err(Error::InvalidManifest(
            "knowledge card requires at least one classified reference".to_owned(),
        ));
    }
    let card = KnowledgeCard {
        schema_version: KNOWLEDGE_SCHEMA_VERSION.to_owned(),
        id: KnowledgeId::new(),
        engagement_id: engagement_id.clone(),
        execution_id: manifest.execution.id,
        review_status: input.review_status,
        what: input.what,
        why: input.why,
        objective: input.objective,
        how: input.how,
        observe: input.observe,
        proves: input.proves,
        does_not_prove: input.does_not_prove,
        errors: input.errors,
        validation: input.validation,
        defensive_context: input.defensive_context,
        references: input.references,
        related_techniques: input.related_techniques,
        created_unix_ms_observed: unix_ms_observed()?,
    };
    crate::validate_knowledge_card(&card, engagement_id, execution_id)?;
    store_record(
        workspace,
        engagement_id,
        "knowledge",
        card.id.as_str(),
        &card,
    )?;
    Ok(card)
}

pub fn create_replay_recipe(
    workspace: &Path,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
    input: ReplayRecipeInput,
) -> Result<ReplayRecipe> {
    let manifest = load_execution_manifest(workspace, engagement_id, execution_id)?;
    let context = manifest.execution.context.ok_or_else(|| {
        Error::InvalidManifest("legacy execution has no replayable full context".to_owned())
    })?;
    let recipe = ReplayRecipe {
        schema_version: REPLAY_SCHEMA_VERSION.to_owned(),
        id: ReplayId::new(),
        engagement_id: engagement_id.clone(),
        source_execution_id: execution_id.clone(),
        context,
        executable: input.executable,
        argv_template: input.argv_template,
        placeholders: input.placeholders,
        prerequisites: input.prerequisites,
        authorization_limits: input.authorization_limits,
        created_unix_ms_observed: unix_ms_observed()?,
    };
    crate::validate_replay_recipe(&recipe, engagement_id, execution_id)?;
    store_record(
        workspace,
        engagement_id,
        "replay",
        recipe.id.as_str(),
        &recipe,
    )?;
    Ok(recipe)
}

pub fn load_associated_knowledge(
    workspace: &Path,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
) -> Result<Vec<KnowledgeCard>> {
    let records: Vec<KnowledgeCard> = load_records(workspace, engagement_id, "knowledge")?;
    records
        .into_iter()
        .filter(|record| &record.execution_id == execution_id)
        .map(|record| {
            crate::validate_knowledge_card(&record, engagement_id, execution_id)?;
            Ok(record)
        })
        .collect()
}

pub fn load_associated_replay(
    workspace: &Path,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
) -> Result<Vec<ReplayRecipe>> {
    let records: Vec<ReplayRecipe> = load_records(workspace, engagement_id, "replay")?;
    records
        .into_iter()
        .filter(|record| &record.source_execution_id == execution_id)
        .map(|record| {
            crate::validate_replay_recipe(&record, engagement_id, execution_id)?;
            Ok(record)
        })
        .collect()
}

fn load_scope(workspace: &Path, engagement_id: &EngagementId, id: &ScopeId) -> Result<Scope> {
    let scope: Scope = load_context_record(workspace, engagement_id, "scopes", id.as_str())?;
    if &scope.engagement_id != engagement_id || &scope.id != id {
        return Err(Error::InvalidManifest(
            "scope identity does not match its engagement/path".to_owned(),
        ));
    }
    Ok(scope)
}

fn load_environment(
    workspace: &Path,
    engagement_id: &EngagementId,
    id: &EnvironmentId,
) -> Result<Environment> {
    let environment: Environment =
        load_context_record(workspace, engagement_id, "environments", id.as_str())?;
    if &environment.engagement_id != engagement_id || &environment.id != id {
        return Err(Error::InvalidManifest(
            "environment identity does not match its engagement/path".to_owned(),
        ));
    }
    Ok(environment)
}

fn load_target(workspace: &Path, engagement_id: &EngagementId, id: &TargetId) -> Result<Target> {
    let target: Target = load_context_record(workspace, engagement_id, "targets", id.as_str())?;
    if &target.engagement_id != engagement_id || &target.id != id {
        return Err(Error::InvalidManifest(
            "target identity does not match its engagement/path".to_owned(),
        ));
    }
    Ok(target)
}

fn load_session(workspace: &Path, engagement_id: &EngagementId, id: &SessionId) -> Result<Session> {
    let session: Session = load_context_record(workspace, engagement_id, "sessions", id.as_str())?;
    if &session.engagement_id != engagement_id || &session.id != id {
        return Err(Error::InvalidManifest(
            "session identity does not match its engagement/path".to_owned(),
        ));
    }
    Ok(session)
}

fn store_context_record<T: serde::Serialize>(
    workspace: &Path,
    engagement_id: &EngagementId,
    kind: &str,
    id: &str,
    value: &T,
) -> Result<()> {
    let directory = context_directory(workspace, engagement_id, kind)?;
    write_json_new_atomic(&directory.join(format!("{id}.json")), value)
}

fn load_context_record<T: DeserializeOwned>(
    workspace: &Path,
    engagement_id: &EngagementId,
    kind: &str,
    id: &str,
) -> Result<T> {
    let directory = context_directory(workspace, engagement_id, kind)?;
    read_json_limited(&directory.join(format!("{id}.json")))
}

fn context_directory(
    workspace: &Path,
    engagement_id: &EngagementId,
    kind: &str,
) -> Result<PathBuf> {
    ensure_engagement_subdirectory(
        workspace,
        engagement_id,
        Path::new("context").join(kind).as_path(),
    )
}

fn store_record<T: serde::Serialize>(
    workspace: &Path,
    engagement_id: &EngagementId,
    kind: &str,
    id: &str,
    value: &T,
) -> Result<()> {
    let directory = ensure_engagement_subdirectory(workspace, engagement_id, Path::new(kind))?;
    write_json_new_atomic(&directory.join(format!("{id}.json")), value)
}

fn load_records<T: DeserializeOwned>(
    workspace: &Path,
    engagement_id: &EngagementId,
    kind: &str,
) -> Result<Vec<T>> {
    let root = crate::bundle::engagement_root(workspace, engagement_id)?;
    if !root.join(kind).exists() {
        return Ok(Vec::new());
    }
    let directory = engagement_existing_subdirectory(workspace, engagement_id, Path::new(kind))?;
    let mut paths = fs::read_dir(&directory)
        .map_err(|source| Error::io("read engagement record directory", source))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|source| Error::io("read engagement record entry", source))
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    paths
        .into_iter()
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| read_json_limited(&path))
        .collect()
}

fn require_text(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::InvalidManifest(format!("{field} must not be empty")));
    }
    Ok(())
}
