#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::str::FromStr;
use tatacoa_app_api::{
    AppService, AuthorizationReview, ExportRequest, KnowledgeFromExecutionRequest,
    KnowledgeRequest, NewWorkRequest, NoteFromExecutionRequest, ReplayFromExecutionRequest,
    ReplayRequest, RunRequest, TimestampRequest, TimestampVerifyRequest, WorkContext, WorkSummary,
};
use tatacoa_core::{
    ArtifactId, ArtifactPreview, ContinuityState, Engagement, EngagementId, ExecutionId,
    KnowledgeCard, Manifest, ReplayRecipe, SessionId, TimestampReport, ToolAssistance,
};
use tauri_plugin_dialog::DialogExt;

type CommandResult<T> = Result<T, String>;

#[tauri::command]
fn select_workspace(app: tauri::AppHandle) -> CommandResult<Option<String>> {
    app.dialog()
        .file()
        .set_title("Seleccionar workspace TATACOA")
        .blocking_pick_folder()
        .map(|path| {
            path.into_path()
                .map(|value| value.to_string_lossy().into_owned())
                .map_err(|error| error.to_string())
        })
        .transpose()
}

#[tauri::command]
fn select_export_destination(app: tauri::AppHandle) -> CommandResult<Option<String>> {
    app.dialog()
        .file()
        .set_title("Destino de exportación TATACOA")
        .blocking_save_file()
        .map(|path| {
            path.into_path()
                .map(|value| value.to_string_lossy().into_owned())
                .map_err(|error| error.to_string())
        })
        .transpose()
}

