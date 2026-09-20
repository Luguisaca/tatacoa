#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::str::FromStr;
use tatacoa_app_api::{
    AppService, ExportRequest, KnowledgeRequest, NewWorkRequest, ReplayRequest, RunRequest,
    WorkContext, WorkSummary,
};
use tatacoa_core::{
    ArtifactId, ArtifactPreview, ContinuityState, Engagement, EngagementId, ExecutionId,
    KnowledgeCard, Manifest, ReplayRecipe, SessionId,
};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
fn list_work(workspace: String) -> CommandResult<Vec<Engagement>> {
    AppService::open(workspace)
        .list_work()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_work(workspace: String, request: NewWorkRequest) -> CommandResult<WorkContext> {
    AppService::open(workspace)
        .create_work(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn summarize(workspace: String, engagement_id: String) -> CommandResult<WorkSummary> {
    let id = EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .summarize(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn run_tool(workspace: String, request: RunRequest) -> CommandResult<WorkSummary> {
    let service = AppService::open(&workspace);
    let engagement_id = request.engagement_id.clone();
    service.run(request).map_err(|error| error.to_string())?;
    service
        .summarize(&engagement_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn execution(
    workspace: String,
    engagement_id: String,
    execution_id: String,
) -> CommandResult<Manifest> {
    let engagement_id =
        EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    let execution_id = ExecutionId::from_str(&execution_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .execution(&engagement_id, &execution_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn artifact_preview(
    workspace: String,
    engagement_id: String,
    execution_id: String,
    artifact_id: String,
) -> CommandResult<ArtifactPreview> {
    let engagement_id =
        EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    let execution_id = ExecutionId::from_str(&execution_id).map_err(|error| error.to_string())?;
    let artifact_id = ArtifactId::from_str(&artifact_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .artifact_preview(&engagement_id, &execution_id, &artifact_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_knowledge(workspace: String, request: KnowledgeRequest) -> CommandResult<KnowledgeCard> {
    AppService::open(workspace)
        .create_knowledge(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_replay(workspace: String, request: ReplayRequest) -> CommandResult<ReplayRecipe> {
    AppService::open(workspace)
        .create_replay(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_execution(workspace: String, request: ExportRequest) -> CommandResult<()> {
    AppService::open(workspace)
        .export(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn pause_work(
    workspace: String,
    engagement_id: String,
    session_id: Option<String>,
    pending: Vec<String>,
) -> CommandResult<ContinuityState> {
    let engagement_id =
        EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    let session_id = session_id
        .map(|id| SessionId::from_str(&id))
        .transpose()
        .map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .pause(&engagement_id, session_id, pending)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn resume_work(
    workspace: String,
    engagement_id: String,
    authorization_revalidated: bool,
) -> CommandResult<ContinuityState> {
    let engagement_id =
        EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .resume(&engagement_id, authorization_revalidated)
        .map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_work,
            create_work,
            summarize,
            run_tool,
            execution,
            artifact_preview,
            create_knowledge,
            create_replay,
            export_execution,
            pause_work,
            resume_work
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| eprintln!("TATACOA desktop failed: {error}"));
}