#[tauri::command]
fn select_timestamp_sidecar(app: tauri::AppHandle) -> CommandResult<Option<String>> {
    app.dialog()
        .file()
        .set_title("Destino sidecar RFC 3161 (.tsr)")
        .add_filter("RFC 3161 response", &["tsr"])
        .blocking_save_file()
        .map(|path| {
            path.into_path()
                .map(|value| value.to_string_lossy().into_owned())
                .map_err(|error| error.to_string())
        })
        .transpose()
}

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
fn run_tool(workspace: String, request: RunRequest) -> CommandResult<Manifest> {
    AppService::open(workspace)
        .run(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn authorization_review(
    workspace: String,
    engagement_id: String,
    session_id: String,
) -> CommandResult<AuthorizationReview> {
    let engagement_id =
        EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    let session_id = SessionId::from_str(&session_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .authorization_review(&engagement_id, &session_id)
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
fn execution_workspace(
    workspace: String,
    engagement_id: String,
    execution_id: String,
) -> CommandResult<Manifest> {
    let engagement_id =
        EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    let execution_id = ExecutionId::from_str(&execution_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .execution_workspace(&engagement_id, &execution_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn execution_assistance(
    workspace: String,
    engagement_id: String,
    execution_id: String,
) -> CommandResult<ToolAssistance> {
    let engagement_id =
        EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    let execution_id = ExecutionId::from_str(&execution_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .execution_assistance(&engagement_id, &execution_id)
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
fn create_knowledge_from_execution(
    workspace: String,
    request: KnowledgeFromExecutionRequest,
) -> CommandResult<KnowledgeCard> {
    AppService::open(workspace)
        .create_knowledge_from_execution(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_replay_from_execution(
    workspace: String,
    request: ReplayFromExecutionRequest,
) -> CommandResult<ReplayRecipe> {
    AppService::open(workspace)
        .create_replay_from_execution(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_note_from_execution(
    workspace: String,
    request: NoteFromExecutionRequest,
) -> CommandResult<KnowledgeCard> {
    AppService::open(workspace)
        .create_note_from_execution(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_execution(workspace: String, request: ExportRequest) -> CommandResult<()> {
    AppService::open(workspace)
        .export(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn request_timestamp(
    workspace: String,
    request: TimestampRequest,
) -> CommandResult<TimestampReport> {
    AppService::open(workspace)
        .request_timestamp(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn verify_timestamp(
    workspace: String,
    request: TimestampVerifyRequest,
) -> CommandResult<TimestampReport> {
    AppService::open(workspace)
        .verify_timestamp(request)
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
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_work,
            select_workspace,
            select_export_destination,
            select_timestamp_sidecar,
            create_work,
            summarize,
            run_tool,
            authorization_review,
            execution,
            execution_workspace,
            execution_assistance,
            artifact_preview,
            create_knowledge,
            create_replay,
            create_knowledge_from_execution,
            create_replay_from_execution,
            create_note_from_execution,
            export_execution,
            request_timestamp,
            verify_timestamp,
            pause_work,
            resume_work
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| eprintln!("TATACOA desktop failed: {error}"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn desktop_security_profile_values_match_app_api_wire_contract() {
        let html = include_str!("../ui/index.html");
        for value in ["LAB_LEARNING", "PROFESSIONAL", "HIGH_SENSITIVITY"] {
            assert!(
                html.contains(&format!("value=\"{value}\"")),
                "desktop is missing SecurityProfile wire value {value}"
            );
        }
        for stale in ["LabLearning", "Professional", "HighSensitivity"] {
            assert!(
                !html.contains(&format!("value=\"{stale}\"")),
                "desktop still exposes stale SecurityProfile value {stale}"
            );
        }
        assert!(html.contains("id=\"execution-review\" hidden"));
        assert!(html.contains("id=\"confirm-run\""));
        assert!(html.contains("Confirmar y ejecutar"));
    }

    #[test]
    fn desktop_execution_workflow_reuses_recorded_facts_and_locates_outputs() {
        let html = include_str!("../ui/index.html");
        let javascript = include_str!("../ui/app.js");

        for id in [
            "execution-context",
            "execution-feedback",
            "knowledge-list",
            "replay-list",
            "replay-change-invocation",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")));
        }
        for redundant_field in [
            "name=\"what\"",
            "name=\"how\"",
            "name=\"errors\"",
            "id=\"replay-executable\"",
            "id=\"replay-argv\"",
        ] {
            assert!(!html.contains(redundant_field));
        }
        assert!(javascript.contains("execution_workspace"));
        assert!(javascript.contains("create_knowledge_from_execution"));
        assert!(javascript.contains("create_replay_from_execution"));
        assert!(javascript.contains("executable_override:changed?"));
        assert!(javascript.contains("argv_template_override:changed?"));
        for id in ["assistance-facts", "assistance-documentation", "note-form"] {
            assert!(html.contains(&format!("id=\"{id}\"")));
        }
        assert!(javascript.contains("execution_assistance"));
        assert!(javascript.contains("create_note_from_execution"));
        assert!(html.contains("Edición estructurada opcional"));
    }

    #[test]
    fn desktop_workflow_keeps_context_review_and_continuity_visible() {
        let html = include_str!("../ui/index.html");
        let javascript = include_str!("../ui/app.js");
        for id in [
            "continuity-section",
            "continuity-next",
            "activity-list",
            "work-context",
            "resume-review",
            "confirm-resume",
            "work-feedback",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")));
        }
        assert!(javascript.contains("const captured=await invoke('run_tool'"));
        assert!(javascript.contains("await openExecution(captured.execution.id"));
        assert!(javascript.contains("authorization_review"));
        assert!(javascript.contains("pendingResume"));
        assert!(!javascript.contains("confirm(`REVALIDACIÓN CONTEXTUAL"));
    }

    #[test]
    fn replay_preparation_is_optional_and_never_autoexecutes() {
        let html = include_str!("../ui/index.html");
        let javascript = include_str!("../ui/app.js");
        assert!(html.contains("Opciones avanzadas solo si son necesarias"));
        assert!(html.contains("id=\"replay-limits\""));
        assert!(!html.contains("id=\"replay-limits\" required"));
        assert!(javascript.contains("function prepareRetest(recipe)"));
        assert!(javascript.contains("if(recipe.placeholders.length)return showError"));
        assert!(javascript.contains("pendingRecipe=recipe;cancelRun()"));
        assert!(javascript.contains("await getAuthorizationReview()"));
        assert!(javascript.contains("Comparación de metadatos registrados"));
    }
}
